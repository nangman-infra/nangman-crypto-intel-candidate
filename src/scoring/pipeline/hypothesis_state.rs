use super::super::*;

pub(super) struct HypothesisStateInput<'a> {
    pub(super) packet: &'a StructuredIntelPacket,
    pub(super) policy: &'a ScoringPolicy,
    pub(super) created_at_ms: i64,
    pub(super) candidate_id: Option<&'a str>,
    pub(super) candidate_class: CandidateClass,
    pub(super) score_breakdown: ScoreBreakdown,
    pub(super) reasons: &'a [String],
    pub(super) selected_market_artifacts: Vec<SelectedMarketArtifactTrace>,
    pub(super) idempotency_key: String,
    pub(super) screening_event_id: String,
    pub(super) research_eligible: bool,
}

pub(super) struct HypothesisStateResult {
    pub(super) hypothesis_state: Option<IntelCandidateHypothesisState>,
    pub(super) additional_reasons: Vec<String>,
}

pub(super) fn build_hypothesis_state_for_candidate(
    input: HypothesisStateInput<'_>,
) -> HypothesisStateResult {
    let mut result = HypothesisStateResult {
        hypothesis_state: None,
        additional_reasons: Vec::new(),
    };
    if input.research_eligible || matches!(input.candidate_class, CandidateClass::Quarantine) {
        return result;
    }

    let Some(hypothesis_id) = input.candidate_id else {
        return result;
    };
    match build_hypothesis_state(HypothesisStateBuildContext {
        hypothesis_id,
        packet: input.packet,
        policy: input.policy,
        created_at_ms: input.created_at_ms,
        candidate_class: input.candidate_class,
        score_breakdown: input.score_breakdown,
        reasons: input.reasons.to_vec(),
        selected_market_artifacts: input.selected_market_artifacts,
        idempotency_key: input.idempotency_key,
        screening_event_id: input.screening_event_id,
    }) {
        Ok(state) => {
            result.hypothesis_state = Some(state);
        }
        Err(error) => {
            result
                .additional_reasons
                .push(format!("hypothesis_state_checksum_failed:{error}"));
        }
    }
    result
}
