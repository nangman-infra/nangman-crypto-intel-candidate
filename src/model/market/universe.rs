use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SymbolUniverseSnapshot {
    pub schema_version: String,
    pub symbol_universe_snapshot_id: String,
    pub universe_as_of_ms: i64,
    #[serde(default)]
    pub included_symbols: Vec<SymbolUniverseMember>,
    #[serde(default)]
    pub excluded_symbols: Vec<SymbolUniverseMember>,
    #[serde(default)]
    pub liquidity_rank_at_that_time: Vec<SymbolLiquidityRank>,
    pub selection_policy_version: String,
    pub venue_truth_policy_version: String,
    pub data_quality_cutoff_version: String,
    pub generated_at_ms: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SymbolUniverseMember {
    pub symbol_canonical: String,
    #[serde(default)]
    pub execution_symbol_native: Option<String>,
    #[serde(default)]
    pub reference_symbol_native: Option<String>,
    #[serde(default)]
    pub liquidity_rank_at_that_time: Option<i64>,
    pub approved_universe_symbol: bool,
    pub bootstrap_days_available: i64,
    #[serde(default)]
    pub median_spread_bps_30d: Option<f64>,
    #[serde(default)]
    pub median_traded_notional_30d: Option<f64>,
    #[serde(default)]
    pub gap_rate_30d: Option<f64>,
    pub mapping_confidence: String,
    pub status_reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SymbolLiquidityRank {
    pub symbol_canonical: String,
    pub liquidity_rank_at_that_time: i64,
    pub observed_traded_notional: f64,
}
