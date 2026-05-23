use crate::hash::{sha256_hex, stable_id};
use crate::model::{
    CANDIDATE_BUNDLE_SCHEMA_VERSION, CandidateClass, CandidateProcessingResult, ConfidenceBand,
    ContradictionFlag, DataQualitySummaryRef, EventType, HYPOTHESIS_STATE_SCHEMA_VERSION,
    IntelCandidateEvidenceBundle, IntelCandidateHypothesisState, IntelCandidateScreeningEvent,
    MarketContextRef, MarketContextStatus, MarketFeatureDelta, MarketRegimeContext, PRODUCER_APP,
    SCREENING_EVENT_SCHEMA_VERSION, STRUCTURED_PACKET_SCHEMA_VERSION, ScoreBreakdown,
    ScoreComponent, SelectedMarketArtifactTrace, SourceIndependenceSummary, StructuredIntelPacket,
    SymbolUniverseSnapshot, ValidationRequirements,
};
use crate::policy::{ScoringPolicy, ValidationRequirementDefaults};
use crate::time::{hour_bucket_ms, path_segment, time_part};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdmissionState {
    has_valid_schema: bool,
    has_required_times: bool,
    has_valid_time_order: bool,
    has_evidence: bool,
    has_lineage: bool,
    has_symbols: bool,
    has_source_independence: bool,
    source_independence_ok_for_research: bool,
    source_independence_ok_for_strong: bool,
    has_symbol_resolution_trace: bool,
    symbol_resolution_ok_for_research: bool,
    symbol_resolution_ok_for_strong: bool,
    has_data_quality_summary: bool,
    has_market_feature_delta: bool,
    has_market_regime_context: bool,
    selected_market_feature_delta: Option<SelectedMarketArtifactTrace>,
    selected_market_regime_context: Option<SelectedMarketArtifactTrace>,
    has_point_in_time_universe: bool,
    approved_universe_symbol: bool,
    has_derivatives_metric_delta: bool,
    market_context_allows_research: bool,
    market_context_allows_strong: bool,
    stale_market_context: bool,
    social_only: bool,
    has_medium_or_high_contradiction: bool,
    forbidden_output_terms: Vec<String>,
    quarantine_reasons: Vec<String>,
    reject_reasons: Vec<String>,
    observe_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdmissionSignals {
    has_valid_schema: bool,
    has_required_times: bool,
    has_valid_time_order: bool,
    has_evidence: bool,
    has_lineage: bool,
    has_symbols: bool,
    has_source_independence: bool,
    source_independence_ok_for_research: bool,
    source_independence_ok_for_strong: bool,
    has_symbol_resolution_trace: bool,
    symbol_resolution_ok_for_research: bool,
    symbol_resolution_ok_for_strong: bool,
    has_data_quality_summary: bool,
    has_market_feature_delta: bool,
    has_market_regime_context: bool,
    selected_market_feature_delta: Option<SelectedMarketArtifactTrace>,
    selected_market_regime_context: Option<SelectedMarketArtifactTrace>,
    has_point_in_time_universe: bool,
    approved_universe_symbol: bool,
    has_derivatives_metric_delta: bool,
    market_context_allows_research: bool,
    market_context_allows_strong: bool,
    stale_market_context: bool,
    social_only: bool,
    has_medium_or_high_contradiction: bool,
    forbidden_output_terms: Vec<String>,
}

impl AdmissionState {
    fn research_block_reasons(&self, packet: &StructuredIntelPacket) -> Vec<String> {
        let mut reasons = Vec::new();
        if !self.has_required_times {
            reasons.push("missing_replay_time_contract".to_owned());
        }
        if !self.has_valid_time_order {
            reasons.push("invalid_replay_time_order".to_owned());
        }
        if !self.has_point_in_time_universe {
            reasons.push("missing_point_in_time_universe".to_owned());
        }
        if !self.approved_universe_symbol {
            reasons.push("not_admitted_universe".to_owned());
        }
        if !self.has_data_quality_summary {
            reasons.push("missing_data_quality_summary".to_owned());
        }
        if !self.has_market_feature_delta {
            reasons.push("missing_market_feature_delta".to_owned());
        }
        if !self.has_market_regime_context {
            reasons.push("missing_market_regime_context".to_owned());
        }
        if !self.has_source_independence {
            reasons.push("missing_source_independence".to_owned());
        } else if !self.source_independence_ok_for_research {
            reasons.push("insufficient_source_independence".to_owned());
        }
        if !self.has_symbol_resolution_trace {
            reasons.push("missing_symbol_resolution_trace".to_owned());
        } else if !self.symbol_resolution_ok_for_research {
            reasons.push("weak_symbol_resolution".to_owned());
        }
        if packet.event_type.is_derivatives_like() && !self.has_derivatives_metric_delta {
            reasons.push("derivatives_metric_delta_missing".to_owned());
        }
        if !self.market_context_allows_research {
            reasons.push("market_context_not_research_admissible".to_owned());
        }
        reasons
    }

    fn strong_block_reasons(&self) -> Vec<String> {
        let mut reasons = self.reject_reasons.clone();
        if !self.source_independence_ok_for_strong {
            reasons.push("strong_requires_source_independence_or_official_source".to_owned());
        }
        if !self.symbol_resolution_ok_for_strong {
            reasons.push("strong_requires_moderate_or_strong_symbol_resolution".to_owned());
        }
        if !self.market_context_allows_strong {
            reasons.push("strong_requires_available_symbol_context".to_owned());
        }
        if self.has_medium_or_high_contradiction {
            reasons.push("strong_blocked_by_contradiction".to_owned());
        }
        if self.social_only {
            reasons.push("strong_blocked_by_social_only".to_owned());
        }
        reasons
    }
}

pub fn process_packet(
    packet: StructuredIntelPacket,
    policy: &ScoringPolicy,
    universe: Option<&SymbolUniverseSnapshot>,
    created_at_ms: i64,
) -> CandidateProcessingResult {
    process_packet_with_artifacts(
        packet,
        policy,
        MarketArtifactInputs {
            universe,
            market_feature_deltas: &[],
            market_regime_contexts: &[],
        },
        created_at_ms,
    )
}

#[derive(Debug, Clone, Copy)]
pub struct MarketArtifactInputs<'a> {
    pub universe: Option<&'a SymbolUniverseSnapshot>,
    pub market_feature_deltas: &'a [MarketFeatureDelta],
    pub market_regime_contexts: &'a [MarketRegimeContext],
}

