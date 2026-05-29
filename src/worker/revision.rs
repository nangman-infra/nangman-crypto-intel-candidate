use super::*;

pub(super) const REVISION_INDEX_MAX_KEYS: usize = 256;

pub(super) fn revision_index_prefix(
    packet_family_id: &str,
    scoring_policy_version: &str,
) -> String {
    format!(
        "candidate-revision-index/schema={}/packet_family_id={}/scoring_policy={}/",
        CANDIDATE_REVISION_INDEX_SCHEMA_VERSION,
        path_segment(packet_family_id),
        path_segment(scoring_policy_version)
    )
}

pub(super) fn revision_index_key(
    packet_family_id: &str,
    scoring_policy_version: &str,
    revision: u32,
) -> String {
    format!(
        "{}revision={:010}.json",
        revision_index_prefix(packet_family_id, scoring_policy_version),
        revision
    )
}

pub(super) fn parse_revision_from_key(key: &str) -> Option<u32> {
    key.strip_suffix(".json")?
        .rsplit_once("revision=")?
        .1
        .parse()
        .ok()
}
