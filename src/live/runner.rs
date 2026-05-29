use super::logging::{log_idle_if_due, log_worker_connected, log_worker_started};
use super::process::{handle_fetch_result, process_and_ack_message};
use super::signal::shutdown_signal;
use crate::error::AppResult;
use crate::nats::StructuredIntelConsumer;
use crate::telemetry;
use crate::time::now_ms;
use crate::worker::{CandidateWorker, WorkerArgs};
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveWorkerReport {
    pub worker_run_id: String,
    pub processed_messages: usize,
}

pub async fn run_live_worker(args: WorkerArgs) -> AppResult<LiveWorkerReport> {
    let worker_run_id = format!("intel-candidate-worker-{}", now_ms());
    log_worker_started(&worker_run_id, &args)?;
    let worker = CandidateWorker::connect(&args).await?;
    let mut consumer = StructuredIntelConsumer::connect(&args.nats).await?;
    log_worker_connected(&worker_run_id, &args)?;

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
                let Some(message) = handle_fetch_result(message_result, &worker_run_id).await? else {
                    last_idle_log_ms = log_idle_if_due(
                        &worker_run_id,
                        &args,
                        processed,
                        last_idle_log_ms,
                    )?;
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
    Ok(LiveWorkerReport {
        worker_run_id,
        processed_messages: processed,
    })
}
