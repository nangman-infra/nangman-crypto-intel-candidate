use intel_candidate_app::worker::WorkerArgs;

pub(super) const DEFAULT_MAX_KEYS: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReplayArgs {
    pub(crate) worker: WorkerArgs,
    pub(crate) input_prefixes: Vec<String>,
    pub(crate) max_keys_per_prefix: usize,
    pub(crate) created_at_ms: Option<i64>,
    pub(crate) report_prefix: String,
    pub(crate) result_prefix: String,
    pub(crate) fail_on_record_error: bool,
    pub(crate) write_artifacts: bool,
}
