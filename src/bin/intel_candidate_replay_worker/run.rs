use intel_candidate_app::error::{AppError, AppResult};
use intel_candidate_app::storage::ObjectStore;
use intel_candidate_app::time::now_ms;
use intel_candidate_app::worker::CandidateWorker;

use super::args::{ReplayArgs, parse_args, replay_help};
use super::model::ReplayFailure;
use counters::ReplayCounters;
use input::collect_replay_input_keys;
use logging::{
    log_replay_finished, log_replay_key_failed, log_replay_key_processed, log_replay_started,
};
use process::process_replay_key;
use report::write_replay_report;

#[path = "run/counters.rs"]
mod counters;
#[path = "run/input.rs"]
mod input;
#[path = "run/logging.rs"]
mod logging;
#[path = "run/process.rs"]
mod process;
#[path = "run/report.rs"]
mod report;

pub(super) async fn run() -> AppResult<()> {
    let Some(args) = parse_args(std::env::args().skip(1))? else {
        println!("{}", replay_help());
        return Ok(());
    };
    run_with_args(args).await
}

async fn run_with_args(args: ReplayArgs) -> AppResult<()> {
    let created_at_ms = args.created_at_ms.unwrap_or_else(now_ms);
    let replay_run_id = format!("intel-candidate-replay-{}", created_at_ms);
    log_replay_started(&args, &replay_run_id)?;

    let output_store = ObjectStore::connect(args.worker.output_store.clone()).await?;
    let worker = CandidateWorker::connect(&args.worker).await?;
    let all_keys = collect_replay_input_keys(&args, &worker, &replay_run_id).await?;
    let mut counters = ReplayCounters::default();
    let mut failed_keys = Vec::new();

    for key in &all_keys {
        let replay_result = process_replay_key(
            &args,
            &worker,
            &output_store,
            created_at_ms,
            &replay_run_id,
            key,
        )
        .await;
        match replay_result {
            Ok((result, result_key)) => {
                counters.record_success(&result);
                log_replay_key_processed(&args, &replay_run_id, key, &result, &result_key)?;
            }
            Err(error) => {
                failed_keys.push(ReplayFailure {
                    input_key: key.clone(),
                    error: error.to_string(),
                });
                log_replay_key_failed(&replay_run_id, key, &error)?;
            }
        }
    }

    let report = write_replay_report(
        &args,
        &output_store,
        created_at_ms,
        &replay_run_id,
        all_keys.len(),
        counters,
        failed_keys,
    )
    .await?;
    log_replay_finished(&replay_run_id, &report)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if args.fail_on_record_error && report.keys_failed > 0 {
        return Err(AppError::validation(format!(
            "candidate replay finished with {} failed keys; report_key={}",
            report.keys_failed, report.report_key
        )));
    }
    Ok(())
}
