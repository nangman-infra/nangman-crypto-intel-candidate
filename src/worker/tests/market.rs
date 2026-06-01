use super::*;

#[test]
fn expands_market_feature_delta_summary_for_candidate_scoring() {
    let deltas = expand_market_feature_delta_summary(MarketFeatureDeltaSummary {
        schema_version: "market_feature_delta_summary_v1".to_owned(),
        feature_delta_summary_id: "summary_001".to_owned(),
        l1_run_id: "l1_001".to_owned(),
        detail_feature_delta_key: "market_feature_delta/run_id=l1_001/delta.json".to_owned(),
        window_start_ms: 1_000,
        window_end_ms: 2_000,
        known_as_of_ms: 2_100,
        detail_record_count: 900,
        summary_row_count: 1,
        rows: vec![crate::model::MarketFeatureDeltaSummaryRow {
            venue: "binance".to_owned(),
            symbol_native: "SUIUSDT".to_owned(),
            symbol_canonical: "SUI".to_owned(),
            market_type: "spot".to_owned(),
            window_start_ms: 1_000,
            window_end_ms: 2_000,
            known_as_of_ms: 2_100,
            quality_status: "complete".to_owned(),
            missing_reasons: Vec::new(),
            metrics: vec![crate::model::MarketFeatureDeltaSummaryMetric {
                metric_name: "price".to_owned(),
                value_now: 1.25,
                value_15m_ago: Some(1.2),
                value_1h_ago: Some(1.1),
                change_pct_15m: Some(4.16),
                change_pct_1h: Some(13.63),
                price_change_same_window: Some(13.63),
                volume_change_same_window: Some(5.0),
                oi_price_divergence: None,
                window_start_ms: 1_000,
                window_end_ms: 2_000,
                quality_status: "complete".to_owned(),
            }],
        }],
    });

    assert_eq!(deltas.len(), 1);
    assert_eq!(deltas[0].schema_version, "market_feature_delta_summary_v1");
    assert_eq!(deltas[0].l1_run_id, "l1_001");
    assert_eq!(deltas[0].symbol_canonical, "SUI");
    assert_eq!(deltas[0].metric_name, "price");
    assert_eq!(deltas[0].known_as_of_ms, 2_100);
    assert!(
        deltas[0]
            .feature_delta_id
            .starts_with("market_delta_summary_metric_")
    );
}

#[test]
fn summary_delta_without_decision_safe_metric_requires_detail_fallback() {
    let packet: StructuredIntelPacket = serde_json::from_value(json!({
        "packet_id": "packet_001",
        "cluster_id": "cluster_001",
        "decision_available_at_ms": 2_000,
        "normalized_symbols": ["SUIUSDT"]
    }))
    .expect("minimal packet uses serde defaults");
    let future_delta = MarketFeatureDelta {
        schema_version: "market_feature_delta_summary_v1".to_owned(),
        feature_delta_id: "future_delta".to_owned(),
        l1_run_id: "l1_001".to_owned(),
        metric_name: "price".to_owned(),
        venue: "binance".to_owned(),
        symbol_native: "SUIUSDT".to_owned(),
        symbol_canonical: "SUI".to_owned(),
        market_type: "spot".to_owned(),
        value_now: 1.2,
        value_15m_ago: Some(1.1),
        value_1h_ago: Some(1.0),
        change_pct_15m: Some(9.0),
        change_pct_1h: Some(20.0),
        price_change_same_window: Some(20.0),
        volume_change_same_window: Some(5.0),
        oi_price_divergence: None,
        window_start_ms: 2_500,
        window_end_ms: 3_000,
        known_as_of_ms: 3_000,
        quality_status: "complete".to_owned(),
        missing_reasons: Vec::new(),
    };
    let valid_delta = MarketFeatureDelta {
        window_start_ms: 1_500,
        window_end_ms: 1_900,
        known_as_of_ms: 1_950,
        ..future_delta.clone()
    };

    assert!(!market_feature_deltas_satisfy_packet(
        &packet,
        &[future_delta]
    ));
    assert!(market_feature_deltas_satisfy_packet(
        &packet,
        &[valid_delta]
    ));
}
