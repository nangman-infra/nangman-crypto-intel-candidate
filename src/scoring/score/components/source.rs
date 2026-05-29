use super::helpers::push_component;
use super::*;

pub(super) fn push_source_score_components(
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