pub fn process_packet_with_artifacts(
    packet: StructuredIntelPacket,
    policy: &ScoringPolicy,
    market_artifacts: MarketArtifactInputs<'_>,
    created_at_ms: i64,
) -> CandidateProcessingResult {
    let universe = market_artifacts.universe;
    let admission = evaluate_admission(&packet, policy, market_artifacts, created_at_ms);
    let score = calculate_score(&packet, policy, universe, &admission);
    let mut reasons = Vec::new();
    reasons.extend(admission.quarantine_reasons.clone());
    reasons.extend(admission.reject_reasons.clone());
    reasons.extend(admission.observe_reasons.clone());

    let candidate_id = if admission.has_valid_schema && admission.has_symbols {
        Some(candidate_id(&packet, policy, universe))
    } else {
        None
    };

    let class = classify_candidate(&packet, policy, &admission, &score, &mut reasons);
    let research_eligible = class.is_research_eligible();
    let selected_market_artifacts = admission.selected_market_artifacts();
    let idempotency_key = stable_id(
        "cand_idem",
        &[
            packet.packet_id.as_str(),
            policy.policy_version.as_str(),
            class.as_policy_key(),
            candidate_id.as_deref().unwrap_or("no_candidate_id"),
        ],
    );

    let evidence_bundle = if research_eligible {
        candidate_id.as_ref().map(|id| {
            build_evidence_bundle(BundleBuildContext {
                candidate_id: id,
                packet: &packet,
                policy,
                universe: universe
                    .expect("research-eligible candidates require a universe snapshot"),
                created_at_ms,
                candidate_class: class.clone(),
                score_breakdown: score.clone(),
                reasons: reasons.clone(),
                selected_market_artifacts: selected_market_artifacts.clone(),
                idempotency_key: idempotency_key.clone(),
            })
        })
    } else {
        None
    };
    let hypothesis_state = if !research_eligible
        && !matches!(class, CandidateClass::Quarantine)
        && candidate_id.is_some()
    {
        Some(build_hypothesis_state(HypothesisStateBuildContext {
            hypothesis_id: candidate_id
                .as_deref()
                .expect("candidate_id is present for hypothesis_state"),
            packet: &packet,
            policy,
            created_at_ms,
            candidate_class: class.clone(),
            score_breakdown: score.clone(),
            reasons: reasons.clone(),
            selected_market_artifacts,
            idempotency_key: idempotency_key.clone(),
            screening_event_id: stable_id(
                "cand_screen",
                &[packet.packet_id.as_str(), policy.policy_version.as_str()],
            ),
        }))
    } else {
        None
    };

    let screening_event = IntelCandidateScreeningEvent {
        screening_event_id: stable_id(
            "cand_screen",
            &[packet.packet_id.as_str(), policy.policy_version.as_str()],
        ),
        schema_version: SCREENING_EVENT_SCHEMA_VERSION.to_owned(),
        producer_app: PRODUCER_APP.to_owned(),
        created_at_ms,
        input_packet_id: packet.packet_id.clone(),
        input_packet_family_id: effective_packet_family_id(&packet).to_owned(),
        input_packet_revision: packet.revision,
        source_structured_packet_ids: vec![packet.packet_id.clone()],
        supersedes_packet_id: packet.supersedes_packet_id.clone(),
        supersedes_screening_event_id: packet.supersedes_packet_id.as_deref().map(|packet_id| {
            stable_id("cand_screen", &[packet_id, policy.policy_version.as_str()])
        }),
        scoring_policy_version: policy.policy_version.clone(),
        candidate_class: class.clone(),
        candidate_score: score.final_score,
        score_breakdown: score,
        research_eligible,
        quarantine: matches!(class, CandidateClass::Quarantine),
        reasons,
        candidate_id: if research_eligible {
            candidate_id
        } else {
            None
        },
        idempotency_key,
    };

    CandidateProcessingResult {
        screening_event,
        evidence_bundle,
        hypothesis_state,
    }
}

fn evaluate_admission(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    market_artifacts: MarketArtifactInputs<'_>,
    candidate_created_at_ms: i64,
) -> AdmissionState {
    let signals =
        build_admission_signals(packet, policy, market_artifacts, candidate_created_at_ms);
    let quarantine_reasons = collect_quarantine_reasons(policy, &signals);
    let reject_reasons = collect_reject_reasons(policy, &signals);
    let observe_reasons = collect_observe_reasons(packet, policy, &signals);

    AdmissionState {
        has_valid_schema: signals.has_valid_schema,
        has_required_times: signals.has_required_times,
        has_valid_time_order: signals.has_valid_time_order,
        has_evidence: signals.has_evidence,
        has_lineage: signals.has_lineage,
        has_symbols: signals.has_symbols,
        has_source_independence: signals.has_source_independence,
        source_independence_ok_for_research: signals.source_independence_ok_for_research,
        source_independence_ok_for_strong: signals.source_independence_ok_for_strong,
        has_symbol_resolution_trace: signals.has_symbol_resolution_trace,
        symbol_resolution_ok_for_research: signals.symbol_resolution_ok_for_research,
        symbol_resolution_ok_for_strong: signals.symbol_resolution_ok_for_strong,
        has_data_quality_summary: signals.has_data_quality_summary,
        has_market_feature_delta: signals.has_market_feature_delta,
        has_market_regime_context: signals.has_market_regime_context,
        selected_market_feature_delta: signals.selected_market_feature_delta,
        selected_market_regime_context: signals.selected_market_regime_context,
        has_point_in_time_universe: signals.has_point_in_time_universe,
        approved_universe_symbol: signals.approved_universe_symbol,
        has_derivatives_metric_delta: signals.has_derivatives_metric_delta,
        market_context_allows_research: signals.market_context_allows_research,
        market_context_allows_strong: signals.market_context_allows_strong,
        stale_market_context: signals.stale_market_context,
        social_only: signals.social_only,
        has_medium_or_high_contradiction: signals.has_medium_or_high_contradiction,
        forbidden_output_terms: signals.forbidden_output_terms,
        quarantine_reasons,
        reject_reasons,
        observe_reasons,
    }
}

fn build_admission_signals(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    market_artifacts: MarketArtifactInputs<'_>,
    candidate_created_at_ms: i64,
) -> AdmissionSignals {
    let universe = market_artifacts.universe;
    let market_artifact_cutoff_ms = market_artifact_cutoff_ms(packet, candidate_created_at_ms);
    let has_valid_schema =
        packet.schema_version.as_deref() == Some(STRUCTURED_PACKET_SCHEMA_VERSION);
    let forbidden_output_terms = forbidden_generated_terms(packet, policy);
    let has_required_times = has_required_replay_times(packet);
    let has_valid_time_order = has_valid_replay_time_order(packet);
    let has_evidence = !packet.text_evidence.is_empty()
        || !packet.metric_evidence.is_empty()
        || !packet.evidence_sentences.is_empty();
    let has_lineage = !packet.packet_id.trim().is_empty()
        && !packet.cluster_id.trim().is_empty()
        && !packet.source_event_ids.is_empty();
    let has_symbols = !packet.normalized_symbols.is_empty();
    let has_source_independence = packet.source_independence_summary.is_some();
    let source_independence_ok_for_research = packet
        .source_independence_summary
        .as_ref()
        .is_some_and(|summary| source_independence_ok_for_research(summary, policy));
    let source_independence_ok_for_strong = packet
        .source_independence_summary
        .as_ref()
        .is_some_and(|summary| source_independence_ok_for_strong(summary, policy));
    let has_symbol_resolution_trace = !packet.symbol_resolution_trace.is_empty();
    let symbol_resolution_ok_for_research = symbol_resolution_ok(
        packet,
        &policy
            .admission_requirements
            .research_allowed_symbol_mapping_confidence,
    );
    let symbol_resolution_ok_for_strong = symbol_resolution_ok(
        packet,
        &policy
            .admission_requirements
            .strong_allowed_symbol_mapping_confidence,
    );
    let has_data_quality_summary = packet
        .market_context_ref
        .as_ref()
        .and_then(|reference| reference.market_data_quality_summary_key.as_ref())
        .is_some_and(|key| !key.trim().is_empty());
    let selected_market_feature_delta = selected_market_feature_delta_when_referenced(
        packet,
        market_artifacts,
        market_artifact_cutoff_ms,
    );
    let selected_market_regime_context = selected_market_regime_context_when_referenced(
        packet,
        market_artifacts,
        market_artifact_cutoff_ms,
    );
    let has_market_feature_delta = selected_market_feature_delta.is_some();
    let has_market_regime_context = selected_market_regime_context.is_some();
    let has_point_in_time_universe = universe.is_some();
    let approved_universe_symbol =
        universe.is_some_and(|snapshot| approved_universe_symbols(packet, snapshot));
    let has_derivatives_metric_delta =
        has_derivatives_metric_delta(packet, market_artifacts, market_artifact_cutoff_ms);
    let market_status = effective_market_context_status(packet);
    let market_context_allows_strong =
        market_status.as_policy_key() == policy.market_context_status_policy.strong_requires;
    let market_context_allows_research =
        market_context_allows_research(packet, policy, &market_status);
    let stale_market_context = matches!(market_status, MarketContextStatus::StaleButUsable);
    let social_only = packet
        .source_quality_summary
        .to_ascii_lowercase()
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .any(|token| token == "social_only");
    let has_medium_or_high_contradiction = packet
        .contradiction_flags
        .iter()
        .any(is_medium_contradiction);

    AdmissionSignals {
        has_valid_schema,
        has_required_times,
        has_valid_time_order,
        has_evidence,
        has_lineage,
        has_symbols,
        has_source_independence,
        source_independence_ok_for_research,
        source_independence_ok_for_strong,
        has_symbol_resolution_trace,
        symbol_resolution_ok_for_research,
        symbol_resolution_ok_for_strong,
        has_data_quality_summary,
        has_market_feature_delta,
        has_market_regime_context,
        selected_market_feature_delta,
        selected_market_regime_context,
        has_point_in_time_universe,
        approved_universe_symbol,
        has_derivatives_metric_delta,
        market_context_allows_research,
        market_context_allows_strong,
        stale_market_context,
        social_only,
        has_medium_or_high_contradiction,
        forbidden_output_terms,
    }
}

