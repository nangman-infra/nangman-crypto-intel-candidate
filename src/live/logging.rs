use crate::error::{AppError, AppResult};
use crate::telemetry;
use crate::time::now_ms;
use crate::worker::WorkerArgs;
use serde_json::json;

pub const IDLE_LOG_INTERVAL_MS: i64 = 60_000;

pub fn log_idle_if_due(
    worker_run_id: &str,
    args: &WorkerArgs,
    processed: usize,
    last_idle_log_ms: i64,
) -> AppResult<i64> {
    let current_ms = now_ms();
    if current_ms - last_idle_log_ms < IDLE_LOG_INTERVAL_MS {
        return Ok(last_idle_log_ms);
    }
    telemetry::info(
        "worker_idle",
        json!({
            "worker_run_id": worker_run_id,
            "input_stream": args.nats.input_stream,
            "input_consumer": args.nats.input_consumer,
            "processed_messages": processed,
        }),
    )?;
    Ok(current_ms)
}

pub fn log_worker_started(worker_run_id: &str, args: &WorkerArgs) -> AppResult<()> {
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

pub fn log_worker_connected(worker_run_id: &str, args: &WorkerArgs) -> AppResult<()> {
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
    )
}

pub fn log_error(event: &str, worker_run_id: Option<&str>, error: &AppError) -> AppResult<()> {
    telemetry::error(
        event,
        json!({
            "worker_run_id": worker_run_id,
            "ack": "no",
            "error": error.to_string()
        }),
    )
}
