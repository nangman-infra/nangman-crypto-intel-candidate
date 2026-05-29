use intel_candidate_app::error::AppResult;
use intel_candidate_app::model::CandidateProcessingResult;
use intel_candidate_app::telemetry;
use serde_json::json;
use std::error::Error;

use super::super::args::ReplayArgs;
use super::super::model::ReplayReport;

pub(super) fn log_replay_started(args: &ReplayArgs, replay_run_id: &str) -> AppResult<()> {
    telemetry::info(
        "replay_started",
        json!({
            "replay_run_id": replay_run_id,
            "input_bucket": args.worker.input_store.bucket.as_str(),
            "output_bucket": args.worker.output_store.bucket.as_str(),
            "input_prefixes": &args.input_prefixes,
            "max_keys_per_prefix": args.max_keys_per_prefix,
        }),
    )
}

pub(super) fn log_replay_key_processed(
    args: &ReplayArgs,
    replay_run_id: &str,
    key: &str,
    result: &CandidateProcessingResult,
    result_key: &str,
) -> AppResult<()> {
    telemetry::info(
        "replay_key_processed",
        json!({
            "replay_run_id": replay_run_id,
            "input_key": key,
            "screening_event_id": result.screening_event.screening_event_id,
            "candidate_class": result.screening_event.candidate_class.as_policy_key(),
            "research_eligible": result.screening_event.research_eligible,
            "replay_result_s3_key": result_key,
            "replay_artifacts_written": args.write_artifacts,
            "evidence_bundle_created": result.evidence_bundle.is_some(),
            "hypothesis_state_created": result.hypothesis_state.is_some(),
        }),
    )
}

pub(super) fn log_replay_key_failed(
    replay_run_id: &str,
    key: &str,
    error: &dyn Error,
) -> AppResult<()> {
    telemetry::error(
        "replay_key_failed",
        json!({
            "replay_run_id": replay_run_id,
            "input_key": key,
            "error": error.to_string(),
        }),
    )
}

pub(super) fn log_replay_finished(replay_run_id: &str, report: &ReplayReport) -> AppResult<()> {
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
    )
}