fn collect_quarantine_reasons(policy: &ScoringPolicy, signals: &AdmissionSignals) -> Vec<String> {
    let mut reasons = gated_reasons([(
        policy.hard_gates.forbid_invalid_schema && !signals.has_valid_schema,
        "invalid_schema_version",
    )]);
    if policy.hard_gates.forbid_forbidden_output_terms && !signals.forbidden_output_terms.is_empty()
    {
        reasons.push(format!(
            "forbidden_output_terms:{}",
            signals.forbidden_output_terms.join(",")
        ));
    }
    reasons
}

fn collect_reject_reasons(policy: &ScoringPolicy, signals: &AdmissionSignals) -> Vec<String> {
    gated_reasons([
        (
            policy.hard_gates.forbid_missing_evidence && !signals.has_evidence,
            "missing_evidence",
        ),
        (
            policy.hard_gates.forbid_missing_lineage && !signals.has_lineage,
            "missing_lineage",
        ),
        (!signals.has_symbols, "missing_normalized_symbols"),
    ])
}

fn collect_observe_reasons(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    signals: &AdmissionSignals,
) -> Vec<String> {
    dedupe_strings(gated_reasons([
        (
            policy.hard_gates.require_decision_available_at_ms && !signals.has_required_times,
            "missing_replay_time_contract",
        ),
        (
            signals.has_required_times && !signals.has_valid_time_order,
            "invalid_replay_time_order",
        ),
        (
            policy.hard_gates.require_point_in_time_universe && !signals.has_point_in_time_universe,
            "missing_point_in_time_universe",
        ),
        (
            policy.hard_gates.require_approved_universe_for_research
                && !signals.approved_universe_symbol,
            "not_admitted_universe",
        ),
        (
            policy.hard_gates.require_data_quality_summary_for_research
                && !signals.has_data_quality_summary,
            "missing_data_quality_summary",
        ),
        (
            policy.hard_gates.require_market_feature_delta_for_research
                && !signals.has_market_feature_delta,
            "missing_market_feature_delta",
        ),
        (
            policy.hard_gates.require_market_regime_context_for_research
                && !signals.has_market_regime_context,
            "missing_market_regime_context",
        ),
        (
            policy.hard_gates.require_source_independence_for_research
                && !signals.has_source_independence,
            "missing_source_independence",
        ),
        (
            signals.has_source_independence && !signals.source_independence_ok_for_research,
            "insufficient_source_independence",
        ),
        (
            policy
                .hard_gates
                .require_symbol_resolution_trace_for_research
                && !signals.has_symbol_resolution_trace,
            "missing_symbol_resolution_trace",
        ),
        (
            signals.has_symbol_resolution_trace && !signals.symbol_resolution_ok_for_research,
            "weak_symbol_resolution",
        ),
        (
            policy
                .hard_gates
                .forbid_research_without_metric_delta_for_derivatives
                && packet.event_type.is_derivatives_like()
                && !signals.has_derivatives_metric_delta,
            "derivatives_metric_delta_missing",
        ),
        (
            !signals.market_context_allows_research,
            "market_context_not_research_admissible",
        ),
    ]))
}

fn gated_reasons<const N: usize>(checks: [(bool, &str); N]) -> Vec<String> {
    checks
        .into_iter()
        .filter(|(enabled, _reason)| *enabled)
        .map(|(_enabled, reason)| reason.to_owned())
        .collect()
}

fn has_required_replay_times(packet: &StructuredIntelPacket) -> bool {
    packet.fetched_at_ms.is_some()
        && packet.structured_at_ms.is_some()
        && packet.decision_available_at_ms.is_some()
}

fn has_valid_replay_time_order(packet: &StructuredIntelPacket) -> bool {
    let Some(decision_available_at_ms) = packet.decision_available_at_ms else {
        return false;
    };
    let Some(fetched_at_ms) = packet.fetched_at_ms else {
        return false;
    };
    let Some(structured_at_ms) = packet.structured_at_ms else {
        return false;
    };
    decision_available_at_ms >= packet.published_at_ms.unwrap_or(fetched_at_ms)
        && decision_available_at_ms >= fetched_at_ms
        && decision_available_at_ms >= structured_at_ms
}

fn market_artifact_cutoff_ms(
    packet: &StructuredIntelPacket,
    candidate_created_at_ms: i64,
) -> Option<i64> {
    packet
        .decision_available_at_ms
        .map(|decision_available_at_ms| decision_available_at_ms.max(candidate_created_at_ms))
}

fn selected_market_feature_delta_when_referenced(
    packet: &StructuredIntelPacket,
    market_artifacts: MarketArtifactInputs<'_>,
    market_artifact_cutoff_ms: Option<i64>,
) -> Option<SelectedMarketArtifactTrace> {
    let has_reference = packet
        .market_context_ref
        .as_ref()
        .and_then(market_feature_delta_artifact_key)
        .is_some_and(|key| !key.trim().is_empty());
    has_reference
        .then(|| {
            selected_market_feature_delta(
                packet,
                market_artifacts,
                market_artifact_cutoff_ms,
                |_| true,
            )
        })
        .flatten()
}

fn selected_market_regime_context_when_referenced(
    packet: &StructuredIntelPacket,
    market_artifacts: MarketArtifactInputs<'_>,
    market_artifact_cutoff_ms: Option<i64>,
) -> Option<SelectedMarketArtifactTrace> {
    let has_reference = packet
        .market_context_ref
        .as_ref()
        .and_then(|reference| reference.market_regime_context_key.as_ref())
        .is_some_and(|key| !key.trim().is_empty());
    has_reference
        .then(|| {
            selected_market_regime_context(packet, market_artifacts, market_artifact_cutoff_ms)
        })
        .flatten()
}

