use intel_candidate_app::error::AppResult;
use intel_candidate_app::time::{path_segment, time_part};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::model::{REPORT_SCHEMA_VERSION, RESULT_SCHEMA_VERSION};

pub(super) fn replay_report_key(prefix: &str, created_at_ms: i64, replay_run_id: &str) -> String {
    let part = time_part(created_at_ms);
    format!(
        "{}/schema={}/dt={}/hour={:02}/replay_run_id={}/report.json",
        prefix.trim_matches('/'),
        REPORT_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(replay_run_id)
    )
}

pub(super) fn replay_result_key(
    prefix: &str,
    created_at_ms: i64,
    replay_run_id: &str,
    input_key: &str,
) -> String {
    let part = time_part(created_at_ms);
    format!(
        "{}/schema={}/dt={}/hour={:02}/replay_run_id={}/input_key_sha256={}/result.json",
        prefix.trim_matches('/'),
        RESULT_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(replay_run_id),
        sha256_hex(input_key.as_bytes())
    )
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(super) fn checksum_json<T: Serialize>(value: &T) -> AppResult<String> {
    let bytes = serde_json::to_vec(value)?;
    Ok(sha256_hex(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_key_is_partitioned() {
        assert_eq!(
            replay_report_key("candidate-replay-report", 0, "run/1"),
            "candidate-replay-report/schema=intel_candidate_replay_report_v1/dt=1970-01-01/hour=00/replay_run_id=run_1/report.json"
        );
        assert_eq!(
            replay_result_key("candidate-replay-result", 0, "run/1", "input/key.json"),
            format!(
                "candidate-replay-result/schema=intel_candidate_replay_result_v1/dt=1970-01-01/hour=00/replay_run_id=run_1/input_key_sha256={}/result.json",
                sha256_hex(b"input/key.json")
            )
        );
    }
}
