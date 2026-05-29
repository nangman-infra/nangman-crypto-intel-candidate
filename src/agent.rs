use crate::error::AppResult;
use crate::live::{
    handle_fetch_result, log_idle_if_due, log_worker_connected, log_worker_started,
    process_and_ack_message, shutdown_signal,
};
use crate::nats::StructuredIntelConsumer;
use crate::telemetry;
use crate::time::now_ms;
use crate::worker::CandidateWorker;
use serde_json::json;
use std::time::Duration;

mod args;
mod repair;

pub use args::{AgentArgs, agent_help};
use repair::{RepairScanCursors, log_repair_cycle_finished, repair_due, run_repair_cycle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentReport {
    pub agent_run_id: String,
    pub live_messages_processed: usize,
    pub repair_cycles_completed: usize,
}

pub async fn run_agent(args: AgentArgs) -> AppResult<AgentReport> {
    let agent_run_id = format!("intel-candidate-agent-{}", now_ms());
    log_agent_started(&agent_run_id, &args)?;
    log_worker_started(&agent_run_id, &args.worker)?;
    let worker = CandidateWorker::connect(&args.worker).await?;
    let mut consumer = StructuredIntelConsumer::connect(&args.worker.nats).await?;
    log_worker_connected(&agent_run_id, &args.worker)?;

    let mut live_messages_processed = 0usize;
    let mut repair_cycles_completed = 0usize;
    let mut last_idle_log_ms = 0i64;
    let mut next_repair_after_ms = now_ms();
    let mut repair_scan_cursors = RepairScanCursors::default();
    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);

    loop {
        if let Some(max_messages) = args.worker.max_messages
            && live_messages_processed >= max_messages
        {
            break;
        }

        tokio::select! {
            shutdown_result = &mut shutdown => {
                shutdown_result?;
                telemetry::info(
                    "agent_shutdown_received",
                    json!({
                        "agent_run_id": agent_run_id,
                        "live_messages_processed": live_messages_processed,
                        "repair_cycles_completed": repair_cycles_completed,
                    }),
                )?;
                break;
            }
            message_result = consumer.next_message() => {
                let Some(message) = handle_fetch_result(message_result, &agent_run_id).await? else {
                    last_idle_log_ms = log_idle_if_due(
                        &agent_run_id,
                        &args.worker,
                        live_messages_processed,
                        last_idle_log_ms,
                    )?;
                    if repair_due(&args, next_repair_after_ms) {
                        let report = run_repair_cycle(
                            &agent_run_id,
                            &worker,
                            &args,
                            &mut repair_scan_cursors,
                        )
                        .await?;
                        repair_cycles_completed += 1;
                        next_repair_after_ms = now_ms()
                            + Duration::from_secs(args.repair_interval_secs).as_millis() as i64;
                        log_repair_cycle_finished(&agent_run_id, repair_cycles_completed, &report)?;
                    }
                    if args.worker.exit_on_idle {
                        break;
                    }
                    continue;
                };
                process_and_ack_message(&agent_run_id, &worker, message).await?;
                live_messages_processed += 1;
            }
        }
    }

    telemetry::info(
        "agent_stopped",
        json!({
            "agent_run_id": agent_run_id,
            "live_messages_processed": live_messages_processed,
            "repair_cycles_completed": repair_cycles_completed,
        }),
    )?;
    Ok(AgentReport {
        agent_run_id,
        live_messages_processed,
        repair_cycles_completed,
    })
}

fn log_agent_started(agent_run_id: &str, args: &AgentArgs) -> AppResult<()> {
    telemetry::info(
        "agent_started",
        json!({
            "agent_run_id": agent_run_id,
            "repair_enabled": args.repair_enabled,
            "repair_input_prefixes": &args.repair_input_prefixes,
            "repair_interval_secs": args.repair_interval_secs,
            "repair_max_keys_per_prefix": args.repair_max_keys_per_prefix,
            "repair_max_pages_per_prefix": args.repair_max_pages_per_prefix,
            "repair_recent_partition_days": args.repair_recent_partition_days,
            "live_input_stream": args.worker.nats.input_stream,
            "live_input_subject": args.worker.nats.input_subject,
            "live_input_consumer": args.worker.nats.input_consumer,
        }),
    )
}