fn has_derivatives_metric_delta(
    packet: &StructuredIntelPacket,
    market_artifacts: MarketArtifactInputs<'_>,
    market_artifact_cutoff_ms: Option<i64>,
) -> bool {
    packet.metric_evidence.iter().any(|metric| {
        metric.delta_pct.is_some()
            && matches!(
                metric.metric_name.as_str(),
                "open_interest" | "funding_rate" | "liquidation" | "long_short_ratio"
            )
    }) || selected_derivatives_market_feature_delta(
        packet,
        market_artifacts,
        market_artifact_cutoff_ms,
    )
    .is_some()
}

fn calculate_score(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    universe: Option<&SymbolUniverseSnapshot>,
    admission: &AdmissionState,
) -> ScoreBreakdown {
    let mut components = Vec::new();
    push_source_score_components(&mut components, packet, policy);
    push_market_score_components(&mut components, packet, policy, universe, admission);
    push_quality_score_components(&mut components, packet, policy, admission);
    push_novelty_score_component(&mut components, packet, policy);
    push_contradiction_score_components(&mut components, packet, policy);
    push_penalty_score_components(&mut components, policy, admission);
    push_evidence_quality_score_components(&mut components, packet, policy);
    let final_score = components.iter().map(|component| component.value).sum();
    ScoreBreakdown {
        components,
        final_score,
    }
}

fn push_source_score_components(
    components: &mut Vec<ScoreComponent>,
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
) {
    push_component(
        components,
        "symbol_confidence",
        policy.weight(&format!(
            "symbol_confidence_{}",
            packet.symbol_confidence_band.as_policy_key()
        )),
        "symbol confidence band",
    );
    if packet
        .source_independence_summary
        .as_ref()
        .is_some_and(|summary| summary.independent_source_count >= 2)
    {
        push_component(
            components,
            "source_independence_multi",
            policy.weight("source_independence_multi"),
            "two or more independent sources",
        );
    }
    if packet
        .source_independence_summary
        .as_ref()
        .is_some_and(|summary| summary.official_source_present)
    {
        push_component(
            components,
            "official_source",
            policy.weight("official_source"),
            "official source present",
        );
    }
}

fn push_market_score_components(
    components: &mut Vec<ScoreComponent>,
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    universe: Option<&SymbolUniverseSnapshot>,
    admission: &AdmissionState,
) {
    let market_status = effective_market_context_status(packet);
    push_component(
        components,
        "market_context",
        policy.weight(&format!("market_context_{}", market_status.as_policy_key())),
        "market context status",
    );
    if universe.is_some() && admission.approved_universe_symbol {
        push_component(
            components,
            "approved_universe_symbol",
            policy.weight("approved_universe_symbol"),
            "point-in-time universe approved every symbol",
        );
    }
}

fn push_quality_score_components(
    components: &mut Vec<ScoreComponent>,
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    admission: &AdmissionState,
) {
    if admission.has_data_quality_summary {
        push_component(
            components,
            "data_quality_good",
            policy.weight("data_quality_good"),
            "market data quality summary present",
        );
    }
    if packet
        .symbol_resolution_trace
        .iter()
        .any(|trace| trace.mapping_confidence.is_strong())
    {
        push_component(
            components,
            "symbol_resolution_strong",
            policy.weight("symbol_resolution_strong"),
            "strong symbol resolution trace",
        );
    }
    if admission.has_derivatives_metric_delta {
        push_component(
            components,
            "metric_delta_present",
            policy.weight("metric_delta_present"),
            "metric evidence includes delta_pct",
        );
    }
    push_component(
        components,
        "confidence_band",
        policy.weight(&format!(
            "confidence_{}",
            packet.confidence_band.as_policy_key()
        )),
        "packet confidence band",
    );
}

fn push_novelty_score_component(
    components: &mut Vec<ScoreComponent>,
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
) {
    if packet.novelty_score >= 0.7 {
        push_component(
            components,
            "novelty_high",
            policy.weight("novelty_high"),
            "novelty score >= 0.7",
        );
    } else if packet.novelty_score >= 0.4 {
        push_component(
            components,
            "novelty_medium",
            policy.weight("novelty_medium"),
            "novelty score >= 0.4",
        );
    }
}

fn push_contradiction_score_components(
    components: &mut Vec<ScoreComponent>,
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
) {
    for flag in &packet.contradiction_flags {
        if is_medium_contradiction(flag) {
            push_component(
                components,
                "contradiction_medium_penalty",
                policy.weight("contradiction_medium_penalty"),
                "medium contradiction flag",
            );
        } else {
            push_component(
                components,
                "contradiction_low_penalty",
                policy.weight("contradiction_low_penalty"),
                "low contradiction flag",
            );
        }
        if matches!(flag, ContradictionFlag::EvidenceWeak) {
            push_component(
                components,
                "legacy_evidence_weak_penalty",
                policy.weight("legacy_evidence_weak_penalty"),
                "legacy evidence_weak flag",
            );
        }
    }
}

fn push_penalty_score_components(
    components: &mut Vec<ScoreComponent>,
    policy: &ScoringPolicy,
    admission: &AdmissionState,
) {
    if admission.social_only {
        push_component(
            components,
            "social_only_penalty",
            policy.weight("social_only_penalty"),
            "social-only source quality",
        );
    }
    if admission.stale_market_context {
        push_component(
            components,
            "stale_event_penalty",
            policy.weight("stale_event_penalty"),
            "stale market context",
        );
    }
}

fn push_evidence_quality_score_components(
    components: &mut Vec<ScoreComponent>,
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
) {
    for reason in &packet.evidence_quality_reasons {
        push_component(
            components,
            reason.as_policy_key(),
            policy.evidence_penalty(reason.as_policy_key()),
            "evidence quality reason",
        );
    }
}

fn classify_candidate(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    admission: &AdmissionState,
    score: &ScoreBreakdown,
    reasons: &mut Vec<String>,
) -> CandidateClass {
    if !admission.quarantine_reasons.is_empty() {
        return CandidateClass::Quarantine;
    }
    if !admission.reject_reasons.is_empty() {
        return CandidateClass::Reject;
    }

    let research_block_reasons = admission.research_block_reasons(packet);
    let strong_block_reasons = admission.strong_block_reasons();
    let desired = if score.final_score >= policy.thresholds.strong_candidate {
        CandidateClass::StrongCandidate
    } else if score.final_score >= policy.thresholds.research_candidate {
        CandidateClass::ResearchCandidate
    } else if score.final_score >= policy.thresholds.weak_candidate {
        CandidateClass::WeakCandidate
    } else {
        CandidateClass::ObserveOnly
    };

    let class = match desired {
        CandidateClass::StrongCandidate => {
            if strong_block_reasons.is_empty() && research_block_reasons.is_empty() {
                CandidateClass::StrongCandidate
            } else if research_block_reasons.is_empty() {
                reasons.extend(strong_block_reasons);
                CandidateClass::ResearchCandidate
            } else {
                reasons.extend(strong_block_reasons);
                reasons.extend(research_block_reasons);
                downgraded_non_research_class(packet, policy, score, admission)
            }
        }
        CandidateClass::ResearchCandidate => {
            if research_block_reasons.is_empty() {
                CandidateClass::ResearchCandidate
            } else {
                reasons.extend(research_block_reasons);
                downgraded_non_research_class(packet, policy, score, admission)
            }
        }
        CandidateClass::WeakCandidate => CandidateClass::WeakCandidate,
        CandidateClass::ObserveOnly => CandidateClass::ObserveOnly,
        CandidateClass::Reject | CandidateClass::Quarantine => desired,
    };

    let deduped = dedupe_strings(std::mem::take(reasons));
    reasons.extend(deduped);
    class
}

