use super::types::{
    AgentArgs, DEFAULT_REPAIR_INTERVAL_SECS, DEFAULT_REPAIR_MAX_KEYS_PER_PREFIX,
    DEFAULT_REPAIR_MAX_PAGES_PER_PREFIX, DEFAULT_REPAIR_RECENT_PARTITION_DAYS,
};
use crate::error::{AppError, AppResult};
use crate::worker::WorkerArgs;

impl AgentArgs {
    pub fn parse(values: impl Iterator<Item = String>) -> AppResult<Option<Self>> {
        let raw = values.collect::<Vec<_>>();
        if raw.iter().any(|value| value == "-h" || value == "--help") {
            return Ok(None);
        }

        let mut repair_input_prefixes = Vec::new();
        let mut repair_interval_secs = DEFAULT_REPAIR_INTERVAL_SECS;
        let mut repair_max_keys_per_prefix = DEFAULT_REPAIR_MAX_KEYS_PER_PREFIX;
        let mut repair_max_pages_per_prefix = DEFAULT_REPAIR_MAX_PAGES_PER_PREFIX;
        let mut repair_recent_partition_days = DEFAULT_REPAIR_RECENT_PARTITION_DAYS;
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
                "--repair-max-pages-per-prefix" | "--agent-repair-max-pages-per-prefix" => {
                    index += 1;
                    repair_max_pages_per_prefix =
                        parse_positive_usize(&next_value(&raw, index, raw[index - 1].as_str())?)?;
                }
                "--repair-recent-partition-days" | "--agent-repair-recent-partition-days" => {
                    index += 1;
                    repair_recent_partition_days =
                        parse_non_negative_u32(&next_value(&raw, index, raw[index - 1].as_str())?)?;
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
            repair_max_pages_per_prefix,
            repair_recent_partition_days,
            repair_enabled,
        }))
    }
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

fn parse_non_negative_u32(value: &str) -> AppResult<u32> {
    value
        .parse::<u32>()
        .map_err(|error| AppError::config(format!("invalid non-negative integer {value}: {error}")))
}
