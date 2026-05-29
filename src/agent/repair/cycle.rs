use super::super::args::AgentArgs;
use super::cursor::RepairScanCursors;
use super::prefixes::repair_prefixes_for_cycle;
use super::report::RepairCycleReport;
use crate::error::AppResult;
use crate::live::log_error;
use crate::telemetry;
use crate::time::now_ms;
use crate::worker::CandidateWorker;
use serde_json::json;
use std::collections::HashSet;

pub(in crate::agent) async fn run_repair_cycle(
    agent_run_id: &str,
    worker: &CandidateWorker,
    args: &AgentArgs,
    repair_scan_cursors: &mut RepairScanCursors,
) -> AppResult<RepairCycleReport> {
    let repair_started_at_ms = now_ms();
    let repair_input_prefixes = repair_prefixes_for_cycle(args, repair_started_at_ms);
    telemetry::info(
        "agent_repair_cycle_started",
        json!({
            "agent_run_id": agent_run_id,
            "configured_repair_input_prefixes": &args.repair_input_prefixes,
            "repair_input_prefixes": &repair_input_prefixes,
            "repair_max_keys_per_prefix": args.repair_max_keys_per_prefix,
            "repair_max_pages_per_prefix": args.repair_max_pages_per_prefix,
            "repair_recent_partition_days": args.repair_recent_partition_days,
        }),
    )?;

    let mut all_keys = Vec::new();
    let mut seen_keys = HashSet::new();
    for prefix in &repair_input_prefixes {
        let keys =
            collect_repair_keys_for_prefix(agent_run_id, worker, args, repair_scan_cursors, prefix)
                .await?;
        append_unique_keys(&mut all_keys, &mut seen_keys, keys);
    }

    let mut report = RepairCycleReport {
        keys_seen: all_keys.len(),
        keys_processed: 0,
        keys_skipped_stale_revision: 0,
        keys_failed: 0,
    };
    for key in all_keys {
        match worker.process_s3_key(&key, now_ms()).await {
            Ok(Some(result)) => {
                report.keys_processed += 1;
                telemetry::info(
                    "agent_repair_key_processed",
                    json!({
                        "agent_run_id": agent_run_id,
                        "input_key": key,
                        "screening_event_id": result.screening_event.screening_event_id,
                        "candidate_id": result.screening_event.candidate_id,
                        "candidate_class": result.screening_event.candidate_class.as_policy_key(),
                        "research_eligible": result.screening_event.research_eligible,
                    }),
                )?;
            }
            Ok(None) => {
                report.keys_skipped_stale_revision += 1;
                telemetry::info(
                    "agent_repair_key_skipped_as_stale_revision",
                    json!({
                        "agent_run_id": agent_run_id,
                        "input_key": key,
                    }),
                )?;
            }
            Err(error) => {
                report.keys_failed += 1;
                log_error("agent_repair_key_failed", Some(agent_run_id), &error)?;
            }
        }
    }
    Ok(report)
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

pub(in crate::agent) fn repair_due(args: &AgentArgs, next_repair_after_ms: i64) -> bool {
    args.repair_enabled
        && !args.repair_input_prefixes.is_empty()
        && now_ms() >= next_repair_after_ms
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
