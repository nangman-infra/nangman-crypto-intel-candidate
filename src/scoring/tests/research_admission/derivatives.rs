use super::*;

#[test]
fn derivatives_market_feature_delta_can_satisfy_derivatives_delta_gate() {
    let policy = policy();
    let universe = universe(true);
    let feature_deltas = vec![market_feature_delta("open_interest")];
    let regime_contexts = vec![market_regime_context()];
    let mut input = packet();
    input.event_type = EventType::FundingShift;
    input.metric_evidence = vec![MetricEvidence {
        metric_name: "open_interest".to_owned(),
        symbol: Some("SUIUSDT".to_owned()),
        venue: Some("binance_usdm".to_owned()),
        value: Some(98_000_000.0),
        previous_value: None,
        delta_pct: None,
        window_ms: Some(3_600_000),
        observed_at_ms: 1_250,
        source_event_id: "source_001".to_owned(),
    }];

    let result = process_packet_with_artifacts(
        input,
        &policy,
        market_artifacts(&universe, &feature_deltas, &regime_contexts),
        7_200_000,
    );

    assert!(result.screening_event.research_eligible);
    assert!(result.evidence_bundle.is_some());
}

#[test]
fn official_derivatives_delta_resolves_single_snapshot_penalties() {
    let policy = policy();
    let universe = universe(true);
    let feature_deltas = vec![market_feature_delta("open_interest")];
    let regime_contexts = vec![market_regime_context()];
    let mut input = packet();
    input.event_type = EventType::FundingShift;
    input.symbol_confidence_band = ConfidenceBand::Moderate;
    input.symbol_resolution_trace[0].mapping_confidence = ConfidenceBand::Moderate;
    input.confidence_band = ConfidenceBand::Low;
    input.novelty_score = 0.58;
    input.contradiction_flags = vec![ContradictionFlag::EvidenceWeak];
    input.evidence_quality_reasons = vec![
        EvidenceQualityReason::BaselineMissing,
        EvidenceQualityReason::SingleNumericSnapshot,
        EvidenceQualityReason::SingleSourceOnly,
    ];
    input.metric_evidence = vec![MetricEvidence {
        metric_name: "open_interest_snapshot".to_owned(),
        symbol: Some("SUIUSDT".to_owned()),
        venue: Some("binance_usdm".to_owned()),
        value: Some(98_000_000.0),
        previous_value: None,
        delta_pct: None,
        window_ms: None,
        observed_at_ms: 1_250,
        source_event_id: "source_001".to_owned(),
    }];

    let result = process_packet_with_artifacts(
        input,
        &policy,
        market_artifacts(&universe, &feature_deltas, &regime_contexts),
        7_200_000,
    );

    assert_eq!(
        result.screening_event.candidate_class,
        CandidateClass::ResearchCandidate
    );
    assert!(result.screening_event.research_eligible);
    let bundle = result
        .evidence_bundle
        .as_ref()
        .expect("bundle should exist");
    assert!(bundle.selected_market_artifacts.iter().any(|artifact| {
        artifact.artifact_type == "market_feature_delta_summary"
            && artifact.metric_name.as_deref() == Some("open_interest")
    }));
    assert!(
        !result
            .screening_event
            .score_breakdown
            .components
            .iter()
            .any(|component| matches!(
                component.name.as_str(),
                "baseline_missing"
                    | "single_numeric_snapshot"
                    | "single_source_only"
                    | "legacy_evidence_weak_penalty"
            ))
    );
}

#[test]
fn derivatives_without_delta_stays_out_of_research() {
    let policy = policy();
    let universe = universe(true);
    let feature_deltas = vec![market_feature_delta("price")];
    let regime_contexts = vec![market_regime_context()];
    let mut input = packet();
    input.event_type = EventType::FundingShift;
    input.metric_evidence = vec![MetricEvidence {
        metric_name: "open_interest".to_owned(),
        symbol: Some("SUIUSDT".to_owned()),
        venue: Some("binance_usdm".to_owned()),
        value: Some(98_000_000.0),
        previous_value: None,
        delta_pct: None,
        window_ms: Some(3_600_000),
        observed_at_ms: 1_250,
        source_event_id: "source_001".to_owned(),
    }];

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
            .any(|reason| reason == "derivatives_metric_delta_missing")
    );
}
