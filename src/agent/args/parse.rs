mod repair;
mod value;

use super::types::AgentArgs;
use crate::error::AppResult;
use crate::worker::WorkerArgs;
use repair::RepairArgs;

impl AgentArgs {
    pub fn parse(values: impl Iterator<Item = String>) -> AppResult<Option<Self>> {
        let raw = values.collect::<Vec<_>>();
        if raw.iter().any(|value| value == "-h" || value == "--help") {
            return Ok(None);
        }

        let mut repair = RepairArgs::default();
        let mut worker_args = Vec::new();
        let mut index = 0usize;

        while index < raw.len() {
            repair.apply_arg(&raw, &mut index, &mut worker_args)?;
            index += 1;
        }

        repair.validate()?;
        let Some(worker) = WorkerArgs::parse(worker_args.into_iter())? else {
            return Ok(None);
        };
        Ok(Some(repair.into_agent_args(worker)))
    }
}
