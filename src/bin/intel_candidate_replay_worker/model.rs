use intel_candidate_app::model::CandidateProcessingResult;
use serde::Serialize;

pub(super) const REPORT_SCHEMA_VERSION: &str = "intel_candidate_replay_report_v1";
pub(super) const RESULT_SCHEMA_VERSION: &str = "intel_candidate_replay_result_v1";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(super) struct ReplayReport {
    pub(super) schema_version: String,
    pub(super) replay_run_id: String,
    pub(super) producer_app: String,
    pub(super) producer_version: String,
    pub(super) created_at_ms: i64,
    pub(super) input_bucket: String,
    pub(super) output_bucket: String,
    pub(super) input_prefixes: Vec<String>,
    pub(super) keys_seen: usize,
    pub(super) keys_processed: usize,
    pub(super) keys_skipped_stale_revision: usize,
    pub(super) keys_failed: usize,
    pub(super) result_records_created: usize,
    pub(super) evidence_bundles_created: usize,
    pub(super) hypothesis_states_created: usize,
    pub(super) screening_events_created: usize,
    pub(super) failed_keys: Vec<ReplayFailure>,
    pub(super) result_prefix: String,
    pub(super) report_key: String,
    pub(super) checksum: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(super) struct ReplayResultRecord {
    pub(super) schema_version: String,
    pub(super) replay_run_id: String,
    pub(super) created_at_ms: i64,
    pub(super) input_bucket: String,
    pub(super) input_key: String,
    pub(super) input_key_sha256: String,
    pub(super) result: CandidateProcessingResult,
    pub(super) checksum: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(super) struct ReplayFailure {
    pub(super) input_key: String,
    pub(super) error: String,
}
