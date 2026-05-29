use super::*;
use crate::model::{
    MarketContextRef, MarketFeatureDelta, MarketRegimeContext, MetricEvidence,
    SourceIndependenceSummary, SymbolResolutionTrace, SymbolUniverseMember, TextEvidence,
};
use crate::policy::load_policy;
use std::path::Path;

fn policy() -> ScoringPolicy {
    load_policy(&Path::new(env!("CARGO_MANIFEST_DIR")).join("policies/scoring-policy.v1.json"))
        .expect("default scoring policy loads")
}

fn universe(approved: bool) -> SymbolUniverseSnapshot {
    SymbolUniverseSnapshot {
        schema_version: "symbol_universe_snapshot_v1".to_owned(),
        symbol_universe_snapshot_id: "universe_001".to_owned(),
        universe_as_of_ms: 1_000,
        included_symbols: vec![SymbolUniverseMember {
            symbol_canonical: "SUI".to_owned(),
            execution_symbol_native: Some("SUIUSDT".to_owned()),
            reference_symbol_native: Some("SUIUSDT".to_owned()),
            liquidity_rank_at_that_time: Some(1),
            approved_universe_symbol: approved,
            bootstrap_days_available: 30,
            median_spread_bps_30d: Some(2.0),
            median_traded_notional_30d: Some(10_000_000.0),
            gap_rate_30d: Some(0.0),
            mapping_confidence: "strong".to_owned(),
            status_reason: if approved {
                "approved".to_owned()
            } else {
                "insufficient_30d_bootstrap".to_owned()
            },
        }],
        excluded_symbols: Vec::new(),
        liquidity_rank_at_that_time: Vec::new(),
        selection_policy_version: "symbol_universe_policy_v1".to_owned(),
        venue_truth_policy_version: "venue_truth_policy_v1".to_owned(),
        data_quality_cutoff_version: "data_quality_cutoff_v1".to_owned(),
        generated_at_ms: 1_000,
    }
}

fn packet() -> StructuredIntelPacket {
    StructuredIntelPacket {
        packet_id: "packet_001".to_owned(),
        packet_family_id: "packet_family_001".to_owned(),
        raw_event_id: "raw_event_001".to_owned(),
        event_timestamp_ms: 1_000,
        revision: 0,
        supersedes_packet_id: None,
        cluster_id: "cluster_001".to_owned(),
        source_event_ids: vec!["source_001".to_owned()],
        published_at_ms: Some(1_000),
        fetched_at_ms: Some(1_100),
        structured_at_ms: Some(1_200),
        decision_available_at_ms: Some(1_300),
        normalized_symbols: vec!["SUI".to_owned()],
        symbol_confidence_band: ConfidenceBand::Strong,
        symbol_resolution_trace: vec![SymbolResolutionTrace {
            raw_mentions: vec!["SUI".to_owned()],
            resolved_project: Some("Sui".to_owned()),
            resolved_asset: Some("SUI".to_owned()),
            canonical_symbol: Some("SUI".to_owned()),
            venue_symbols: vec!["SUIUSDT".to_owned()],
            mapping_confidence: ConfidenceBand::Strong,
            ambiguity_reason: None,
        }],
        event_type: EventType::Incident,
        topic_summary: "official incident notice".to_owned(),
        stance_summary: "risk watch".to_owned(),
        risk_summary: "operational risk".to_owned(),
        regime_hint: "neutral".to_owned(),
        scenario_hint: "observe response".to_owned(),
        confidence_band: ConfidenceBand::High,
        novelty_score: 0.9,
        time_relevance_window: None,
        contradiction_flags: Vec::new(),
        source_quality_summary: "official_notice".to_owned(),
        source_independence_summary: Some(SourceIndependenceSummary {
            source_event_count: 1,
            independent_source_count: 1,
            official_source_present: true,
            duplicate_content_hashes: Vec::new(),
            syndicated_from: None,
            original_source_ids: vec!["official".to_owned()],
        }),
        text_evidence: vec![
            TextEvidence {
                evidence_text: "Official incident notice was published.".to_owned(),
                source_event_id: "source_001".to_owned(),
                source_id: "official".to_owned(),
                published_at_ms: Some(1_000),
                evidence_kind: "source_sentence".to_owned(),
            },
            TextEvidence {
                evidence_text: "The notice names SUI directly.".to_owned(),
                source_event_id: "source_001".to_owned(),
                source_id: "official".to_owned(),
                published_at_ms: Some(1_000),
                evidence_kind: "source_sentence".to_owned(),
            },
        ],
        metric_evidence: Vec::new(),
        evidence_quality_reasons: Vec::new(),
        market_context_status: MarketContextStatus::AvailableSymbolContext,
        market_context_retry_after_ms: None,
        market_context_expire_at_ms: None,
        market_context_terminal_reason: None,
        market_context_ref: Some(MarketContextRef {
            status: MarketContextStatus::AvailableSymbolContext,
            basis_timestamp_ms: Some(1_300),
            basis_kind: "exact".to_owned(),
            window_start_ms: Some(1_000),
            window_end_ms: Some(2_000),
            manifest_key: Some("normalized-market-slice/manifest.json".to_owned()),
            output_object_keys: vec!["normalized-market-slice/part.jsonl".to_owned()],
            market_data_quality_summary_key: Some("quality/summary.json".to_owned()),
            market_feature_delta_key: Some("market-feature-delta/delta.json".to_owned()),
            market_feature_delta_summary_key: Some(
                "market-feature-delta-summary/summary.json".to_owned(),
            ),
            market_regime_context_key: Some("market-regime/context.json".to_owned()),
            symbol_universe_snapshot_key: Some("universe/snapshot.json".to_owned()),
        }),
        model_tier_used: "haiku".to_owned(),
        terminal_decision: "structured_only".to_owned(),
        evidence_sentences: Vec::new(),
        schema_version: Some(STRUCTURED_PACKET_SCHEMA_VERSION.to_owned()),
    }
}

