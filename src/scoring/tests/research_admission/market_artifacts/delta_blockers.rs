use super::*;

#[test]
fn missing_market_feature_delta_artifact_blocks_research() {
    let policy = policy();
    let universe = universe(true);
    let regime_contexts = vec![market_regime_context()];

    let result = process_packet_with_artifacts(
        packet(),
        &policy,
        market_artifacts(&universe, &[], &regime_contexts),
        7_200_000,
    );

    assert_blocked_with_reason(&result, "missing_market_feature_delta");
}

#[test]
fn market_feature_delta_known_after_candidate_time_blocks_research() {
    let policy = policy();
    let universe = universe(true);
    let mut future_delta = market_feature_delta("price");
    future_delta.known_as_of_ms = 7_200_001;
    let feature_deltas = vec![future_delta];
    let regime_contexts = vec![market_regime_context()];

    let result = process_packet_with_artifacts(
        packet(),
        &policy,
        market_artifacts(&universe, &feature_deltas, &regime_contexts),
        7_200_000,
    );

    assert_blocked_with_reason(&result, "missing_market_feature_delta");
}

#[test]
fn symbol_mismatched_market_feature_delta_blocks_research() {
    let policy = policy();
    let universe = universe(true);
    let mut mismatched_delta = market_feature_delta("price");
    mismatched_delta.symbol_canonical = "APT".to_owned();
    let feature_deltas = vec![mismatched_delta];
    let regime_contexts = vec![market_regime_context()];

    let result = process_packet_with_artifacts(
        packet(),
        &policy,
        market_artifacts(&universe, &feature_deltas, &regime_contexts),
        7_200_000,
    );

    assert_blocked_with_reason(&result, "missing_market_feature_delta");
}

#[test]
fn stale_market_feature_delta_blocks_research() {
    let policy = policy();
    let universe = universe(true);
    let mut stale_delta = market_feature_delta("price");
    stale_delta.quality_status = "stale".to_owned();
    let feature_deltas = vec![stale_delta];
    let regime_contexts = vec![market_regime_context()];

    let result = process_packet_with_artifacts(
        packet(),
        &policy,
        market_artifacts(&universe, &feature_deltas, &regime_contexts),
        7_200_000,
    );

    assert_blocked_with_reason(&result, "missing_market_feature_delta");
}
