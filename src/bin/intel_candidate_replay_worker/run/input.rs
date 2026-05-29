use intel_candidate_app::error::AppResult;
use intel_candidate_app::telemetry;
use intel_candidate_app::worker::CandidateWorker;
use serde_json::json;

use super::super::args::ReplayArgs;

pub(super) async fn collect_replay_input_keys(
    args: &ReplayArgs,
    worker: &CandidateWorker,
    replay_run_id: &str,
) -> AppResult<Vec<String>> {
    let mut all_keys = Vec::new();
    for prefix in &args.input_prefixes {
        let keys = worker
            .list_replay_input_keys(prefix, args.max_keys_per_prefix)
            .await?;
        telemetry::info(
            "replay_prefix_scanned",
            json!({
                "replay_run_id": replay_run_id,
                "input_prefix": prefix,
                "keys": keys.len(),
            }),
        )?;
        all_keys.extend(keys);
    }
    all_keys.sort();
    all_keys.dedup();
    Ok(all_keys)
}
