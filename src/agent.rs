use crate::error::{AppError, AppResult};
use crate::live::{
    handle_fetch_result, log_error, log_idle_if_due, log_worker_connected, log_worker_started,
    process_and_ack_message, shutdown_signal,
};
use crate::nats::StructuredIntelConsumer;
use crate::telemetry;
use crate::time::now_ms;
use crate::worker::{CandidateWorker, WorkerArgs, worker_help};
use serde_json::json;
use std::time::Duration;

const DEFAULT_REPAIR_INTERVAL_SECS: u64 = 3_600;
const DEFAULT_REPAIR_MAX_KEYS_PER_PREFIX: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentArgs {
    pub worker: WorkerArgs,
    pub repair_input_prefixes: Vec<String>,
    pub repair_interval_secs: u64,
    pub repair_max_keys_per_prefix: usize,
    pub repair_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentReport {
    pub agent_run_id: String,
    pub live_messages_processed: usize,
    pub repair_cycles_completed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RepairCycleReport {
    keys_seen: usize,
    keys_processed: usize,
    keys_skipped_stale_revision: usize,
    keys_failed: usize,
}

impl AgentArgs {
    pub fn parse(values: impl Iterator<Item = String>) -> AppResult<Option<Self>> {
        let raw = values.collect::<Vec<_>>();
        if raw.iter().any(|value| value == "-h" || value == "--help") {
            return Ok(None);
        }

        let mut repair_input_prefixes = Vec::new();
        let mut repair_interval_secs = DEFAULT_REPAIR_INTERVAL_SECS;
        let mut repair_max_keys_per_prefix = DEFAULT_REPAIR_MAX_KEYS_PER_PREFIX;
        let mut repair_enabled = true;
        let mut worker_args = Vec::new();
        let mut index = 0usize;

        while index < raw.len() {
            match raw[index].as_str() {
                "--repair-input-prefix"
                | "--agent-repair-input-prefix"
                | "--replay-input-prefix" => {
                    index += 1;
                    repair_input_prefixes.push(next_value(&raw, index, raw[index - 1].as_str())?);
                }
                "--repair-interval-secs" | "--agent-repair-interval-secs" => {
                    index += 1;
                    repair_interval_secs =
                        parse_positive_u64(&next_value(&raw, index, raw[index - 1].as_str())?)?;
                }
                "--repair-max-keys-per-prefix" | "--agent-repair-max-keys-per-prefix" => {
                    index += 1;
                    repair_max_keys_per_prefix =
                        parse_positive_usize(&next_value(&raw, index, raw[index - 1].as_str())?)?;
                }
                "--disable-repair" | "--agent-disable-repair" => {
                    repair_enabled = false;
                }
                value => worker_args.push(value.to_owned()),
            }
            index += 1;
        }

        let Some(worker) = WorkerArgs::parse(worker_args.into_iter())? else {
            return Ok(None);
        };
        Ok(Some(Self {
            worker,
            repair_input_prefixes,
            repair_interval_secs,
            repair_max_keys_per_prefix,
            repair_enabled,
        }))
    }
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
                        let report = run_repair_cycle(&agent_run_id, &worker, &args).await?;
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

pub fn agent_help() -> String {
    format!(
        r#"intel-candidate-agent
Usage:
  intel-candidate-agent \
    --nats-url nats://REPLACE_WITH_S2S_NATS_HOST:4222 \
    --input-s3-bucket nangman-crypto-dev-intel-structuring-l1-<account-suffix> \
    --output-s3-bucket nangman-crypto-dev-intel-candidate-<account-suffix> \
    --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
    --policy-file /opt/nangman-crypto/intel-candidate/policies/scoring-policy.v1.json \
    --repair-input-prefix structured-intel-packet/schema=structured_intel_packet_v1/

The agent is the default AI-DLC execution unit. It continuously consumes live
structured intel pointers from NATS and runs bounded S3 repair scans inside the
same process when configured. S3 remains the canonical store; NATS remains the
pointer bus.

Agent-specific flags:
  --repair-input-prefix <s3-prefix>          Repeatable. Enables bounded S3 repair scans.
  --repair-interval-secs <positive>          Default: {DEFAULT_REPAIR_INTERVAL_SECS}
  --repair-max-keys-per-prefix <positive>    Default: {DEFAULT_REPAIR_MAX_KEYS_PER_PREFIX}
  --disable-repair                           Disable repair scans even when prefixes are configured.

Worker flags:
{}
"#,
        worker_help()
    )
}

async fn run_repair_cycle(
    agent_run_id: &str,
    worker: &CandidateWorker,
    args: &AgentArgs,
) -> AppResult<RepairCycleReport> {
    telemetry::info(
        "agent_repair_cycle_started",
        json!({
            "agent_run_id": agent_run_id,
            "repair_input_prefixes": &args.repair_input_prefixes,
            "repair_max_keys_per_prefix": args.repair_max_keys_per_prefix,
        }),
    )?;

    let mut all_keys = Vec::new();
    for prefix in &args.repair_input_prefixes {
        let keys = worker
            .list_replay_input_keys(prefix, args.repair_max_keys_per_prefix)
            .await?;
        telemetry::info(
            "agent_repair_prefix_scanned",
            json!({
                "agent_run_id": agent_run_id,
                "input_prefix": prefix,
                "keys": keys.len(),
            }),
        )?;
        all_keys.extend(keys);
    }
    all_keys.sort();
    all_keys.dedup();

    let mut report = RepairCycleReport {
        keys_seen: all_keys.len(),
        keys_processed: 0,
        keys_skipped_stale_revision: 0,
        keys_failed: 0,
    };
    for key in all_keys {
        match worker.process_s3_key(&key, now_ms()).await {
            Ok(Some(result)) => {
                report.keys_processed += 1;
                telemetry::info(
                    "agent_repair_key_processed",
                    json!({
                        "agent_run_id": agent_run_id,
                        "input_key": key,
                        "screening_event_id": result.screening_event.screening_event_id,
                        "candidate_id": result.screening_event.candidate_id,
                        "candidate_class": result.screening_event.candidate_class.as_policy_key(),
                        "research_eligible": result.screening_event.research_eligible,
                    }),
                )?;
            }
            Ok(None) => {
                report.keys_skipped_stale_revision += 1;
                telemetry::info(
                    "agent_repair_key_skipped_as_stale_revision",
                    json!({
                        "agent_run_id": agent_run_id,
                        "input_key": key,
                    }),
                )?;
            }
            Err(error) => {
                report.keys_failed += 1;
                log_error("agent_repair_key_failed", Some(agent_run_id), &error)?;
            }
        }
    }
    Ok(report)
}

fn repair_due(args: &AgentArgs, next_repair_after_ms: i64) -> bool {
    args.repair_enabled
        && !args.repair_input_prefixes.is_empty()
        && now_ms() >= next_repair_after_ms
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
            "live_input_stream": args.worker.nats.input_stream,
            "live_input_subject": args.worker.nats.input_subject,
            "live_input_consumer": args.worker.nats.input_consumer,
        }),
    )
}

