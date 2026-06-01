use serde_json::{Value, json};

pub(in crate::cli::tests) fn universe_json() -> Value {
    json!({
        "schema_version": "symbol_universe_snapshot_v1",
        "symbol_universe_snapshot_id": "universe_cli_001",
        "universe_as_of_ms": 1000,
        "included_symbols": [{
            "symbol_canonical": "SUI",
            "execution_symbol_native": "SUIUSDT",
            "reference_symbol_native": "SUIUSDT",
            "liquidity_rank_at_that_time": 1,
            "approved_universe_symbol": true,
            "bootstrap_days_available": 30,
            "median_spread_bps_30d": 2.0,
            "median_traded_notional_30d": 10000000.0,
            "gap_rate_30d": 0.0,
            "mapping_confidence": "strong",
            "status_reason": "approved"
        }],
        "excluded_symbols": [],
        "liquidity_rank_at_that_time": [],
        "selection_policy_version": "symbol_universe_policy_v1",
        "venue_truth_policy_version": "venue_truth_policy_v1",
        "data_quality_cutoff_version": "data_quality_cutoff_v1",
        "generated_at_ms": 1000
    })
}
