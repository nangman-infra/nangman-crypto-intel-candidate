use intel_candidate_app::error::{AppError, AppResult};
use intel_candidate_app::model::CandidateProcessingResult;
use intel_candidate_app::storage::ObjectStore;
use intel_candidate_app::telemetry;
use intel_candidate_app::time::{now_ms, path_segment, time_part};
use intel_candidate_app::worker::{CandidateWorker, WorkerArgs, worker_help};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::process;

const REPORT_SCHEMA_VERSION: &str = "intel_candidate_replay_report_v1";
const RESULT_SCHEMA_VERSION: &str = "intel_candidate_replay_result_v1";
const DEFAULT_MAX_KEYS: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReplayArgs {
    worker: WorkerArgs,
    input_prefixes: Vec<String>,
    max_keys_per_prefix: usize,
    created_at_ms: Option<i64>,
    report_prefix: String,
    result_prefix: String,
    fail_on_record_error: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ReplayReport {
    schema_version: String,
    replay_run_id: String,
    producer_app: String,
    producer_version: String,
    created_at_ms: i64,
    input_bucket: String,
    output_bucket: String,
    input_prefixes: Vec<String>,
    keys_seen: usize,
    keys_processed: usize,
    keys_skipped_stale_revision: usize,
    keys_failed: usize,
    result_records_created: usize,
    evidence_bundles_created: usize,
    hypothesis_states_created: usize,
    screening_events_created: usize,
    failed_keys: Vec<ReplayFailure>,
    result_prefix: String,
    report_key: String,
    checksum: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct ReplayResultRecord {
    schema_version: String,
    replay_run_id: String,
    created_at_ms: i64,
    input_bucket: String,
    input_key: String,
    input_key_sha256: String,
    result: CandidateProcessingResult,
    checksum: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ReplayFailure {
    input_key: String,
    error: String,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    if let Err(error) = run().await {
        telemetry::error("replay_worker_fatal", json!({ "error": error.to_string() }))?;
        process::exit(1);
    }
    Ok(())
}

async fn run() -> AppResult<()> {
    let Some(args) = parse_args(std::env::args().skip(1))? else {
        println!("{}", replay_help());
        return Ok(());
    };
    let created_at_ms = args.created_at_ms.unwrap_or_else(now_ms);
    let replay_run_id = format!("intel-candidate-replay-{}", created_at_ms);
    telemetry::info(
        "replay_started",
        json!({
            "replay_run_id": replay_run_id,
            "input_bucket": args.worker.input_store.bucket.as_str(),
            "output_bucket": args.worker.output_store.bucket.as_str(),
            "input_prefixes": &args.input_prefixes,
            "max_keys_per_prefix": args.max_keys_per_prefix,
        }),
    )?;

    let output_store = ObjectStore::connect(args.worker.output_store.clone()).await?;
    let worker = CandidateWorker::connect(&args.worker).await?;
    let mut all_keys = Vec::new();
    for prefix in &args.input_prefixes {
        let keys = worker
            .list_replay_input_keys(prefix, args.max_keys_per_prefix)
            .await?;
        telemetry::info(
            "replay_prefix_scanned",
            json!({
                "replay_run_id": replay_run_id,
                "input_prefix": prefix,
                "keys": keys.len(),
            }),
        )?;
        all_keys.extend(keys);
    }
    all_keys.sort();
    all_keys.dedup();

    let mut keys_processed = 0usize;
    let keys_skipped_stale_revision = 0usize;
    let mut evidence_bundles_created = 0usize;
    let mut hypothesis_states_created = 0usize;
    let mut screening_events_created = 0usize;
    let mut result_records_created = 0usize;
    let mut failed_keys = Vec::new();

    for key in &all_keys {
        let replay_result = async {
            let result = worker.score_s3_key(key, created_at_ms).await?;
            let result_key =
                replay_result_key(&args.result_prefix, created_at_ms, &replay_run_id, key);
            let mut result_record = ReplayResultRecord {
                schema_version: RESULT_SCHEMA_VERSION.to_owned(),
                replay_run_id: replay_run_id.clone(),
                created_at_ms,
                input_bucket: args.worker.input_store.bucket.clone(),
                input_key: key.clone(),
                input_key_sha256: sha256_hex(key.as_bytes()),
                result: result.clone(),
                checksum: String::new(),
            };
            result_record.checksum = checksum_json(&result_record);
            output_store
                .put_bytes_idempotent(
                    &result_key,
                    serde_json::to_vec_pretty(&result_record)?,
                    "application/json",
                )
                .await?;
            AppResult::Ok((result, result_key))
        }
        .await;
        match replay_result {
            Ok((result, result_key)) => {
                keys_processed += 1;
                result_records_created += 1;
                screening_events_created += 1;
                if result.evidence_bundle.is_some() {
                    evidence_bundles_created += 1;
                }
                if result.hypothesis_state.is_some() {
                    hypothesis_states_created += 1;
                }
                telemetry::info(
                    "replay_key_processed",
                    json!({
                        "replay_run_id": replay_run_id,
                        "input_key": key,
                        "screening_event_id": result.screening_event.screening_event_id,
                        "candidate_class": result.screening_event.candidate_class.as_policy_key(),
                        "research_eligible": result.screening_event.research_eligible,
                        "replay_result_s3_key": result_key,
                        "evidence_bundle_created": result.evidence_bundle.is_some(),
                        "hypothesis_state_created": result.hypothesis_state.is_some(),
                    }),
                )?;
            }
            Err(error) => {
                failed_keys.push(ReplayFailure {
                    input_key: key.clone(),
                    error: error.to_string(),
                });
                telemetry::error(
                    "replay_key_failed",
                    json!({
                        "replay_run_id": replay_run_id,
                        "input_key": key,
                        "error": error.to_string(),
                    }),
                )?;
            }
        }
    }

    let report_key = replay_report_key(&args.report_prefix, created_at_ms, &replay_run_id);
    let mut report = ReplayReport {
        schema_version: REPORT_SCHEMA_VERSION.to_owned(),
        replay_run_id: replay_run_id.clone(),
        producer_app: "intel-candidate-app".to_owned(),
        producer_version: env!("CARGO_PKG_VERSION").to_owned(),
        created_at_ms,
        input_bucket: args.worker.input_store.bucket.clone(),
        output_bucket: args.worker.output_store.bucket.clone(),
        input_prefixes: args.input_prefixes.clone(),
        keys_seen: all_keys.len(),
        keys_processed,
        keys_skipped_stale_revision,
        keys_failed: failed_keys.len(),
        result_records_created,
        evidence_bundles_created,
        hypothesis_states_created,
        screening_events_created,
        failed_keys,
        result_prefix: args.result_prefix.clone(),
        report_key: report_key.clone(),
        checksum: String::new(),
    };
    report.checksum = checksum_json(&report);
    output_store
        .put_bytes_idempotent(
            &report_key,
            serde_json::to_vec_pretty(&report)?,
            "application/json",
        )
        .await?;
    telemetry::info(
        "replay_finished",
        json!({
            "replay_run_id": replay_run_id,
            "keys_seen": report.keys_seen,
            "keys_processed": report.keys_processed,
            "keys_skipped_stale_revision": report.keys_skipped_stale_revision,
            "keys_failed": report.keys_failed,
            "result_records_created": report.result_records_created,
            "evidence_bundles_created": report.evidence_bundles_created,
            "hypothesis_states_created": report.hypothesis_states_created,
            "report_s3_key": report.report_key,
        }),
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if args.fail_on_record_error && report.keys_failed > 0 {
        return Err(AppError::validation(format!(
            "candidate replay finished with {} failed keys; report_key={}",
            report.keys_failed, report.report_key
        )));
    }
    Ok(())
}

fn parse_args(values: impl Iterator<Item = String>) -> AppResult<Option<ReplayArgs>> {
    let raw = values.collect::<Vec<_>>();
    if raw.iter().any(|value| value == "-h" || value == "--help") {
        return Ok(None);
    }
    let mut input_prefixes = Vec::new();
    let mut max_keys_per_prefix = DEFAULT_MAX_KEYS;
    let mut created_at_ms = None;
    let mut report_prefix = "candidate-replay-report".to_owned();
    let mut result_prefix = "candidate-replay-result".to_owned();
    let mut fail_on_record_error = true;
    let mut worker_args = Vec::new();
    let mut index = 0usize;
    while index < raw.len() {
        match raw[index].as_str() {
            "--replay-input-prefix" => {
                index += 1;
                input_prefixes.push(next_value(&raw, index, "--replay-input-prefix")?);
            }
            "--replay-max-keys-per-prefix" => {
                index += 1;
                max_keys_per_prefix = parse_positive_usize(&next_value(
                    &raw,
                    index,
                    "--replay-max-keys-per-prefix",
                )?)?;
            }
            "--replay-created-at-ms" => {
                index += 1;
                created_at_ms = Some(parse_non_negative_i64(&next_value(
                    &raw,
                    index,
                    "--replay-created-at-ms",
                )?)?);
            }
            "--replay-report-prefix" => {
                index += 1;
                report_prefix = next_value(&raw, index, "--replay-report-prefix")?;
            }
            "--replay-result-prefix" => {
                index += 1;
                result_prefix = next_value(&raw, index, "--replay-result-prefix")?;
            }
            "--replay-continue-on-record-error" => {
                fail_on_record_error = false;
            }
            value => worker_args.push(value.to_owned()),
        }
        index += 1;
    }
    if input_prefixes.is_empty() {
        return Err(AppError::config("--replay-input-prefix is required"));
    }
    let Some(worker) = WorkerArgs::parse(worker_args.into_iter())? else {
        return Ok(None);
    };
    Ok(Some(ReplayArgs {
        worker,
        input_prefixes,
        max_keys_per_prefix,
        created_at_ms,
        report_prefix,
        result_prefix,
        fail_on_record_error,
    }))
}

fn next_value(values: &[String], index: usize, flag: &str) -> AppResult<String> {
    values
        .get(index)
        .cloned()
        .ok_or_else(|| AppError::config(format!("{flag} requires a value")))
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

fn parse_non_negative_i64(value: &str) -> AppResult<i64> {
    let parsed = value
        .parse::<i64>()
        .map_err(|error| AppError::config(format!("invalid timestamp {value}: {error}")))?;
    if parsed < 0 {
        return Err(AppError::config("timestamp must be non-negative"));
    }
    Ok(parsed)
}

fn replay_report_key(prefix: &str, created_at_ms: i64, replay_run_id: &str) -> String {
    let part = time_part(created_at_ms);
    format!(
        "{}/schema={}/dt={}/hour={:02}/replay_run_id={}/report.json",
        prefix.trim_matches('/'),
        REPORT_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(replay_run_id)
    )
}

fn replay_result_key(
    prefix: &str,
    created_at_ms: i64,
    replay_run_id: &str,
    input_key: &str,
) -> String {
    let part = time_part(created_at_ms);
    format!(
        "{}/schema={}/dt={}/hour={:02}/replay_run_id={}/input_key_sha256={}/result.json",
        prefix.trim_matches('/'),
        RESULT_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(replay_run_id),
        sha256_hex(input_key.as_bytes())
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn checksum_json<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    sha256_hex(&bytes)
}

fn replay_help() -> String {
    format!(
        r#"intel-candidate-replay-worker
Usage:
  intel-candidate-replay-worker \
    --nats-url nats://REPLACE_WITH_S2S_NATS_HOST:4222 \
    --input-s3-bucket nangman-crypto-dev-intel-structuring-l1-<account-suffix> \
    --output-s3-bucket nangman-crypto-dev-intel-candidate-<account-suffix> \
    --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
    --policy-file /opt/nangman-crypto/intel-candidate/policies/scoring-policy.v1.json \
    --replay-input-prefix structured-intel-packet/schema=structured_intel_packet_v1/

Replay-specific flags:
  --replay-input-prefix <s3-prefix>          Repeatable. Scans durable S3 inputs, not NATS retention.
  --replay-max-keys-per-prefix <positive>   Default: {DEFAULT_MAX_KEYS}
  --replay-created-at-ms <timestamp-ms>      Optional deterministic report/output time.
  --replay-report-prefix <s3-prefix>         Default: candidate-replay-report
  --replay-result-prefix <s3-prefix>         Default: candidate-replay-result
  --replay-continue-on-record-error          Write the report and exit 0 even when individual keys fail.

Worker flags:
{}
"#,
        worker_help()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_key_is_partitioned() {
        assert_eq!(
            replay_report_key("candidate-replay-report", 0, "run/1"),
            "candidate-replay-report/schema=intel_candidate_replay_report_v1/dt=1970-01-01/hour=00/replay_run_id=run_1/report.json"
        );
        assert_eq!(
            replay_result_key("candidate-replay-result", 0, "run/1", "input/key.json"),
            format!(
                "candidate-replay-result/schema=intel_candidate_replay_result_v1/dt=1970-01-01/hour=00/replay_run_id=run_1/input_key_sha256={}/result.json",
                sha256_hex(b"input/key.json")
            )
        );
    }

    #[test]
    fn parse_replay_flags_and_worker_flags() {
        let args = parse_args(
            [
                "--nats-url",
                "nats://127.0.0.1:4222",
                "--input-s3-bucket",
                "test-structured-l1",
                "--output-s3-bucket",
                "test-candidate",
                "--market-l1-s3-bucket",
                "test-market-l1",
                "--replay-input-prefix",
                "structured-intel-packet/schema=structured_intel_packet_v1/",
                "--replay-max-keys-per-prefix",
                "7",
                "--replay-created-at-ms",
                "123",
                "--replay-result-prefix",
                "candidate-replay/custom",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect("args parse")
        .expect("args present");
        assert_eq!(
            args.input_prefixes,
            vec!["structured-intel-packet/schema=structured_intel_packet_v1/"]
        );
        assert_eq!(args.max_keys_per_prefix, 7);
        assert_eq!(args.created_at_ms, Some(123));
        assert_eq!(args.result_prefix, "candidate-replay/custom");
        assert_eq!(args.worker.nats.url, "nats://127.0.0.1:4222");
    }
}
