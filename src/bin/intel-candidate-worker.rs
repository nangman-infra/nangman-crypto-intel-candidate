use intel_candidate_app::error::{AppError, AppResult};
use intel_candidate_app::nats::{StructuredIntelConsumer, StructuredIntelMessage};
use intel_candidate_app::scoring::screening_event_key;
use intel_candidate_app::telemetry;
use intel_candidate_app::time::now_ms;
use intel_candidate_app::worker::{CandidateWorker, WorkerArgs, worker_help};
use serde_json::json;
use std::process;

const IDLE_LOG_INTERVAL_MS: i64 = 60_000;

#[tokio::main]
async fn main() -> AppResult<()> {
    if let Err(error) = run().await {
        log_error("worker_fatal", None, &error)?;
        process::exit(1);
    }
    Ok(())
}

async fn run() -> AppResult<()> {
    let Some(args) = WorkerArgs::parse(std::env::args().skip(1))? else {
        println!("{}", worker_help());
        return Ok(());
    };
    let worker_run_id = format!("intel-candidate-worker-{}", now_ms());
    log_worker_started(&worker_run_id, &args)?;
    let worker = CandidateWorker::connect(&args).await?;
    let mut consumer = StructuredIntelConsumer::connect(&args.nats).await?;
    telemetry::info(
        "worker_connected",
        json!({
            "worker_run_id": worker_run_id,
            "input_stream": args.nats.input_stream,
            "input_subject": args.nats.input_subject,
            "input_consumer": args.nats.input_consumer,
            "input_deliver_policy": args.nats.input_deliver_policy,
            "output_stream": args.nats.output_stream,
            "output_s3_bucket": args.output_store.bucket,
        }),
    )?;
    let mut processed = 0usize;
    let mut last_idle_log_ms = 0i64;
    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);
    loop {
        if let Some(max_messages) = args.max_messages
            && processed >= max_messages
        {
            break;
        }
        tokio::select! {
            shutdown_result = &mut shutdown => {
                shutdown_result?;
                telemetry::info(
                    "shutdown_received",
                    json!({
                        "worker_run_id": worker_run_id,
                        "processed_messages": processed,
                    }),
                )?;
                break;
            }
            message_result = consumer.next_message() => {
                let Some(message) = (match message_result {
                    Ok(message) => message,
                    Err(error) => {
                        log_error("fetch_failed", Some(&worker_run_id), &error)?;
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        continue;
                    }
                }) else {
                    let current_ms = now_ms();
                    if current_ms - last_idle_log_ms >= IDLE_LOG_INTERVAL_MS {
                        telemetry::info(
                            "worker_idle",
                            json!({
                                "worker_run_id": worker_run_id,
                                "input_stream": args.nats.input_stream,
                                "input_consumer": args.nats.input_consumer,
                                "processed_messages": processed,
                            }),
                        )?;
                        last_idle_log_ms = current_ms;
                    }
                    if args.exit_on_idle {
                        break;
                    }
                    continue;
                };
                process_and_ack_message(&worker_run_id, &worker, message).await?;
                processed += 1;
            }
        }
    }
    telemetry::info(
        "worker_stopped",
        json!({
            "worker_run_id": worker_run_id,
            "processed_messages": processed,
        }),
    )?;
    Ok(())
}

