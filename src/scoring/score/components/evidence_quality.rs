use super::helpers::push_component;
use super::numeric::derivatives_numeric_baseline_resolved;
use crate::model::{EvidenceQualityReason, ScoreComponent, StructuredIntelPacket};
use crate::policy::ScoringPolicy;
use crate::scoring::admission::AdmissionState;

pub(super) fn push_evidence_quality_score_components(
    components: &mut Vec<ScoreComponent>,
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    admission: &AdmissionState,
) {
    for reason in &packet.evidence_quality_reasons {
        if evidence_quality_penalty_resolved(reason, packet, admission) {
            continue;
        }
        push_component(
            components,
            reason.as_policy_key(),
            policy.evidence_penalty(reason.as_policy_key()),
            "evidence quality reason",
        );
    }
}

fn evidence_quality_penalty_resolved(
    reason: &EvidenceQualityReason,
    packet: &StructuredIntelPacket,
    admission: &AdmissionState,
) -> bool {
    match reason {
        EvidenceQualityReason::BaselineMissing | EvidenceQualityReason::SingleNumericSnapshot => {
            derivatives_numeric_baseline_resolved(packet, admission)
        }
        EvidenceQualityReason::SingleSourceOnly => {
            derivatives_numeric_baseline_resolved(packet, admission)
                && packet
                    .source_independence_summary
                    .as_ref()
                    .is_some_and(|summary| summary.official_source_present)
        }
        _ => false,
    }
}
