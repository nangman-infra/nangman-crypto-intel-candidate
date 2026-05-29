use serde::{Deserialize, Serialize};

use super::super::quality::MarketContextStatus;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MarketContextRef {
    pub status: MarketContextStatus,
    #[serde(default)]
    pub basis_timestamp_ms: Option<i64>,
    #[serde(default)]
    pub basis_kind: String,
    #[serde(default)]
    pub window_start_ms: Option<i64>,
    #[serde(default)]
    pub window_end_ms: Option<i64>,
    #[serde(default)]
    pub manifest_key: Option<String>,
    #[serde(default)]
    pub output_object_keys: Vec<String>,
    #[serde(default)]
    pub market_data_quality_summary_key: Option<String>,
    #[serde(default)]
    pub market_feature_delta_key: Option<String>,
    #[serde(default)]
    pub market_feature_delta_summary_key: Option<String>,
    #[serde(default)]
    pub market_regime_context_key: Option<String>,
    #[serde(default)]
    pub symbol_universe_snapshot_key: Option<String>,
}
