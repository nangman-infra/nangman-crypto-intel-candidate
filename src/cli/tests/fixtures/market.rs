use serde_json::{Value, json};

pub(in crate::cli::tests) fn market_feature_delta_json() -> Value {
    json!([{
        "schema_version": "market_feature_delta_v1",
        "feature_delta_id": "delta_cli_price",
        "l1_run_id": "l1_cli_001",
        "metric_name": "price",
        "venue": "binance",
        "symbol_native": "SUIUSDT",
        "symbol_canonical": "SUI",
        "market_type": "spot",
        "value_now": 101.0,
        "value_15m_ago": 100.0,
        "value_1h_ago": 99.0,
        "change_pct_15m": 1.0,
        "change_pct_1h": 2.02,
        "price_change_same_window": 2.02,
        "volume_change_same_window": 10.0,
        "window_start_ms": 1000,
        "window_end_ms": 1200,
        "known_as_of_ms": 1250,
        "quality_status": "complete",
        "missing_reasons": []
    }])
}

pub(in crate::cli::tests) fn market_regime_context_json() -> Value {
    json!([{
        "schema_version": "market_regime_context_v1",
        "regime_context_id": "regime_cli_001",
        "l1_run_id": "l1_cli_001",
        "scope": "market_all_symbols",
        "window_start_ms": 1000,
        "window_end_ms": 1200,
        "btc_return_same_window": 0.5,
        "eth_return_same_window": 0.7,
        "sector_return_same_window": 0.4,
        "volatility_regime": "medium",
        "correlation_to_btc": 0.8,
        "known_as_of_ms": 1250,
        "quality_status": "complete",
        "missing_reasons": []
    }])
}
