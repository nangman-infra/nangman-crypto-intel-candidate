use super::*;

#[test]
fn candidate_bundle_key_uses_coarse_priority_partition() {
    assert!(
        candidate_bundle_key(7_200_000, "cand_001", "p0").starts_with(
            "candidate-evidence-bundle/priority=p0/schema=intel_candidate_evidence_bundle_v1/"
        )
    );
    assert_eq!(research_priority_partition("p0_event_risk"), "p0");
    assert_eq!(research_priority_partition("p1_high_confidence"), "p1");
    assert_eq!(research_priority_partition("p2_standard"), "p2");
    assert_eq!(research_priority_partition("unknown"), "p2");
}