fn downgraded_non_research_class(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    score: &ScoreBreakdown,
    admission: &AdmissionState,
) -> CandidateClass {
    if admission.stale_market_context
        || matches!(
            packet.event_type,
            EventType::SocialHype | EventType::SocialBacklash
        )
    {
        return CandidateClass::WeakCandidate;
    }
    if score.final_score >= policy.thresholds.weak_candidate
        && admission.has_evidence
        && admission.has_lineage
    {
        CandidateClass::WeakCandidate
    } else {
        CandidateClass::ObserveOnly
    }
}

struct BundleBuildContext<'a> {
    candidate_id: &'a str,
    packet: &'a StructuredIntelPacket,
    policy: &'a ScoringPolicy,
    universe: &'a SymbolUniverseSnapshot,
    created_at_ms: i64,
    candidate_class: CandidateClass,
    score_breakdown: ScoreBreakdown,
    reasons: Vec<String>,
    selected_market_artifacts: Vec<SelectedMarketArtifactTrace>,
    idempotency_key: String,
}

struct HypothesisStateBuildContext<'a> {
    hypothesis_id: &'a str,
    packet: &'a StructuredIntelPacket,
    policy: &'a ScoringPolicy,
    created_at_ms: i64,
    candidate_class: CandidateClass,
    score_breakdown: ScoreBreakdown,
    reasons: Vec<String>,
    selected_market_artifacts: Vec<SelectedMarketArtifactTrace>,
    idempotency_key: String,
    screening_event_id: String,
}

fn build_evidence_bundle(context: BundleBuildContext<'_>) -> IntelCandidateEvidenceBundle {
    let candidate_id = context.candidate_id;
    let packet = context.packet;
    let policy = context.policy;
    let universe = context.universe;
    let created_at_ms = context.created_at_ms;
    let packet_decision_available_at_ms = packet
        .decision_available_at_ms
        .expect("research-eligible candidates require decision_available_at_ms");
    let event_time_ms = packet_decision_available_at_ms;
    let decision_available_at_ms = packet_decision_available_at_ms.max(created_at_ms);
    let fetched_at_ms = packet
        .fetched_at_ms
        .expect("research-eligible candidates require fetched_at_ms");
    let structured_at_ms = packet
        .structured_at_ms
        .expect("research-eligible candidates require structured_at_ms");
    let candidate_score = context.score_breakdown.final_score;
    let hypothesis_type = policy
        .event_type_to_hypothesis_type
        .get(packet.event_type.as_policy_key())
        .cloned()
        .unwrap_or_else(|| "general_intel_observation".to_owned());
    let allowed_horizons = policy
        .event_type_to_allowed_horizons
        .get(packet.event_type.as_policy_key())
        .cloned()
        .unwrap_or_else(|| vec!["24h".to_owned()]);
    let horizon_key = allowed_horizons.join(",");
    let lifecycle_key = stable_id(
        "cand_life",
        &[
            packet
                .normalized_symbols
                .first()
                .map(String::as_str)
                .unwrap_or("unknown_symbol"),
            hypothesis_type.as_str(),
            packet.event_type.as_policy_key(),
            horizon_key.as_str(),
        ],
    );
    let research_priority = research_priority(&packet.event_type, &packet.confidence_band);
    let priority_partition = research_priority_partition(&research_priority);
    let bundle_key = candidate_bundle_key(created_at_ms, candidate_id, priority_partition);
    let source_independence = packet
        .source_independence_summary
        .clone()
        .expect("research-eligible candidates require source_independence_summary");
    let data_quality_summary = DataQualitySummaryRef {
        market_data_quality_summary_key: packet
            .market_context_ref
            .as_ref()
            .and_then(|reference| reference.market_data_quality_summary_key.clone()),
        status: if packet
            .market_context_ref
            .as_ref()
            .and_then(|reference| reference.market_data_quality_summary_key.as_ref())
            .is_some()
        {
            "present".to_owned()
        } else {
            "missing".to_owned()
        },
    };
    let parent_artifact_ids = parent_artifact_ids(packet);
    let evidence_refs = evidence_refs(packet);
    let mut confidence_summary = BTreeMap::new();
    confidence_summary.insert(
        "symbol_confidence_band".to_owned(),
        packet.symbol_confidence_band.as_policy_key().to_owned(),
    );
    confidence_summary.insert(
        "packet_confidence_band".to_owned(),
        packet.confidence_band.as_policy_key().to_owned(),
    );
    confidence_summary.insert(
        "market_context_status".to_owned(),
        effective_market_context_status(packet)
            .as_policy_key()
            .to_owned(),
    );

    let mut bundle = IntelCandidateEvidenceBundle {
        candidate_id: candidate_id.to_owned(),
        candidate_lifecycle_key: lifecycle_key,
        bundle_key: bundle_key.clone(),
        producer_app: PRODUCER_APP.to_owned(),
        producer_run_id: stable_id("cand_run", &[candidate_id, &created_at_ms.to_string()]),
        created_at_ms,
        event_time_ms,
        published_at_ms: packet.published_at_ms,
        fetched_at_ms,
        structured_at_ms,
        candidate_created_at_ms: created_at_ms,
        decision_available_at_ms,
        forbidden_lookahead_boundary_ms: decision_available_at_ms,
        schema_version: CANDIDATE_BUNDLE_SCHEMA_VERSION.to_owned(),
        scoring_policy_version: policy.policy_version.clone(),
        normalized_symbols: packet.normalized_symbols.clone(),
        input_packet_family_id: effective_packet_family_id(packet).to_owned(),
        input_packet_revision: packet.revision,
        supersedes_packet_id: packet.supersedes_packet_id.clone(),
        symbol_universe_snapshot_id: universe.symbol_universe_snapshot_id.clone(),
        universe_as_of_ms: universe.universe_as_of_ms,
        approved_universe_symbol: approved_universe_symbols(packet, universe),
        event_types: vec![packet.event_type.as_policy_key().to_owned()],
        hypothesis_type,
        allowed_horizons,
        source_story_cluster_ids: vec![packet.cluster_id.clone()],
        source_structured_packet_ids: vec![packet.packet_id.clone()],
        source_context_flag_packet_ids: Vec::new(),
        evidence_refs,
        text_evidence: packet.text_evidence.clone(),
        metric_evidence: packet.metric_evidence.clone(),
        market_context_ref: packet.market_context_ref.clone(),
        data_quality_summary,
        selected_market_artifacts: context.selected_market_artifacts,
        candidate_class: context.candidate_class,
        candidate_score,
        score_breakdown: context.score_breakdown,
        research_priority,
        research_eligible: true,
        validation_requirements: validation_requirements(&policy.validation_requirement_defaults),
        source_independence,
        symbol_resolution_trace: packet.symbol_resolution_trace.clone(),
        confidence_summary,
        contradiction_summary: packet.contradiction_flags.clone(),
        observe_or_reject_reasons: context.reasons,
        parent_artifact_ids,
        storage_uri: bundle_key,
        checksum: String::new(),
        idempotency_key: context.idempotency_key,
    };
    let checksum_payload = serde_json::to_vec(&bundle).expect("candidate bundle is serializable");
    bundle.checksum = sha256_hex(checksum_payload);
    bundle
}

