use super::helpers::push_component;
use super::numeric::derivatives_numeric_baseline_resolved;
use super::*;

pub(super) fn push_contradiction_score_components(
    components: &mut Vec<ScoreComponent>,
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    admission: &AdmissionState,
) {
    for flag in &packet.contradiction_flags {
        if is_medium_contradiction(flag) {
            push_component(
                components,
                "contradiction_medium_penalty",
                policy.weight("contradiction_medium_penalty"),
                "medium contradiction flag",
            );
        } else {
            push_component(
                components,
                "contradiction_low_penalty",
                policy.weight("contradiction_low_penalty"),
                "low contradiction flag",
            );
        }
        if matches!(flag, ContradictionFlag::EvidenceWeak)
            && !derivatives_numeric_baseline_resolved(packet, admission)
        {
            push_component(
                components,
                "legacy_evidence_weak_penalty",
                policy.weight("legacy_evidence_weak_penalty"),
                "legacy evidence_weak flag",
            );
        }
    }
}
