use super::*;

#[test]
fn missing_regime_sector_return_blocks_research() {
    let policy = policy();
    let universe = universe(true);
    let feature_deltas = vec![market_feature_delta("price")];
    let mut regime_context = market_regime_context();
    regime_context.sector_return_same_window = None;
    let regime_contexts = vec![regime_context];

    let result = process_packet_with_artifacts(
        packet(),
        &policy,
        market_artifacts(&universe, &feature_deltas, &regime_contexts),
        7_200_000,
    );

    assert_blocked_with_reason(&result, "missing_market_regime_context");
}
