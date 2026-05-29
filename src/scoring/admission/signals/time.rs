use crate::model::StructuredIntelPacket;

pub(super) fn has_required_replay_times(packet: &StructuredIntelPacket) -> bool {
    packet.fetched_at_ms.is_some()
        && packet.structured_at_ms.is_some()
        && packet.decision_available_at_ms.is_some()
}

pub(super) fn has_valid_replay_time_order(packet: &StructuredIntelPacket) -> bool {
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

pub(super) fn market_artifact_cutoff_ms(
    packet: &StructuredIntelPacket,
    candidate_created_at_ms: i64,
) -> Option<i64> {
    packet
        .decision_available_at_ms
        .map(|decision_available_at_ms| decision_available_at_ms.max(candidate_created_at_ms))
}
