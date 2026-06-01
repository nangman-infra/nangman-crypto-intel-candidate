use super::super::args::AgentArgs;
use super::cursor::RepairScanCursors;
use super::prefixes::repair_prefixes_for_cycle;
use super::process::process_repair_keys;
use super::report::RepairCycleReport;
use super::scan::collect_repair_keys;
use crate::error::AppResult;
use crate::telemetry;
use crate::time::now_ms;
use crate::worker::CandidateWorker;
use serde_json::json;

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

    let all_keys = collect_repair_keys(
        agent_run_id,
        worker,
        args,
        repair_scan_cursors,
        &repair_input_prefixes,
    )
    .await?;
    process_repair_keys(agent_run_id, worker, all_keys).await
}

pub(in crate::agent) fn repair_due(args: &AgentArgs, next_repair_after_ms: i64) -> bool {
    args.repair_enabled
        && !args.repair_input_prefixes.is_empty()
        && now_ms() >= next_repair_after_ms
}
