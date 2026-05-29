use intel_candidate_app::error::AppResult;
use intel_candidate_app::storage::ObjectStore;

use super::super::args::ReplayArgs;
use super::super::keys::{checksum_json, replay_report_key};
use super::super::model::{REPORT_SCHEMA_VERSION, ReplayFailure, ReplayReport};
use super::counters::ReplayCounters;

pub(super) async fn write_replay_report(
    args: &ReplayArgs,
    output_store: &ObjectStore,
    created_at_ms: i64,
    replay_run_id: &str,
    keys_seen: usize,
    counters: ReplayCounters,
    failed_keys: Vec<ReplayFailure>,
) -> AppResult<ReplayReport> {
    let report_key = replay_report_key(&args.report_prefix, created_at_ms, replay_run_id);
    let mut report = ReplayReport {
        schema_version: REPORT_SCHEMA_VERSION.to_owned(),
        replay_run_id: replay_run_id.to_owned(),
        producer_app: "intel-candidate-app".to_owned(),
        producer_version: env!("CARGO_PKG_VERSION").to_owned(),
        created_at_ms,
        input_bucket: args.worker.input_store.bucket.clone(),
        output_bucket: args.worker.output_store.bucket.clone(),
        input_prefixes: args.input_prefixes.clone(),
        keys_seen,
        keys_processed: counters.keys_processed,
        keys_skipped_stale_revision: 0,
        keys_failed: failed_keys.len(),
        result_records_created: counters.result_records_created,
        evidence_bundles_created: counters.evidence_bundles_created,
        hypothesis_states_created: counters.hypothesis_states_created,
        screening_events_created: counters.screening_events_created,
        failed_keys,
        result_prefix: args.result_prefix.clone(),
        report_key: report_key.clone(),
        checksum: String::new(),
    };
    report.checksum = checksum_json(&report)?;
    output_store
        .put_bytes_idempotent(
            &report_key,
            serde_json::to_vec_pretty(&report)?,
            "application/json",
        )
        .await?;
    Ok(report)
}
