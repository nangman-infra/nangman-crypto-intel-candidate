use super::*;
use crate::model::CandidateClass;

#[test]
fn forbidden_generated_output_quarantines_packet() {
    let policy = policy();
    let universe = universe(true);
    let feature_deltas = vec![market_feature_delta("price")];
    let regime_contexts = vec![market_regime_context()];
    let mut input = packet();
    input.scenario_hint = "buy immediately".to_owned();

    let result = process_packet_with_artifacts(
        input,
        &policy,
        market_artifacts(&universe, &feature_deltas, &regime_contexts),
        7_200_000,
    );

    assert_eq!(
        result.screening_event.candidate_class,
        CandidateClass::Quarantine
    );
    assert!(result.evidence_bundle.is_none());
}
