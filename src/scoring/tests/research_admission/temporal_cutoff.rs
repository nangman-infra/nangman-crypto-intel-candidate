use super::*;

#[test]
fn rehydrated_market_artifacts_use_candidate_time_as_admission_cutoff() {
    let policy = policy();
    let universe = universe(true);
    let mut feature_delta = market_feature_delta("price");
    feature_delta.window_end_ms = 1_900;
    feature_delta.known_as_of_ms = 2_000;
    let mut regime_context = market_regime_context();
    regime_context.window_end_ms = 1_900;
    regime_context.known_as_of_ms = 2_100;

    let result = process_packet_with_artifacts(
        packet(),
        &policy,
        market_artifacts(&universe, &[feature_delta], &[regime_context]),
        7_200_000,
    );

    assert!(result.screening_event.research_eligible);
    let bundle = result.evidence_bundle.expect("bundle should exist");
    assert_eq!(bundle.decision_available_at_ms, 7_200_000);
    assert!(bundle.selected_market_artifacts.iter().any(|artifact| {
        artifact.artifact_type == "market_feature_delta_summary" && artifact.known_as_of_ms == 2_000
    }));
    assert!(bundle.selected_market_artifacts.iter().any(|artifact| {
        artifact.artifact_type == "market_regime_context" && artifact.known_as_of_ms == 2_100
    }));
}

#[test]
fn future_market_artifacts_after_candidate_time_still_block_research() {
    let policy = policy();
    let universe = universe(true);
    let mut feature_delta = market_feature_delta("price");
    feature_delta.known_as_of_ms = 7_200_001;
    let mut regime_context = market_regime_context();
    regime_context.known_as_of_ms = 7_200_001;

    let result = process_packet_with_artifacts(
        packet(),
        &policy,
        market_artifacts(&universe, &[feature_delta], &[regime_context]),
        7_200_000,
    );

    assert_blocked_with_reason(&result, "missing_market_feature_delta");
    assert_blocked_with_reason(&result, "missing_market_regime_context");
}

#[test]
fn missing_decision_available_time_blocks_research_bundle() {
    let policy = policy();
    let universe = universe(true);
    let feature_deltas = vec![market_feature_delta("price")];
    let regime_contexts = vec![market_regime_context()];
    let mut input = packet();
    input.decision_available_at_ms = None;

    let result = process_packet_with_artifacts(
        input,
        &policy,
        market_artifacts(&universe, &feature_deltas, &regime_contexts),
        7_200_000,
    );

    assert!(!result.screening_event.research_eligible);
    assert!(result.evidence_bundle.is_none());
    assert!(
        result
            .screening_event
            .reasons
            .iter()
            .any(|reason| reason == "missing_replay_time_contract")
    );
}