async fn process_and_ack_message(
    worker_run_id: &str,
    worker: &CandidateWorker,
    message: StructuredIntelMessage,
) -> AppResult<()> {
    let pointer = match message.pointer() {
        Ok(pointer) => pointer,
        Err(error) => {
            log_error("pointer_parse_failed", Some(worker_run_id), &error)?;
            return Ok(());
        }
    };
    telemetry::info(
        "message_received",
        json!({
            "worker_run_id": worker_run_id,
            "packet_id": pointer.packet_id,
            "raw_event_id": pointer.raw_event_id,
            "input_s3_bucket": pointer.storage_ref.bucket,
            "input_s3_key": pointer.storage_ref.key,
            "input_schema_version": pointer.storage_ref.schema_version,
            "manifest_key": pointer.manifest_key,
            "pointer_created_at_ms": pointer.created_at_ms,
        }),
    )?;
    match worker.process_pointer(&pointer, now_ms()).await {
        Ok(Some(result)) => {
            let screening_s3_key = screening_event_key(
                result.screening_event.created_at_ms,
                &result.screening_event.screening_event_id,
            );
            let evidence_bundle_s3_key = result
                .evidence_bundle
                .as_ref()
                .map(|bundle| bundle.bundle_key.as_str());
            telemetry::info(
                "candidate_result_published",
                json!({
                    "worker_run_id": worker_run_id,
                    "packet_id": pointer.packet_id,
                    "raw_event_id": pointer.raw_event_id,
                    "screening_event_id": result.screening_event.screening_event_id,
                    "candidate_id": result.screening_event.candidate_id,
                    "candidate_class": result.screening_event.candidate_class.as_policy_key(),
                    "candidate_score": result.screening_event.candidate_score,
                    "research_eligible": result.screening_event.research_eligible,
                    "quarantine": result.screening_event.quarantine,
                    "screening_s3_key": screening_s3_key,
                    "evidence_bundle_created": evidence_bundle_s3_key.is_some(),
                    "evidence_bundle_s3_key": evidence_bundle_s3_key,
                    "reason_count": result.screening_event.reasons.len(),
                    "ack": "pending",
                }),
            )?;
            if let Err(error) = message.ack().await {
                log_error("ack_failed", Some(worker_run_id), &error)?;
                return Err(error);
            }
            telemetry::info(
                "message_acked",
                json!({
                    "worker_run_id": worker_run_id,
                    "packet_id": pointer.packet_id,
                    "raw_event_id": pointer.raw_event_id,
                    "screening_event_id": result.screening_event.screening_event_id,
                    "ack": "yes",
                }),
            )?;
        }
        Ok(None) => {
            telemetry::info(
                "message_skipped_as_stale_revision",
                json!({
                    "worker_run_id": worker_run_id,
                    "packet_id": pointer.packet_id,
                    "raw_event_id": pointer.raw_event_id,
                    "ack": "pending",
                }),
            )?;
            if let Err(error) = message.ack().await {
                log_error("ack_failed", Some(worker_run_id), &error)?;
                return Err(error);
            }
            telemetry::info(
                "message_acked",
                json!({
                    "worker_run_id": worker_run_id,
                    "packet_id": pointer.packet_id,
                    "raw_event_id": pointer.raw_event_id,
                    "ack": "yes",
                    "skipped": "stale_revision",
                }),
            )?;
        }
        Err(error) => {
            telemetry::error(
                "processing_failed",
                json!({
                    "worker_run_id": worker_run_id,
                    "packet_id": pointer.packet_id,
                    "raw_event_id": pointer.raw_event_id,
                    "input_s3_bucket": pointer.storage_ref.bucket,
                    "input_s3_key": pointer.storage_ref.key,
                    "ack": "no",
                    "error": error.to_string(),
                }),
            )?;
        }
    }
    Ok(())
}

fn log_worker_started(worker_run_id: &str, args: &WorkerArgs) -> AppResult<()> {
    telemetry::info(
        "worker_started",
        json!({
            "worker_run_id": worker_run_id,
            "input_stream": args.nats.input_stream,
            "input_subject": args.nats.input_subject,
            "input_consumer": args.nats.input_consumer,
            "input_deliver_policy": args.nats.input_deliver_policy,
            "output_stream": args.nats.output_stream,
            "bundle_subject": args.nats.bundle_subject,
            "screening_subject": args.nats.screening_subject,
            "hypothesis_state_subject": args.nats.hypothesis_state_subject,
            "health_subject": args.nats.health_subject,
            "input_s3_bucket": args.input_store.bucket,
            "output_s3_bucket": args.output_store.bucket,
            "market_l1_s3_bucket": args.market_store.bucket,
            "policy_file": args.policy_file.display().to_string(),
            "batch_size": args.nats.batch_size,
            "ack_wait_secs": args.nats.ack_wait_secs,
            "max_deliver": args.nats.max_deliver,
            "max_messages": args.max_messages,
            "exit_on_idle": args.exit_on_idle,
            "nats_url_configured": !args.nats.url.trim().is_empty(),
        }),
    )
}

fn log_error(event: &str, worker_run_id: Option<&str>, error: &AppError) -> AppResult<()> {
    telemetry::error(
        event,
        json!({
            "worker_run_id": worker_run_id,
            "ack": "no",
            "error": error.to_string()
        }),
    )
}

async fn shutdown_signal() -> AppResult<()> {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! {
            result = tokio::signal::ctrl_c() => result?,
            _ = terminate.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await?;
    }
    Ok(())
}
