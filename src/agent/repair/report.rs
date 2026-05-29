use crate::error::AppResult;
use crate::telemetry;
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::agent) struct RepairCycleReport {
    pub(in crate::agent::repair) keys_seen: usize,
    pub(in crate::agent::repair) keys_processed: usize,
    pub(in crate::agent::repair) keys_skipped_stale_revision: usize,
    pub(in crate::agent::repair) keys_failed: usize,
}

pub(in crate::agent) fn log_repair_cycle_finished(
    agent_run_id: &str,
    repair_cycle_number: usize,
    report: &RepairCycleReport,
) -> AppResult<()> {
    telemetry::info(
        "agent_repair_cycle_finished",
        json!({
            "agent_run_id": agent_run_id,
            "repair_cycle_number": repair_cycle_number,
            "keys_seen": report.keys_seen,
            "keys_processed": report.keys_processed,
            "keys_skipped_stale_revision": report.keys_skipped_stale_revision,
            "keys_failed": report.keys_failed,
        }),
    )
}