fn build_hypothesis_state(
    context: HypothesisStateBuildContext<'_>,
) -> IntelCandidateHypothesisState {
    let packet = context.packet;
    let market_context_status = effective_market_context_status(packet);
    let hypothesis_type = hypothesis_type(context.policy, &packet.event_type);
    let state_key = hypothesis_state_key(context.created_at_ms, context.hypothesis_id);
    let retryable_reasons = retryable_reasons(&context.reasons);
    let terminal_reasons = terminal_reasons(&context.reasons);
    let next_action = next_hypothesis_action(&context.candidate_class, &retryable_reasons);
    let mut state = IntelCandidateHypothesisState {
        hypothesis_id: context.hypothesis_id.to_owned(),
        state_key: state_key.clone(),
        schema_version: HYPOTHESIS_STATE_SCHEMA_VERSION.to_owned(),
        producer_app: PRODUCER_APP.to_owned(),
        producer_version: env!("CARGO_PKG_VERSION").to_owned(),
        created_at_ms: context.created_at_ms,
        updated_at_ms: context.created_at_ms,
        input_packet_id: packet.packet_id.clone(),
        input_packet_family_id: effective_packet_family_id(packet).to_owned(),
        input_packet_revision: packet.revision,
        source_structured_packet_ids: vec![packet.packet_id.clone()],
        source_event_ids: packet.source_event_ids.clone(),
        supersedes_packet_id: packet.supersedes_packet_id.clone(),
        supersedes_hypothesis_id: None,
        latest_screening_event_id: context.screening_event_id,
        scoring_policy_version: context.policy.policy_version.clone(),
        normalized_symbols: packet.normalized_symbols.clone(),
        event_type: packet.event_type.as_policy_key().to_owned(),
        hypothesis_type,
        current_state: context.candidate_class,
        current_score: context.score_breakdown.final_score,
        previous_score: None,
        research_eligible: false,
        transition: "created_or_refreshed".to_owned(),
        next_action,
        reasons: context.reasons,
        retryable_reasons,
        terminal_reasons,
        selected_market_artifacts: context.selected_market_artifacts,
        market_context_ref: packet.market_context_ref.clone(),
        market_context_status,
        evidence_quality_reasons: packet.evidence_quality_reasons.clone(),
        score_breakdown: context.score_breakdown,
        lineage_refs: hypothesis_lineage_refs(packet),
        dirty_triggers: dirty_triggers(packet),
        harness_queue_hint: harness_queue_hint(packet),
        idempotency_key: context.idempotency_key,
        checksum: String::new(),
    };
    let checksum_payload = serde_json::to_vec(&state).expect("hypothesis state is serializable");
    state.checksum = sha256_hex(checksum_payload);
    state
}

pub fn effective_packet_family_id(packet: &StructuredIntelPacket) -> &str {
    if !packet.packet_family_id.trim().is_empty() {
        packet.packet_family_id.as_str()
    } else if !packet.raw_event_id.trim().is_empty() {
        packet.raw_event_id.as_str()
    } else {
        packet.packet_id.as_str()
    }
}

pub fn candidate_bundle_key(
    created_at_ms: i64,
    candidate_id: &str,
    priority_partition: &str,
) -> String {
    let part = time_part(created_at_ms);
    format!(
        "candidate-evidence-bundle/priority={}/schema={}/dt={}/hour={:02}/candidate_id={}/part-000001.jsonl",
        path_segment(priority_partition),
        CANDIDATE_BUNDLE_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(candidate_id)
    )
}

pub fn screening_event_key(created_at_ms: i64, screening_event_id: &str) -> String {
    let part = time_part(created_at_ms);
    format!(
        "candidate-screening/schema={}/dt={}/hour={:02}/screening_event_id={}/part-000001.jsonl",
        SCREENING_EVENT_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(screening_event_id)
    )
}

pub fn hypothesis_state_key(created_at_ms: i64, hypothesis_id: &str) -> String {
    let part = time_part(created_at_ms);
    format!(
        "hypothesis-state/schema={}/dt={}/hour={:02}/hypothesis_id={}/state.json",
        HYPOTHESIS_STATE_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(hypothesis_id)
    )
}

fn candidate_id(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    universe: Option<&SymbolUniverseSnapshot>,
) -> String {
    let decision_available_at_ms = packet.decision_available_at_ms.unwrap_or_default();
    let event_bucket = hour_bucket_ms(decision_available_at_ms).to_string();
    let decision_key = decision_available_at_ms.to_string();
    let symbols = packet.normalized_symbols.join(",");
    let universe_id = universe
        .map(|snapshot| snapshot.symbol_universe_snapshot_id.as_str())
        .unwrap_or("missing_universe");
    let hypothesis_type = policy
        .event_type_to_hypothesis_type
        .get(packet.event_type.as_policy_key())
        .map(String::as_str)
        .unwrap_or("general_intel_observation");
    stable_id(
        "cand",
        &[
            policy.policy_version.as_str(),
            symbols.as_str(),
            packet.event_type.as_policy_key(),
            packet.cluster_id.as_str(),
            event_bucket.as_str(),
            decision_key.as_str(),
            universe_id,
            hypothesis_type,
        ],
    )
}

fn hypothesis_type(policy: &ScoringPolicy, event_type: &EventType) -> String {
    policy
        .event_type_to_hypothesis_type
        .get(event_type.as_policy_key())
        .cloned()
        .unwrap_or_else(|| "general_intel_observation".to_owned())
}

fn retryable_reasons(reasons: &[String]) -> Vec<String> {
    let retryable_markers = [
        "missing_market_feature_delta",
        "missing_market_regime_context",
        "market_context_not_research_admissible",
        "derivatives_metric_delta_missing",
        "baseline_missing",
        "single_numeric_snapshot",
        "single_source_only",
        "missing_source_independence",
        "insufficient_source_independence",
        "missing_symbol_resolution_trace",
        "weak_symbol_resolution",
    ];
    reasons
        .iter()
        .filter(|reason| retryable_markers.contains(&reason.as_str()))
        .cloned()
        .collect()
}

fn terminal_reasons(reasons: &[String]) -> Vec<String> {
    let terminal_markers = [
        "invalid_schema_version",
        "invalid_replay_time_order",
        "forbidden_output_term",
        "not_admitted_universe",
    ];
    reasons
        .iter()
        .filter(|reason| {
            terminal_markers.contains(&reason.as_str())
                || reason.starts_with("forbidden_generated_term:")
        })
        .cloned()
        .collect()
}

fn next_hypothesis_action(class: &CandidateClass, retryable_reasons: &[String]) -> String {
    if matches!(class, CandidateClass::Reject) && retryable_reasons.is_empty() {
        "archive_until_revision".to_owned()
    } else if retryable_reasons.iter().any(|reason| {
        reason == "missing_market_feature_delta" || reason == "derivatives_metric_delta_missing"
    }) {
        "rerun_when_market_feature_delta_updates".to_owned()
    } else if retryable_reasons
        .iter()
        .any(|reason| reason == "market_context_not_research_admissible")
    {
        "rerun_when_market_context_updates".to_owned()
    } else {
        "enqueue_cheap_harness".to_owned()
    }
}

