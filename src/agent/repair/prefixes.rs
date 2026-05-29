use super::super::args::AgentArgs;
use crate::time::time_part;
use std::collections::HashSet;

pub(in crate::agent::repair) const MILLIS_PER_DAY: i64 = 86_400_000;
const STRUCTURED_PACKET_REPAIR_PREFIX: &str =
    "structured-intel-packet/schema=structured_intel_packet_v1/";

pub(in crate::agent::repair) fn repair_prefixes_for_cycle(
    args: &AgentArgs,
    timestamp_ms: i64,
) -> Vec<String> {
    let mut prefixes = Vec::new();
    let mut seen = HashSet::new();

    for prefix in &args.repair_input_prefixes {
        if prefix == STRUCTURED_PACKET_REPAIR_PREFIX && args.repair_recent_partition_days > 0 {
            for offset_days in 0..args.repair_recent_partition_days {
                let offset_ms = i64::from(offset_days).saturating_mul(MILLIS_PER_DAY);
                let partition_timestamp_ms = timestamp_ms.saturating_sub(offset_ms);
                let part = time_part(partition_timestamp_ms);
                let recent_prefix = format!("{prefix}dt={}/", part.event_date);
                push_unique_prefix(&mut prefixes, &mut seen, recent_prefix);
            }
        }
    }

    for prefix in &args.repair_input_prefixes {
        push_unique_prefix(&mut prefixes, &mut seen, prefix.to_owned());
    }

    prefixes
}

fn push_unique_prefix(prefixes: &mut Vec<String>, seen: &mut HashSet<String>, prefix: String) {
    if seen.insert(prefix.clone()) {
        prefixes.push(prefix);
    }
}
