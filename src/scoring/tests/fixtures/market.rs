use crate::model::{MarketFeatureDelta, MarketRegimeContext, SymbolUniverseSnapshot};
use crate::scoring::MarketArtifactInputs;

pub(in crate::scoring::tests) fn market_feature_delta(metric_name: &str) -> MarketFeatureDelta {
    MarketFeatureDelta {
        schema_version: "market_feature_delta_v1".to_owned(),
        feature_delta_id: format!("delta_{metric_name}"),
        l1_run_id: "l1_001".to_owned(),
        metric_name: metric_name.to_owned(),
        venue: "binance".to_owned(),
        symbol_native: "SUIUSDT".to_owned(),
        symbol_canonical: "SUI".to_owned(),
        market_type: "spot".to_owned(),
        value_now: 101.0,
        value_15m_ago: Some(100.0),
        value_1h_ago: Some(99.0),
        change_pct_15m: Some(1.0),
        change_pct_1h: Some(2.02),
        price_change_same_window: Some(2.02),
        volume_change_same_window: Some(10.0),
        oi_price_divergence: None,
        window_start_ms: 1_000,
        window_end_ms: 1_200,
        known_as_of_ms: 1_250,
        quality_status: "complete".to_owned(),
        missing_reasons: Vec::new(),
    }
}

pub(in crate::scoring::tests) fn market_regime_context() -> MarketRegimeContext {
    MarketRegimeContext {
        schema_version: "market_regime_context_v1".to_owned(),
        regime_context_id: "regime_001".to_owned(),
        l1_run_id: "l1_001".to_owned(),
        scope: "market_all_symbols".to_owned(),
        window_start_ms: 1_000,
        window_end_ms: 1_200,
        btc_return_same_window: Some(0.5),
        eth_return_same_window: Some(0.7),
        sector_return_same_window: Some(0.4),
        volatility_regime: "medium".to_owned(),
        correlation_to_btc: Some(0.8),
        known_as_of_ms: 1_250,
        quality_status: "complete".to_owned(),
        missing_reasons: Vec::new(),
    }
}

pub(in crate::scoring::tests) fn market_artifacts<'a>(
    universe: &'a SymbolUniverseSnapshot,
    feature_deltas: &'a [MarketFeatureDelta],
    regime_contexts: &'a [MarketRegimeContext],
) -> MarketArtifactInputs<'a> {
    MarketArtifactInputs {
        universe: Some(universe),
        market_feature_deltas: feature_deltas,
        market_regime_contexts: regime_contexts,
    }
}
