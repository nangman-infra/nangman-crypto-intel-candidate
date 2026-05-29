use super::super::*;
use super::identity::supersedes_screening_event_id;

pub(super) struct ScreeningEventInput<'a> {
    pub(super) screening_event_id: String,
    pub(super) packet: &'a StructuredIntelPacket,
    pub(super) policy: &'a ScoringPolicy,
    pub(super) created_at_ms: i64,
    pub(super) candidate_class: CandidateClass,
    pub(super) score_breakdown: ScoreBreakdown,
    pub(super) research_eligible: bool,
    pub(super) reasons: Vec<String>,
    pub(super) candidate_id: Option<&'a str>,
    pub(super) idempotency_key: String,
}

pub(super) fn build_screening_event(
    input: ScreeningEventInput<'_>,
) -> IntelCandidateScreeningEvent {
    let quarantine = matches!(input.candidate_class, CandidateClass::Quarantine);
    IntelCandidateScreeningEvent {
        screening_event_id: input.screening_event_id,
        schema_version: SCREENING_EVENT_SCHEMA_VERSION.to_owned(),
        producer_app: PRODUCER_APP.to_owned(),
        created_at_ms: input.created_at_ms,
        input_packet_id: input.packet.packet_id.clone(),
        input_packet_family_id: effective_packet_family_id(input.packet).to_owned(),
        input_packet_revision: input.packet.revision,
        source_structured_packet_ids: vec![input.packet.packet_id.clone()],
        supersedes_packet_id: input.packet.supersedes_packet_id.clone(),
        supersedes_screening_event_id: supersedes_screening_event_id(input.packet, input.policy),
        scoring_policy_version: input.policy.policy_version.clone(),
        candidate_score: input.score_breakdown.final_score,
        candidate_class: input.candidate_class,
        score_breakdown: input.score_breakdown,
        research_eligible: input.research_eligible,
        quarantine,
        reasons: input.reasons,
        candidate_id: if input.research_eligible {
            input.candidate_id.map(str::to_owned)
        } else {
            None
        },
        idempotency_key: input.idempotency_key,
    }
}
