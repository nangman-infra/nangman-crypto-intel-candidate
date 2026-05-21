use intel_candidate_app::agent::{AgentArgs, agent_help, run_agent};
use intel_candidate_app::error::AppResult;
use intel_candidate_app::live::log_error;
use std::process;

#[tokio::main]
async fn main() -> AppResult<()> {
    if let Err(error) = run().await {
        log_error("agent_fatal", None, &error)?;
        process::exit(1);
    }
    Ok(())
}

async fn run() -> AppResult<()> {
    let Some(args) = AgentArgs::parse(std::env::args().skip(1))? else {
        println!("{}", agent_help());
        return Ok(());
    };
    run_agent(args).await?;
    Ok(())
}
