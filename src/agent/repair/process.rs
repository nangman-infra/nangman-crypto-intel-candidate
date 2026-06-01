use super::report::RepairCycleReport;
use crate::error::AppResult;
use crate::live::log_error;
use crate::telemetry;
use crate::time::now_ms;
use crate::worker::CandidateWorker;
use serde_json::json;

pub(in crate::agent::repair) async fn process_repair_keys(
    agent_run_id: &str,
    worker: &CandidateWorker,
    all_keys: Vec<String>,
) -> AppResult<RepairCycleReport> {
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
