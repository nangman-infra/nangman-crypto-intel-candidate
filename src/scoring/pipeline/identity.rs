use super::super::*;

pub(super) fn candidate_id_for_admission(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    universe: Option<&SymbolUniverseSnapshot>,
    admission: &AdmissionState,
) -> Option<String> {
    if admission.has_valid_schema && admission.has_symbols {
        Some(candidate_id(packet, policy, universe))
    } else {
        None
    }
}

pub(super) fn idempotency_key(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    class: &CandidateClass,
    candidate_id: Option<&str>,
) -> String {
    stable_id(
        "cand_idem",
        &[
            packet.packet_id.as_str(),
            policy.policy_version.as_str(),
            class.as_policy_key(),
            candidate_id.unwrap_or("no_candidate_id"),
        ],
    )
}

pub(super) fn screening_event_id(packet: &StructuredIntelPacket, policy: &ScoringPolicy) -> String {
    stable_id(
        "cand_screen",
        &[packet.packet_id.as_str(), policy.policy_version.as_str()],
    )
}

pub(super) fn supersedes_screening_event_id(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
) -> Option<String> {
    packet
        .supersedes_packet_id
        .as_deref()
        .map(|packet_id| stable_id("cand_screen", &[packet_id, policy.policy_version.as_str()]))
}
