use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MarketRegimeContext {
    pub schema_version: String,
    pub regime_context_id: String,
    pub l1_run_id: String,
    pub scope: String,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    #[serde(default)]
    pub btc_return_same_window: Option<f64>,
    #[serde(default)]
    pub eth_return_same_window: Option<f64>,
    #[serde(default)]
    pub sector_return_same_window: Option<f64>,
    pub volatility_regime: String,
    #[serde(default)]
    pub correlation_to_btc: Option<f64>,
    pub known_as_of_ms: i64,
    #[serde(default)]
    pub quality_status: String,
    #[serde(default)]
    pub missing_reasons: Vec<String>,
}
