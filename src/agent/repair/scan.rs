use super::super::args::AgentArgs;
use super::cursor::RepairScanCursors;
use super::process::process_repair_keys;
use super::report::RepairCycleReport;
use crate::error::AppResult;
use crate::telemetry;
use crate::worker::CandidateWorker;
use serde_json::json;
use std::collections::HashSet;

pub(in crate::agent::repair) async fn run_repair_scan(
    agent_run_id: &str,
    worker: &CandidateWorker,
    args: &AgentArgs,
    repair_scan_cursors: &mut RepairScanCursors,
    repair_input_prefixes: &[String],
) -> AppResult<RepairCycleReport> {
    let mut report = RepairCycleReport::empty();
    let mut seen_keys = HashSet::new();
    for prefix in repair_input_prefixes {
        let prefix_report = run_repair_scan_for_prefix(
            agent_run_id,
            worker,
            args,
            repair_scan_cursors,
            prefix,
            &mut seen_keys,
        )
        .await?;
        report.merge(prefix_report);
    }
    Ok(report)
}

async fn run_repair_scan_for_prefix(
    agent_run_id: &str,
    worker: &CandidateWorker,
    args: &AgentArgs,
    repair_scan_cursors: &mut RepairScanCursors,
    prefix: &str,
    seen_keys: &mut HashSet<String>,
) -> AppResult<RepairCycleReport> {
    let mut report = RepairCycleReport::empty();
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
        let unique_page_keys = take_unique_keys(seen_keys, page.keys);
        let unique_page_key_count = unique_page_keys.len();
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
                "unique_keys": unique_page_key_count,
            }),
        )?;
        let page_report = process_repair_keys(agent_run_id, worker, unique_page_keys).await?;
        report.merge(page_report);
        if !cursor_continues || page_key_count == 0 {
            break;
        }
    }
    Ok(report)
}

pub(in crate::agent::repair) fn take_unique_keys(
    seen: &mut HashSet<String>,
    keys: Vec<String>,
) -> Vec<String> {
    let mut unique_keys = Vec::new();
    for key in keys {
        if seen.insert(key.clone()) {
            unique_keys.push(key);
        }
    }
    unique_keys
}
