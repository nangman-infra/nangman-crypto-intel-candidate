use crate::model::{SymbolUniverseMember, SymbolUniverseSnapshot};

pub(in crate::scoring::tests) fn universe(approved: bool) -> SymbolUniverseSnapshot {
    SymbolUniverseSnapshot {
        schema_version: "symbol_universe_snapshot_v1".to_owned(),
        symbol_universe_snapshot_id: "universe_001".to_owned(),
        universe_as_of_ms: 1_000,
        included_symbols: vec![SymbolUniverseMember {
            symbol_canonical: "SUI".to_owned(),
            execution_symbol_native: Some("SUIUSDT".to_owned()),
            reference_symbol_native: Some("SUIUSDT".to_owned()),
            liquidity_rank_at_that_time: Some(1),
            approved_universe_symbol: approved,
            bootstrap_days_available: 30,
            median_spread_bps_30d: Some(2.0),
            median_traded_notional_30d: Some(10_000_000.0),
            gap_rate_30d: Some(0.0),
            mapping_confidence: "strong".to_owned(),
            status_reason: if approved {
                "approved".to_owned()
            } else {
                "insufficient_30d_bootstrap".to_owned()
            },
        }],
        excluded_symbols: Vec::new(),
        liquidity_rank_at_that_time: Vec::new(),
        selection_policy_version: "symbol_universe_policy_v1".to_owned(),
        venue_truth_policy_version: "venue_truth_policy_v1".to_owned(),
        data_quality_cutoff_version: "data_quality_cutoff_v1".to_owned(),
        generated_at_ms: 1_000,
    }
}
