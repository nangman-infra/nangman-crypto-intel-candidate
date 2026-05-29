use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MarketFeatureDelta {
    pub schema_version: String,
    pub feature_delta_id: String,
    pub l1_run_id: String,
    pub metric_name: String,
    pub venue: String,
    pub symbol_native: String,
    pub symbol_canonical: String,
    pub market_type: String,
    pub value_now: f64,
    #[serde(default)]
    pub value_15m_ago: Option<f64>,
    #[serde(default)]
    pub value_1h_ago: Option<f64>,
    #[serde(default)]
    pub change_pct_15m: Option<f64>,
    #[serde(default)]
    pub change_pct_1h: Option<f64>,
    #[serde(default)]
    pub price_change_same_window: Option<f64>,
    #[serde(default)]
    pub volume_change_same_window: Option<f64>,
    #[serde(default)]
    pub oi_price_divergence: Option<f64>,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub known_as_of_ms: i64,
    #[serde(default)]
    pub quality_status: String,
    #[serde(default)]
    pub missing_reasons: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MarketFeatureDeltaSummary {
    pub schema_version: String,
    pub feature_delta_summary_id: String,
    pub l1_run_id: String,
    pub detail_feature_delta_key: String,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub known_as_of_ms: i64,
    pub detail_record_count: usize,
    pub summary_row_count: usize,
    #[serde(default)]
    pub rows: Vec<MarketFeatureDeltaSummaryRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MarketFeatureDeltaSummaryRow {
    pub venue: String,
    pub symbol_native: String,
    pub symbol_canonical: String,
    pub market_type: String,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub known_as_of_ms: i64,
    #[serde(default)]
    pub quality_status: String,
    #[serde(default)]
    pub missing_reasons: Vec<String>,
    #[serde(default)]
    pub metrics: Vec<MarketFeatureDeltaSummaryMetric>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MarketFeatureDeltaSummaryMetric {
    pub metric_name: String,
    pub value_now: f64,
    #[serde(default)]
    pub value_15m_ago: Option<f64>,
    #[serde(default)]
    pub value_1h_ago: Option<f64>,
    #[serde(default)]
    pub change_pct_15m: Option<f64>,
    #[serde(default)]
    pub change_pct_1h: Option<f64>,
    #[serde(default)]
    pub price_change_same_window: Option<f64>,
    #[serde(default)]
    pub volume_change_same_window: Option<f64>,
    #[serde(default)]
    pub oi_price_divergence: Option<f64>,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    #[serde(default)]
    pub quality_status: String,
}
