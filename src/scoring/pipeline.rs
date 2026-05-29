use super::*;

mod hypothesis_state;
mod identity;
mod research;
mod screening;

use hypothesis_state::{HypothesisStateInput, build_hypothesis_state_for_candidate};
use identity::{candidate_id_for_admission, idempotency_key, screening_event_id};
use research::{ResearchArtifactInput, build_research_artifacts};
use screening::{ScreeningEventInput, build_screening_event};

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
    let mut reasons = collect_admission_reasons(&admission);
    let candidate_id = candidate_id_for_admission(&packet, policy, universe, &admission);
    let mut class = classify_candidate(&packet, policy, &admission, &score, &mut reasons);
    let selected_market_artifacts = admission.selected_market_artifacts();
    let idempotency_key = idempotency_key(&packet, policy, &class, candidate_id.as_deref());

    let research_artifacts = build_research_artifacts(ResearchArtifactInput {
        packet: &packet,
        policy,
        universe,
        candidate_id: candidate_id.as_deref(),
        created_at_ms,
        candidate_class: class,
        score_breakdown: score.clone(),
        reasons: &reasons,
        selected_market_artifacts: selected_market_artifacts.clone(),
        idempotency_key: idempotency_key.clone(),
    });
    class = research_artifacts.candidate_class;
    let research_eligible = research_artifacts.research_eligible;
    reasons.extend(research_artifacts.additional_reasons);

    let screening_event_id = screening_event_id(&packet, policy);
    let hypothesis_result = build_hypothesis_state_for_candidate(HypothesisStateInput {
        packet: &packet,
        policy,
        created_at_ms,
        candidate_id: candidate_id.as_deref(),
        candidate_class: class.clone(),
        score_breakdown: score.clone(),
        reasons: &reasons,
        selected_market_artifacts,
        idempotency_key: idempotency_key.clone(),
        screening_event_id: screening_event_id.clone(),
        research_eligible,
    });
    reasons.extend(hypothesis_result.additional_reasons);

    let screening_event = build_screening_event(ScreeningEventInput {
        screening_event_id,
        packet: &packet,
        policy,
        created_at_ms,
        candidate_class: class,
        score_breakdown: score,
        research_eligible,
        reasons,
        candidate_id: candidate_id.as_deref(),
        idempotency_key,
    });

    CandidateProcessingResult {
        screening_event,
        evidence_bundle: research_artifacts.evidence_bundle,
        hypothesis_state: hypothesis_result.hypothesis_state,
    }
}

fn collect_admission_reasons(admission: &AdmissionState) -> Vec<String> {
    let mut reasons = Vec::new();
    reasons.extend(admission.quarantine_reasons.clone());
    reasons.extend(admission.reject_reasons.clone());
    reasons.extend(admission.observe_reasons.clone());
    reasons
}
