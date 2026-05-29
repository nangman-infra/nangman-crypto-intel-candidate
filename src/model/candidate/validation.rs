use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ValidationRequirements {
    pub required_adapters: Vec<String>,
    pub optional_adapters: Vec<String>,
    pub min_unseen_windows: usize,
    pub include_fee: bool,
    pub include_slippage: bool,
    pub include_latency_assumption: bool,
    pub include_liquidity_filter: bool,
    pub required_train_validation_split: bool,
    pub max_adapter_runtime_minutes: usize,
}