fn dirty_triggers(packet: &StructuredIntelPacket) -> Vec<String> {
    let mut triggers = vec![
        "scoring_policy_version_changed".to_owned(),
        "candidate_app_version_changed".to_owned(),
    ];
    if packet.market_context_ref.is_some()
        || !matches!(packet.market_context_status, MarketContextStatus::Unknown)
    {
        triggers.push("market_context_updated".to_owned());
        triggers.push("market_feature_delta_updated".to_owned());
    }
    if matches!(packet.event_type, EventType::FundingShift) {
        triggers.push("derivatives_rollup_updated".to_owned());
    }
    triggers
}

fn harness_queue_hint(packet: &StructuredIntelPacket) -> String {
    match packet.event_type {
        EventType::FundingShift => "derivatives_delta_persistence".to_owned(),
        EventType::SocialHype | EventType::SocialBacklash => "attention_reaction_smoke".to_owned(),
        EventType::Listing
        | EventType::ExchangeListing
        | EventType::Delisting
        | EventType::ExchangeDelisting => "venue_event_reaction".to_owned(),
        _ => "event_reaction_smoke".to_owned(),
    }
}

fn hypothesis_lineage_refs(packet: &StructuredIntelPacket) -> Vec<String> {
    let mut refs = parent_artifact_ids(packet);
    if let Some(reference) = &packet.market_context_ref {
        refs.extend(reference.output_object_keys.clone());
        refs.extend(reference.market_data_quality_summary_key.clone());
        refs.extend(reference.market_feature_delta_key.clone());
        refs.extend(reference.market_feature_delta_summary_key.clone());
        refs.extend(reference.market_regime_context_key.clone());
        refs.extend(reference.symbol_universe_snapshot_key.clone());
    }
    dedupe_strings(refs)
}

fn push_component(
    components: &mut Vec<ScoreComponent>,
    name: impl Into<String>,
    value: i64,
    reason: impl Into<String>,
) {
    if value != 0 {
        components.push(ScoreComponent {
            name: name.into(),
            value,
            reason: reason.into(),
        });
    }
}

fn source_independence_ok_for_research(
    summary: &SourceIndependenceSummary,
    policy: &ScoringPolicy,
) -> bool {
    summary.independent_source_count
        >= policy
            .admission_requirements
            .research_min_independent_source_count
        || (policy
            .admission_requirements
            .official_source_can_replace_min_independent_source_count
            && summary.official_source_present)
}

fn source_independence_ok_for_strong(
    summary: &SourceIndependenceSummary,
    policy: &ScoringPolicy,
) -> bool {
    summary.independent_source_count
        >= policy
            .admission_requirements
            .strong_min_independent_source_count
        || (policy
            .admission_requirements
            .official_source_can_replace_min_independent_source_count
            && summary.official_source_present)
}

fn symbol_resolution_ok(packet: &StructuredIntelPacket, allowed: &[String]) -> bool {
    if packet.normalized_symbols.is_empty() || packet.symbol_resolution_trace.is_empty() {
        return false;
    }
    let allowed: BTreeSet<String> = allowed
        .iter()
        .map(|value| value.to_ascii_lowercase())
        .collect();
    packet.normalized_symbols.iter().all(|symbol| {
        let candidates = canonical_symbol_candidates(symbol);
        packet.symbol_resolution_trace.iter().any(|trace| {
            trace
                .ambiguity_reason
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
                && trace
                    .canonical_symbol
                    .as_ref()
                    .is_some_and(|canonical| candidates.contains(&canonical.to_ascii_uppercase()))
                && allowed.contains(trace.mapping_confidence.as_policy_key())
        })
    })
}

fn approved_universe_symbols(
    packet: &StructuredIntelPacket,
    snapshot: &SymbolUniverseSnapshot,
) -> bool {
    if packet.normalized_symbols.is_empty() {
        return false;
    }
    packet.normalized_symbols.iter().all(|symbol| {
        let candidates = canonical_symbol_candidates(symbol);
        snapshot.included_symbols.iter().any(|member| {
            candidates.contains(&member.symbol_canonical.to_ascii_uppercase())
                && member.approved_universe_symbol
        })
    })
}

fn canonical_symbol_candidates(symbol: &str) -> BTreeSet<String> {
    let upper = symbol.trim().to_ascii_uppercase();
    let mut values = BTreeSet::from([upper.clone()]);
    for suffix in ["USDT", "USDC", "USD", "BUSD", "BTC", "ETH"] {
        if upper.len() > suffix.len() && upper.ends_with(suffix) {
            values.insert(upper.trim_end_matches(suffix).to_owned());
        }
    }
    values
}

fn selected_derivatives_market_feature_delta(
    packet: &StructuredIntelPacket,
    market_artifacts: MarketArtifactInputs<'_>,
    market_artifact_cutoff_ms: Option<i64>,
) -> Option<SelectedMarketArtifactTrace> {
    selected_market_feature_delta(
        packet,
        market_artifacts,
        market_artifact_cutoff_ms,
        |metric_name| {
            matches!(
                metric_name,
                "open_interest" | "funding_rate" | "liquidation" | "long_short_ratio"
            )
        },
    )
}

fn selected_market_feature_delta(
    packet: &StructuredIntelPacket,
    market_artifacts: MarketArtifactInputs<'_>,
    market_artifact_cutoff_ms: Option<i64>,
    metric_allowed: impl Fn(&str) -> bool,
) -> Option<SelectedMarketArtifactTrace> {
    let market_artifact_cutoff_ms = market_artifact_cutoff_ms?;
    let artifact_key = packet
        .market_context_ref
        .as_ref()
        .and_then(market_feature_delta_artifact_key)
        .cloned();
    let artifact_type = packet
        .market_context_ref
        .as_ref()
        .and_then(|reference| {
            reference
                .market_feature_delta_summary_key
                .as_ref()
                .filter(|key| !key.trim().is_empty())
        })
        .map(|_| "market_feature_delta_summary")
        .unwrap_or("market_feature_delta");
    let metric_allowed_ref = &metric_allowed;
    packet
        .normalized_symbols
        .iter()
        .flat_map(|symbol| {
            let candidates = canonical_symbol_candidates(symbol);
            market_artifacts
                .market_feature_deltas
                .iter()
                .filter(move |delta| {
                    candidates.contains(&delta.symbol_canonical.to_ascii_uppercase())
                        && metric_allowed_ref(delta.metric_name.as_str())
                        && delta.window_end_ms <= market_artifact_cutoff_ms
                        && delta.known_as_of_ms <= market_artifact_cutoff_ms
                        && is_usable_market_artifact_quality(&delta.quality_status)
                        && (delta.change_pct_1h.is_some() || delta.change_pct_15m.is_some())
                })
        })
        .max_by_key(|delta| (delta.window_end_ms, delta.known_as_of_ms))
        .map(|delta| SelectedMarketArtifactTrace {
            artifact_type: artifact_type.to_owned(),
            artifact_id: delta.feature_delta_id.clone(),
            artifact_key,
            l1_run_id: Some(delta.l1_run_id.clone()),
            symbol_canonical: Some(delta.symbol_canonical.clone()),
            metric_name: Some(delta.metric_name.clone()),
            scope: None,
            window_start_ms: delta.window_start_ms,
            window_end_ms: delta.window_end_ms,
            known_as_of_ms: delta.known_as_of_ms,
            quality_status: normalize_market_artifact_quality(&delta.quality_status),
        })
}

