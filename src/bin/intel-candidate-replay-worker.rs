#[path = "intel_candidate_replay_worker/args.rs"]
mod args;
#[path = "intel_candidate_replay_worker/keys.rs"]
mod keys;
#[path = "intel_candidate_replay_worker/model.rs"]
mod model;
#[path = "intel_candidate_replay_worker/run.rs"]
mod run;

use intel_candidate_app::error::AppResult;
use intel_candidate_app::telemetry;
use serde_json::json;
use std::process;

#[tokio::main]
async fn main() -> AppResult<()> {
    if let Err(error) = run::run().await {
        telemetry::error("replay_worker_fatal", json!({ "error": error.to_string() }))?;
        process::exit(1);
    }
    Ok(())
}
