use super::*;

pub(super) struct HypothesisStateBuildContext<'a> {
    pub(super) hypothesis_id: &'a str,
    pub(super) packet: &'a StructuredIntelPacket,
    pub(super) policy: &'a ScoringPolicy,
    pub(super) created_at_ms: i64,
    pub(super) candidate_class: CandidateClass,
    pub(super) score_breakdown: ScoreBreakdown,
    pub(super) reasons: Vec<String>,
    pub(super) selected_market_artifacts: Vec<SelectedMarketArtifactTrace>,
    pub(super) idempotency_key: String,
    pub(super) screening_event_id: String,
}

pub(super) fn build_hypothesis_state(
    context: HypothesisStateBuildContext<'_>,
) -> Result<IntelCandidateHypothesisState, serde_json::Error> {
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
    let checksum_payload = serde_json::to_vec(&state)?;
    state.checksum = sha256_hex(checksum_payload);
    Ok(state)
}
