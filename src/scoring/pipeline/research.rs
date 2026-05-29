use super::super::*;

pub(super) struct ResearchArtifactInput<'a> {
    pub(super) packet: &'a StructuredIntelPacket,
    pub(super) policy: &'a ScoringPolicy,
    pub(super) universe: Option<&'a SymbolUniverseSnapshot>,
    pub(super) candidate_id: Option<&'a str>,
    pub(super) created_at_ms: i64,
    pub(super) candidate_class: CandidateClass,
    pub(super) score_breakdown: ScoreBreakdown,
    pub(super) reasons: &'a [String],
    pub(super) selected_market_artifacts: Vec<SelectedMarketArtifactTrace>,
    pub(super) idempotency_key: String,
}

pub(super) struct ResearchArtifactResult {
    pub(super) candidate_class: CandidateClass,
    pub(super) research_eligible: bool,
    pub(super) evidence_bundle: Option<IntelCandidateEvidenceBundle>,
    pub(super) additional_reasons: Vec<String>,
}

pub(super) fn build_research_artifacts(input: ResearchArtifactInput<'_>) -> ResearchArtifactResult {
    let mut result = ResearchArtifactResult {
        research_eligible: input.candidate_class.is_research_eligible(),
        candidate_class: input.candidate_class,
        evidence_bundle: None,
        additional_reasons: Vec::new(),
    };
    if !result.research_eligible {
        return result;
    }

    let Some(candidate_id) = input.candidate_id else {
        block_research_bundle(
            &mut result,
            input.packet,
            input.universe,
            input.candidate_id,
        );
        return result;
    };
    let Some(research_inputs) = research_bundle_inputs(input.packet, input.universe) else {
        block_research_bundle(
            &mut result,
            input.packet,
            input.universe,
            input.candidate_id,
        );
        return result;
    };

    match build_evidence_bundle(BundleBuildContext {
        candidate_id,
        packet: input.packet,
        policy: input.policy,
        research_inputs,
        created_at_ms: input.created_at_ms,
        candidate_class: result.candidate_class.clone(),
        score_breakdown: input.score_breakdown,
        reasons: input.reasons.to_vec(),
        selected_market_artifacts: input.selected_market_artifacts,
        idempotency_key: input.idempotency_key,
    }) {
        Ok(bundle) => {
            result.evidence_bundle = Some(bundle);
        }
        Err(error) => {
            result
                .additional_reasons
                .push(format!("candidate_bundle_checksum_failed:{error}"));
            result.candidate_class = CandidateClass::ObserveOnly;
            result.research_eligible = false;
        }
    }
    result
}

fn block_research_bundle(
    result: &mut ResearchArtifactResult,
    packet: &StructuredIntelPacket,
    universe: Option<&SymbolUniverseSnapshot>,
    candidate_id: Option<&str>,
) {
    result
        .additional_reasons
        .extend(research_bundle_block_reasons(
            packet,
            universe,
            candidate_id,
        ));
    result.candidate_class = CandidateClass::ObserveOnly;
    result.research_eligible = false;
}
