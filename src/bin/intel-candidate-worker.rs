use intel_candidate_app::error::AppResult;
use intel_candidate_app::live::{log_error, run_live_worker};
use intel_candidate_app::worker::{WorkerArgs, worker_help};
use std::process;

#[tokio::main]
async fn main() -> AppResult<()> {
    if let Err(error) = run().await {
        log_error("worker_fatal", None, &error)?;
        process::exit(1);
    }
    Ok(())
}

async fn run() -> AppResult<()> {
    let Some(args) = WorkerArgs::parse(std::env::args().skip(1))? else {
        println!("{}", worker_help());
        return Ok(());
    };
    run_live_worker(args).await?;
    Ok(())
}
