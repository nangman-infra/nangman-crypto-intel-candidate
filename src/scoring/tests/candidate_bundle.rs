use super::*;
use crate::hash::sha256_hex;
use crate::model::CandidateClass;

#[test]
fn creates_strong_candidate_when_p0_contract_passes() {
    let policy = policy();
    let universe = universe(true);
    let feature_deltas = vec![market_feature_delta("price")];
    let regime_contexts = vec![market_regime_context()];
    let result = process_packet_with_artifacts(
        packet(),
        &policy,
        market_artifacts(&universe, &feature_deltas, &regime_contexts),
        7_200_000,
    );

    assert_eq!(
        result.screening_event.candidate_class,
        CandidateClass::StrongCandidate
    );
    assert!(result.evidence_bundle.is_some());
    let bundle = result.evidence_bundle.unwrap();
    assert_eq!(bundle.event_time_ms, 1_300);
    assert_eq!(bundle.candidate_created_at_ms, 7_200_000);
    assert_eq!(bundle.decision_available_at_ms, 7_200_000);
    assert_eq!(bundle.forbidden_lookahead_boundary_ms, 7_200_000);
    assert_eq!(bundle.symbol_universe_snapshot_id, "universe_001");
    assert!(bundle.approved_universe_symbol);
    assert_eq!(bundle.research_priority, "p0_event_risk");
    assert!(bundle.storage_uri.starts_with(
        "candidate-evidence-bundle/priority=p0/schema=intel_candidate_evidence_bundle_v1/"
    ));
    assert_eq!(bundle.selected_market_artifacts.len(), 2);
    assert!(bundle.selected_market_artifacts.iter().any(|artifact| {
        artifact.artifact_type == "market_feature_delta_summary"
            && artifact.artifact_id == "delta_price"
            && artifact.artifact_key.as_deref() == Some("market-feature-delta-summary/summary.json")
            && artifact.known_as_of_ms == 1_250
    }));
    assert!(bundle.selected_market_artifacts.iter().any(|artifact| {
        artifact.artifact_type == "market_regime_context"
            && artifact.artifact_id == "regime_001"
            && artifact.artifact_key.as_deref() == Some("market-regime/context.json")
            && artifact.known_as_of_ms == 1_250
    }));
    assert_eq!(
        result.screening_event.input_packet_family_id,
        "packet_family_001"
    );
    assert_eq!(result.screening_event.input_packet_revision, 0);
    assert_eq!(bundle.input_packet_family_id, "packet_family_001");
    assert_eq!(bundle.input_packet_revision, 0);
    assert_eq!(bundle.hypothesis_type, "risk_incident_watch");
    assert_eq!(
        bundle.allowed_horizons,
        vec!["4h".to_owned(), "24h".to_owned(), "72h".to_owned()]
    );
    assert_eq!(bundle.data_quality_summary.status, "present");
    assert_eq!(
        bundle
            .data_quality_summary
            .market_data_quality_summary_key
            .as_deref(),
        Some("quality/summary.json")
    );
    assert_eq!(
        bundle.confidence_summary.get("symbol_confidence_band"),
        Some(&"strong".to_owned())
    );
    assert_eq!(
        bundle.confidence_summary.get("packet_confidence_band"),
        Some(&"strong".to_owned())
    );
    assert_eq!(
        bundle.confidence_summary.get("market_context_status"),
        Some(&"available_symbol_context".to_owned())
    );
    let mut checksum_payload = bundle.clone();
    checksum_payload.checksum.clear();
    assert_eq!(
        bundle.checksum,
        sha256_hex(serde_json::to_vec(&checksum_payload).unwrap())
    );
}
