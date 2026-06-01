use super::*;

#[test]
fn revision_index_key_is_scoped_by_scoring_policy() {
    assert_eq!(
        revision_index_key("family/001", "intel_candidate_scoring_v2", 1),
        "candidate-revision-index/schema=intel_candidate_revision_index_v1/packet_family_id=family_001/scoring_policy=intel_candidate_scoring_v2/revision=0000000001.json"
    );
}
