use super::super::args::AgentArgs;
use super::cursor::RepairScanCursors;
use crate::error::AppResult;
use crate::telemetry;
use crate::worker::CandidateWorker;
use serde_json::json;
use std::collections::HashSet;

pub(in crate::agent::repair) async fn collect_repair_keys(
    agent_run_id: &str,
    worker: &CandidateWorker,
    args: &AgentArgs,
    repair_scan_cursors: &mut RepairScanCursors,
    repair_input_prefixes: &[String],
) -> AppResult<Vec<String>> {
    let mut all_keys = Vec::new();
    let mut seen_keys = HashSet::new();
    for prefix in repair_input_prefixes {
        let keys =
            collect_repair_keys_for_prefix(agent_run_id, worker, args, repair_scan_cursors, prefix)
                .await?;
        append_unique_keys(&mut all_keys, &mut seen_keys, keys);
    }
    Ok(all_keys)
}

async fn collect_repair_keys_for_prefix(
    agent_run_id: &str,
    worker: &CandidateWorker,
    args: &AgentArgs,
    repair_scan_cursors: &mut RepairScanCursors,
    prefix: &str,
) -> AppResult<Vec<String>> {
    let mut keys = Vec::new();
    for page_number in 1..=args.repair_max_pages_per_prefix {
        let scan_start_after = repair_scan_cursors.start_after(prefix).map(str::to_owned);
        let page = worker
            .list_replay_input_key_page(
                prefix,
                args.repair_max_keys_per_prefix,
                scan_start_after.as_deref(),
            )
            .await?;
        let page_key_count = page.keys.len();
        let cursor_continues =
            repair_scan_cursors.update_after_page(prefix, page.next_start_after.clone());
        telemetry::info(
            "agent_repair_prefix_scanned",
            json!({
                "agent_run_id": agent_run_id,
                "input_prefix": prefix,
                "repair_page_number": page_number,
                "repair_max_pages_per_prefix": args.repair_max_pages_per_prefix,
                "scan_start_after": scan_start_after,
                "next_start_after": page.next_start_after,
                "cursor_continues": cursor_continues,
                "keys": page_key_count,
            }),
        )?;
        keys.extend(page.keys);
        if !cursor_continues || page_key_count == 0 {
            break;
        }
    }
    Ok(keys)
}

pub(in crate::agent::repair) fn append_unique_keys(
    destination: &mut Vec<String>,
    seen: &mut HashSet<String>,
    keys: Vec<String>,
) {
    for key in keys {
        if seen.insert(key.clone()) {
            destination.push(key);
        }
    }
}
