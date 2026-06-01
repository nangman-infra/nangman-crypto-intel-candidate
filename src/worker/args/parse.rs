mod arg;
mod finalize;

use super::WorkerArgs;
use crate::error::AppResult;
use arg::{WorkerArgAction, apply_worker_arg};
use finalize::finalize_worker_args;

pub(super) fn parse_worker_args(
    values: impl Iterator<Item = String>,
) -> AppResult<Option<WorkerArgs>> {
    parse_worker_args_with_env(values, || std::env::var("NATS_URL").ok())
}

pub(super) fn parse_worker_args_with_env(
    mut values: impl Iterator<Item = String>,
    nats_url_env: impl FnOnce() -> Option<String>,
) -> AppResult<Option<WorkerArgs>> {
    let mut args = WorkerArgs::default();
    while let Some(arg) = values.next() {
        if matches!(
            apply_worker_arg(arg, &mut values, &mut args)?,
            WorkerArgAction::Help
        ) {
            return Ok(None);
        }
    }
    finalize_worker_args(args, nats_url_env).map(Some)
}