fn log_repair_cycle_finished(
    agent_run_id: &str,
    repair_cycle_number: usize,
    report: &RepairCycleReport,
) -> AppResult<()> {
    telemetry::info(
        "agent_repair_cycle_finished",
        json!({
            "agent_run_id": agent_run_id,
            "repair_cycle_number": repair_cycle_number,
            "keys_seen": report.keys_seen,
            "keys_processed": report.keys_processed,
            "keys_skipped_stale_revision": report.keys_skipped_stale_revision,
            "keys_failed": report.keys_failed,
        }),
    )
}

fn next_value(values: &[String], index: usize, flag: &str) -> AppResult<String> {
    values
        .get(index)
        .cloned()
        .ok_or_else(|| AppError::config(format!("{flag} requires a value")))
}

fn parse_positive_u64(value: &str) -> AppResult<u64> {
    let parsed = value
        .parse::<u64>()
        .map_err(|error| AppError::config(format!("invalid positive integer {value}: {error}")))?;
    if parsed == 0 {
        return Err(AppError::config("value must be positive"));
    }
    Ok(parsed)
}

fn parse_positive_usize(value: &str) -> AppResult<usize> {
    let parsed = value
        .parse::<usize>()
        .map_err(|error| AppError::config(format!("invalid positive integer {value}: {error}")))?;
    if parsed == 0 {
        return Err(AppError::config("value must be positive"));
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_agent_flags_and_worker_flags() {
        let args = AgentArgs::parse(
            [
                "--nats-url",
                "nats://127.0.0.1:4222",
                "--input-s3-bucket",
                "test-structured-l1",
                "--output-s3-bucket",
                "test-candidate",
                "--market-l1-s3-bucket",
                "test-market-l1",
                "--repair-input-prefix",
                "structured-intel-packet/schema=structured_intel_packet_v1/",
                "--repair-interval-secs",
                "60",
                "--repair-max-keys-per-prefix",
                "11",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect("args parse")
        .expect("args present");
        assert_eq!(args.worker.nats.url, "nats://127.0.0.1:4222");
        assert_eq!(
            args.repair_input_prefixes,
            vec!["structured-intel-packet/schema=structured_intel_packet_v1/"]
        );
        assert_eq!(args.repair_interval_secs, 60);
        assert_eq!(args.repair_max_keys_per_prefix, 11);
        assert!(args.repair_enabled);
    }

    #[test]
    fn parse_disable_repair() {
        let args = AgentArgs::parse(
            [
                "--nats-url",
                "nats://127.0.0.1:4222",
                "--input-s3-bucket",
                "test-structured-l1",
                "--output-s3-bucket",
                "test-candidate",
                "--market-l1-s3-bucket",
                "test-market-l1",
                "--disable-repair",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect("args parse")
        .expect("args present");
        assert!(!args.repair_enabled);
    }
}
