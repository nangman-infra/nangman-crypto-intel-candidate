use super::*;

#[test]
fn blocked_research_candidate_is_preserved_as_hypothesis_state() {
    let policy = policy();
    let universe = universe(true);
    let regime_contexts = vec![market_regime_context()];

    let result = process_packet_with_artifacts(
        packet(),
        &policy,
        market_artifacts(&universe, &[], &regime_contexts),
        7_200_000,
    );

    assert!(!result.screening_event.research_eligible);
    assert!(result.evidence_bundle.is_none());
    let state = result
        .hypothesis_state
        .expect("non-research candidate should remain rerunnable");
    assert_eq!(state.current_state, result.screening_event.candidate_class);
    assert_eq!(
        state.latest_screening_event_id,
        result.screening_event.screening_event_id
    );
    assert_eq!(state.hypothesis_type, "risk_incident_watch");
    assert!(
        state
            .state_key
            .starts_with("hypothesis-state/schema=intel_candidate_hypothesis_state_v1/")
    );
    assert!(
        state
            .retryable_reasons
            .iter()
            .any(|reason| reason == "missing_market_feature_delta")
    );
    assert_eq!(state.next_action, "rerun_when_market_feature_delta_updates");
    assert!(
        state
            .dirty_triggers
            .iter()
            .any(|trigger| trigger == "candidate_app_version_changed")
    );
}
