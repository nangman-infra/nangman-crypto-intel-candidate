use super::{NatsConfig, ObjectStoreConfig};
use crate::error::AppResult;
use std::path::PathBuf;

mod defaults;
mod parse;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerArgs {
    pub nats: NatsConfig,
    pub input_store: ObjectStoreConfig,
    pub output_store: ObjectStoreConfig,
    pub market_store: ObjectStoreConfig,
    pub policy_file: PathBuf,
    pub max_messages: Option<usize>,
    pub exit_on_idle: bool,
}

impl WorkerArgs {
    pub fn parse(values: impl Iterator<Item = String>) -> AppResult<Option<Self>> {
        parse::parse_worker_args(values)
    }
}

impl Default for WorkerArgs {
    fn default() -> Self {
        defaults::default_worker_args()
    }
}
