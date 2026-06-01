use super::*;

#[test]
fn revision_packet_carries_supersede_metadata() {
    let policy = policy();
    let universe = universe(true);
    let feature_deltas = vec![market_feature_delta("price")];
    let regime_contexts = vec![market_regime_context()];
    let mut input = packet();
    input.packet_id = "packet_002".to_owned();
    input.revision = 1;
    input.supersedes_packet_id = Some("packet_001".to_owned());
    let result = process_packet_with_artifacts(
        input,
        &policy,
        market_artifacts(&universe, &feature_deltas, &regime_contexts),
        7_200_000,
    );

    assert_eq!(
        result.screening_event.supersedes_packet_id.as_deref(),
        Some("packet_001")
    );
    assert_eq!(result.screening_event.input_packet_revision, 1);
    assert!(
        result
            .screening_event
            .supersedes_screening_event_id
            .as_deref()
            .is_some_and(|value| value.starts_with("cand_screen_"))
    );
    let bundle = result.evidence_bundle.expect("bundle should exist");
    assert_eq!(bundle.supersedes_packet_id.as_deref(), Some("packet_001"));
    assert_eq!(bundle.input_packet_revision, 1);
}
