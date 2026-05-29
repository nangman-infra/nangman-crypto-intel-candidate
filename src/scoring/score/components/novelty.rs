use super::helpers::push_component;
use super::*;

pub(super) fn push_novelty_score_component(
    components: &mut Vec<ScoreComponent>,
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
) {
    if packet.novelty_score >= 0.7 {
        push_component(
            components,
            "novelty_high",
            policy.weight("novelty_high"),
            "novelty score >= 0.7",
        );
    } else if packet.novelty_score >= 0.4 {
        push_component(
            components,
            "novelty_medium",
            policy.weight("novelty_medium"),
            "novelty score >= 0.4",
        );
    }
}
