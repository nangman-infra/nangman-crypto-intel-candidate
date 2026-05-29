use crate::model::{MarketContextStatus, StructuredIntelPacket};
use crate::policy::ScoringPolicy;

pub(super) fn market_context_allows_research(
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

pub(in crate::scoring) fn effective_market_context_status(
    packet: &StructuredIntelPacket,
) -> MarketContextStatus {
    packet
        .market_context_ref
        .as_ref()
        .map(|reference| reference.status.clone())
        .unwrap_or_else(|| packet.market_context_status.clone())
}