fn market_feature_delta(metric_name: &str) -> MarketFeatureDelta {
    MarketFeatureDelta {
        schema_version: "market_feature_delta_v1".to_owned(),
        feature_delta_id: format!("delta_{metric_name}"),
        l1_run_id: "l1_001".to_owned(),
        metric_name: metric_name.to_owned(),
        venue: "binance".to_owned(),
        symbol_native: "SUIUSDT".to_owned(),
        symbol_canonical: "SUI".to_owned(),
        market_type: "spot".to_owned(),
        value_now: 101.0,
        value_15m_ago: Some(100.0),
        value_1h_ago: Some(99.0),
        change_pct_15m: Some(1.0),
        change_pct_1h: Some(2.02),
        price_change_same_window: Some(2.02),
        volume_change_same_window: Some(10.0),
        oi_price_divergence: None,
        window_start_ms: 1_000,
        window_end_ms: 1_200,
        known_as_of_ms: 1_250,
        quality_status: "complete".to_owned(),
        missing_reasons: Vec::new(),
    }
}

fn market_regime_context() -> MarketRegimeContext {
    MarketRegimeContext {
        schema_version: "market_regime_context_v1".to_owned(),
        regime_context_id: "regime_001".to_owned(),
        l1_run_id: "l1_001".to_owned(),
        scope: "market_all_symbols".to_owned(),
        window_start_ms: 1_000,
        window_end_ms: 1_200,
        btc_return_same_window: Some(0.5),
        eth_return_same_window: Some(0.7),
        sector_return_same_window: Some(0.4),
        volatility_regime: "medium".to_owned(),
        correlation_to_btc: Some(0.8),
        known_as_of_ms: 1_250,
        quality_status: "complete".to_owned(),
        missing_reasons: Vec::new(),
    }
}

fn market_artifacts<'a>(
    universe: &'a SymbolUniverseSnapshot,
    feature_deltas: &'a [MarketFeatureDelta],
    regime_contexts: &'a [MarketRegimeContext],
) -> MarketArtifactInputs<'a> {
    MarketArtifactInputs {
        universe: Some(universe),
        market_feature_deltas: feature_deltas,
        market_regime_contexts: regime_contexts,
    }
}

fn assert_blocked_with_reason(result: &CandidateProcessingResult, expected_reason: &str) {
    assert!(!result.screening_event.research_eligible);
    assert!(result.evidence_bundle.is_none());
    assert!(
        result
            .screening_event
            .reasons
            .iter()
            .any(|reason| reason == expected_reason),
        "expected reason {expected_reason}, got {:?}",
        result.screening_event.reasons
    );
}

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
}

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

#[test]
fn candidate_bundle_key_uses_coarse_priority_partition() {
    assert!(
        candidate_bundle_key(7_200_000, "cand_001", "p0").starts_with(
            "candidate-evidence-bundle/priority=p0/schema=intel_candidate_evidence_bundle_v1/"
        )
    );
    assert_eq!(research_priority_partition("p0_event_risk"), "p0");
    assert_eq!(research_priority_partition("p1_high_confidence"), "p1");
    assert_eq!(research_priority_partition("p2_standard"), "p2");
    assert_eq!(research_priority_partition("unknown"), "p2");
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
