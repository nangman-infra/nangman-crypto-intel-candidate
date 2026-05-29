use super::*;

pub(in crate::scoring) fn classify_candidate(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    admission: &AdmissionState,
    score: &ScoreBreakdown,
    reasons: &mut Vec<String>,
) -> CandidateClass {
    if !admission.quarantine_reasons.is_empty() {
        return CandidateClass::Quarantine;
    }
    if !admission.reject_reasons.is_empty() {
        return CandidateClass::Reject;
    }

    let research_block_reasons = admission.research_block_reasons(packet);
    let strong_block_reasons = admission.strong_block_reasons();
    let desired = if score.final_score >= policy.thresholds.strong_candidate {
        CandidateClass::StrongCandidate
    } else if score.final_score >= policy.thresholds.research_candidate {
        CandidateClass::ResearchCandidate
    } else if score.final_score >= policy.thresholds.weak_candidate {
        CandidateClass::WeakCandidate
    } else {
        CandidateClass::ObserveOnly
    };

    let class = match desired {
        CandidateClass::StrongCandidate => {
            if strong_block_reasons.is_empty() && research_block_reasons.is_empty() {
                CandidateClass::StrongCandidate
            } else if research_block_reasons.is_empty() {
                reasons.extend(strong_block_reasons);
                CandidateClass::ResearchCandidate
            } else {
                reasons.extend(strong_block_reasons);
                reasons.extend(research_block_reasons);
                downgraded_non_research_class(packet, policy, score, admission)
            }
        }
        CandidateClass::ResearchCandidate => {
            if research_block_reasons.is_empty() {
                CandidateClass::ResearchCandidate
            } else {
                reasons.extend(research_block_reasons);
                downgraded_non_research_class(packet, policy, score, admission)
            }
        }
        CandidateClass::WeakCandidate => CandidateClass::WeakCandidate,
        CandidateClass::ObserveOnly => CandidateClass::ObserveOnly,
        CandidateClass::Reject | CandidateClass::Quarantine => desired,
    };

    let deduped = dedupe_strings(std::mem::take(reasons));
    reasons.extend(deduped);
    class
}

fn downgraded_non_research_class(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    score: &ScoreBreakdown,
    admission: &AdmissionState,
) -> CandidateClass {
    if admission.stale_market_context
        || matches!(
            packet.event_type,
            EventType::SocialHype | EventType::SocialBacklash
        )
    {
        return CandidateClass::WeakCandidate;
    }
    if score.final_score >= policy.thresholds.weak_candidate
        && admission.has_evidence
        && admission.has_lineage
    {
        CandidateClass::WeakCandidate
    } else {
        CandidateClass::ObserveOnly
    }
}
