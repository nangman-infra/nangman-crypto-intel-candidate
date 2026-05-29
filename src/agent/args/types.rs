use crate::worker::WorkerArgs;

pub(super) const DEFAULT_REPAIR_INTERVAL_SECS: u64 = 3_600;
pub(super) const DEFAULT_REPAIR_MAX_KEYS_PER_PREFIX: usize = 500;
pub(super) const DEFAULT_REPAIR_MAX_PAGES_PER_PREFIX: usize = 8;
pub(super) const DEFAULT_REPAIR_RECENT_PARTITION_DAYS: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentArgs {
    pub worker: WorkerArgs,
    pub repair_input_prefixes: Vec<String>,
    pub repair_interval_secs: u64,
    pub repair_max_keys_per_prefix: usize,
    pub repair_max_pages_per_prefix: usize,
    pub repair_recent_partition_days: u32,
    pub repair_enabled: bool,
}
