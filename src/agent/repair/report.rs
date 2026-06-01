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

impl RepairCycleReport {
    pub(in crate::agent::repair) fn empty() -> Self {
        Self {
            keys_seen: 0,
            keys_processed: 0,
            keys_skipped_stale_revision: 0,
            keys_failed: 0,
        }
    }

    pub(in crate::agent::repair) fn merge(&mut self, report: Self) {
        self.keys_seen += report.keys_seen;
        self.keys_processed += report.keys_processed;
        self.keys_skipped_stale_revision += report.keys_skipped_stale_revision;
        self.keys_failed += report.keys_failed;
    }
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
