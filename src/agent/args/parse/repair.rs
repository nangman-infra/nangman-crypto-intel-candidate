use super::super::types::{
    AgentArgs, DEFAULT_REPAIR_INTERVAL_SECS, DEFAULT_REPAIR_MAX_KEYS_PER_PREFIX,
    DEFAULT_REPAIR_MAX_PAGES_PER_PREFIX, DEFAULT_REPAIR_RECENT_PARTITION_DAYS,
};
use super::value::{next_value, parse_non_negative_u32, parse_positive_u64, parse_positive_usize};
use crate::error::{AppError, AppResult};
use crate::storage::validate_object_prefix;
use crate::worker::WorkerArgs;

pub(super) struct RepairArgs {
    input_prefixes: Vec<String>,
    interval_secs: u64,
    max_keys_per_prefix: usize,
    max_pages_per_prefix: usize,
    recent_partition_days: u32,
    enabled: bool,
}

impl RepairArgs {
    pub(super) fn apply_arg(
        &mut self,
        raw: &[String],
        index: &mut usize,
        worker_args: &mut Vec<String>,
    ) -> AppResult<()> {
        let flag = raw[*index].as_str();
        match flag {
            "--repair-input-prefix" | "--agent-repair-input-prefix" | "--replay-input-prefix" => {
                *index += 1;
                self.input_prefixes.push(next_value(raw, *index, flag)?);
            }
            "--repair-interval-secs" | "--agent-repair-interval-secs" => {
                *index += 1;
                self.interval_secs = parse_positive_u64(&next_value(raw, *index, flag)?)?;
            }
            "--repair-max-keys-per-prefix" | "--agent-repair-max-keys-per-prefix" => {
                *index += 1;
                self.max_keys_per_prefix = parse_positive_usize(&next_value(raw, *index, flag)?)?;
            }
            "--repair-max-pages-per-prefix" | "--agent-repair-max-pages-per-prefix" => {
                *index += 1;
                self.max_pages_per_prefix = parse_positive_usize(&next_value(raw, *index, flag)?)?;
            }
            "--repair-recent-partition-days" | "--agent-repair-recent-partition-days" => {
                *index += 1;
                self.recent_partition_days =
                    parse_non_negative_u32(&next_value(raw, *index, flag)?)?;
            }
            "--disable-repair" | "--agent-disable-repair" => {
                self.enabled = false;
            }
            value => worker_args.push(value.to_owned()),
        }
        Ok(())
    }

    pub(super) fn validate(&self) -> AppResult<()> {
        for prefix in &self.input_prefixes {
            validate_object_prefix(prefix, "--repair-input-prefix")
                .map_err(|error| AppError::config(error.to_string()))?;
        }
        Ok(())
    }

    pub(super) fn into_agent_args(self, worker: WorkerArgs) -> AgentArgs {
        AgentArgs {
            worker,
            repair_input_prefixes: self.input_prefixes,
            repair_interval_secs: self.interval_secs,
            repair_max_keys_per_prefix: self.max_keys_per_prefix,
            repair_max_pages_per_prefix: self.max_pages_per_prefix,
            repair_recent_partition_days: self.recent_partition_days,
            repair_enabled: self.enabled,
        }
    }
}

impl Default for RepairArgs {
    fn default() -> Self {
        Self {
            input_prefixes: Vec::new(),
            interval_secs: DEFAULT_REPAIR_INTERVAL_SECS,
            max_keys_per_prefix: DEFAULT_REPAIR_MAX_KEYS_PER_PREFIX,
            max_pages_per_prefix: DEFAULT_REPAIR_MAX_PAGES_PER_PREFIX,
            recent_partition_days: DEFAULT_REPAIR_RECENT_PARTITION_DAYS,
            enabled: true,
        }
    }
}