fn market_feature_delta_artifact_key(reference: &MarketContextRef) -> Option<&String> {
    reference
        .market_feature_delta_summary_key
        .as_ref()
        .filter(|key| !key.trim().is_empty())
        .or_else(|| {
            reference
                .market_feature_delta_key
                .as_ref()
                .filter(|key| !key.trim().is_empty())
        })
}

fn selected_market_regime_context(
    packet: &StructuredIntelPacket,
    market_artifacts: MarketArtifactInputs<'_>,
    market_artifact_cutoff_ms: Option<i64>,
) -> Option<SelectedMarketArtifactTrace> {
    let market_artifact_cutoff_ms = market_artifact_cutoff_ms?;
    let artifact_key = packet
        .market_context_ref
        .as_ref()
        .and_then(|reference| reference.market_regime_context_key.clone());
    market_artifacts
        .market_regime_contexts
        .iter()
        .filter(|context| {
            context.window_end_ms <= market_artifact_cutoff_ms
                && context.known_as_of_ms <= market_artifact_cutoff_ms
                && context.sector_return_same_window.is_some()
                && is_usable_market_artifact_quality(&context.quality_status)
        })
        .max_by_key(|context| (context.window_end_ms, context.known_as_of_ms))
        .map(|context| SelectedMarketArtifactTrace {
            artifact_type: "market_regime_context".to_owned(),
            artifact_id: context.regime_context_id.clone(),
            artifact_key,
            l1_run_id: Some(context.l1_run_id.clone()),
            symbol_canonical: None,
            metric_name: None,
            scope: Some(context.scope.clone()),
            window_start_ms: context.window_start_ms,
            window_end_ms: context.window_end_ms,
            known_as_of_ms: context.known_as_of_ms,
            quality_status: normalize_market_artifact_quality(&context.quality_status),
        })
}

impl AdmissionState {
    fn selected_market_artifacts(&self) -> Vec<SelectedMarketArtifactTrace> {
        let mut artifacts = Vec::new();
        if let Some(trace) = &self.selected_market_feature_delta {
            artifacts.push(trace.clone());
        }
        if let Some(trace) = &self.selected_market_regime_context {
            artifacts.push(trace.clone());
        }
        artifacts
    }
}

fn is_usable_market_artifact_quality(status: &str) -> bool {
    matches!(status.trim(), "" | "complete" | "partial")
}

fn normalize_market_artifact_quality(status: &str) -> String {
    let trimmed = status.trim();
    if trimmed.is_empty() {
        "unspecified".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn market_context_allows_research(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    status: &MarketContextStatus,
) -> bool {
    if policy
        .market_context_status_policy
        .research_allows
        .iter()
        .any(|allowed| allowed == status.as_policy_key())
    {
        return true;
    }
    if matches!(status, MarketContextStatus::Pending) {
        let event_type = packet.event_type.as_policy_key();
        return policy
            .market_context_pending_policy
            .allow_research_candidate_for
            .iter()
            .any(|allowed| allowed == event_type)
            || (policy
                .market_context_pending_policy
                .allow_research_candidate_for
                .iter()
                .any(|allowed| allowed == "official")
                && packet
                    .source_independence_summary
                    .as_ref()
                    .is_some_and(|summary| summary.official_source_present));
    }
    false
}

fn effective_market_context_status(packet: &StructuredIntelPacket) -> MarketContextStatus {
    packet
        .market_context_ref
        .as_ref()
        .map(|reference| reference.status.clone())
        .unwrap_or_else(|| packet.market_context_status.clone())
}

fn forbidden_generated_terms(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
) -> Vec<String> {
    let terms: BTreeSet<String> = policy
        .forbidden_output_terms
        .iter()
        .map(|term| term.to_ascii_lowercase())
        .collect();
    let generated_text = [
        packet.topic_summary.as_str(),
        packet.stance_summary.as_str(),
        packet.risk_summary.as_str(),
        packet.regime_hint.as_str(),
        packet.scenario_hint.as_str(),
        packet.terminal_decision.as_str(),
    ]
    .join(" ");
    let tokens: BTreeSet<String> = generated_text
        .to_ascii_lowercase()
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    terms
        .into_iter()
        .filter(|term| tokens.contains(term))
        .collect()
}

fn is_medium_contradiction(flag: &ContradictionFlag) -> bool {
    matches!(
        flag,
        ContradictionFlag::SourceClaimConflict | ContradictionFlag::RumorVsOfficial
    )
}

fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for value in values {
        if seen.insert(value.clone()) {
            deduped.push(value);
        }
    }
    deduped
}

fn evidence_refs(packet: &StructuredIntelPacket) -> Vec<String> {
    let mut refs = Vec::new();
    refs.extend(packet.text_evidence.iter().map(|evidence| {
        format!(
            "text_evidence:{}:{}",
            evidence.source_event_id, evidence.source_id
        )
    }));
    refs.extend(packet.metric_evidence.iter().map(|evidence| {
        format!(
            "metric_evidence:{}:{}",
            evidence.source_event_id, evidence.metric_name
        )
    }));
    dedupe_strings(refs)
}

fn parent_artifact_ids(packet: &StructuredIntelPacket) -> Vec<String> {
    let mut ids = vec![packet.packet_id.clone(), packet.cluster_id.clone()];
    ids.extend(packet.source_event_ids.clone());
    dedupe_strings(ids)
}

fn validation_requirements(defaults: &ValidationRequirementDefaults) -> ValidationRequirements {
    ValidationRequirements {
        required_adapters: defaults.required_adapters.clone(),
        optional_adapters: defaults.optional_adapters.clone(),
        min_unseen_windows: defaults.min_unseen_windows,
        include_fee: defaults.include_fee,
        include_slippage: defaults.include_slippage,
        include_latency_assumption: defaults.include_latency_assumption,
        include_liquidity_filter: defaults.include_liquidity_filter,
        required_train_validation_split: defaults.required_train_validation_split,
        max_adapter_runtime_minutes: defaults.max_adapter_runtime_minutes,
    }
}

fn research_priority(event_type: &EventType, confidence_band: &ConfidenceBand) -> String {
    match (event_type, confidence_band) {
        (
            EventType::Incident | EventType::Regulatory,
            ConfidenceBand::High | ConfidenceBand::Strong,
        ) => "p0_event_risk".to_owned(),
        (EventType::FundingShift, ConfidenceBand::High | ConfidenceBand::Strong) => {
            "p1_derivatives".to_owned()
        }
        (_, ConfidenceBand::High | ConfidenceBand::Strong) => "p1_high_confidence".to_owned(),
        _ => "p2_standard".to_owned(),
    }
}

fn research_priority_partition(research_priority: &str) -> &str {
    match research_priority.split_once('_') {
        Some((partition, _)) if matches!(partition, "p0" | "p1" | "p2") => partition,
        _ => "p2",
    }
}

#[cfg(test)]
mod tests {
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
                && artifact.artifact_key.as_deref()
                    == Some("market-feature-delta-summary/summary.json")
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
            artifact.artifact_type == "market_feature_delta_summary"
                && artifact.known_as_of_ms == 2_000
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
}
