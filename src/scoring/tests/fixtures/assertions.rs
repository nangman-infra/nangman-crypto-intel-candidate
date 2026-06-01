use crate::model::CandidateProcessingResult;

pub(in crate::scoring::tests) fn assert_blocked_with_reason(
    result: &CandidateProcessingResult,
    expected_reason: &str,
) {
    assert!(!result.screening_event.research_eligible);
    assert!(result.evidence_bundle.is_none());
    assert!(
        result
            .screening_event
            .reasons
            .iter()
            .any(|reason| reason == expected_reason),
        "expected reason {expected_reason}, got {:?}",
        result.screening_event.reasons
    );
}
