use super::priority::{research_priority, research_priority_partition};
use crate::hash::stable_id;
use crate::model::StructuredIntelPacket;
use crate::policy::ScoringPolicy;
use crate::scoring::helpers::candidate_bundle_key;

pub(super) struct BundleIdentity {
    pub(super) candidate_lifecycle_key: String,
    pub(super) bundle_key: String,
    pub(super) hypothesis_type: String,
    pub(super) allowed_horizons: Vec<String>,
    pub(super) research_priority: String,
}

pub(super) fn bundle_identity(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    candidate_id: &str,
    created_at_ms: i64,
) -> BundleIdentity {
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
    let research_priority = research_priority(&packet.event_type, &packet.confidence_band);
    let priority_partition = research_priority_partition(&research_priority);

    BundleIdentity {
        candidate_lifecycle_key: candidate_lifecycle_key(
            packet,
            &hypothesis_type,
            &allowed_horizons,
        ),
        bundle_key: candidate_bundle_key(created_at_ms, candidate_id, priority_partition),
        hypothesis_type,
        allowed_horizons,
        research_priority,
    }
}

fn candidate_lifecycle_key(
    packet: &StructuredIntelPacket,
    hypothesis_type: &str,
    allowed_horizons: &[String],
) -> String {
    let horizon_key = allowed_horizons.join(",");
    stable_id(
        "cand_life",
        &[
            packet
                .normalized_symbols
                .first()
                .map(String::as_str)
                .unwrap_or("unknown_symbol"),
            hypothesis_type,
            packet.event_type.as_policy_key(),
            horizon_key.as_str(),
        ],
    )
}
