use super::*;

#[test]
fn unapproved_universe_blocks_research_bundle() {
    let policy = policy();
    let universe = universe(false);
    let feature_deltas = vec![market_feature_delta("price")];
    let regime_contexts = vec![market_regime_context()];

    let result = process_packet_with_artifacts(
        packet(),
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
            .any(|reason| reason == "not_admitted_universe")
    );
}
