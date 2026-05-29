use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CandidateRevisionIndex {
    pub schema_version: String,
    pub packet_family_id: String,
    #[serde(default)]
    pub scoring_policy_version: String,
    pub latest_packet_revision: u32,
    pub latest_packet_id: String,
    pub latest_screening_event_id: String,
    #[serde(default)]
    pub latest_candidate_id: Option<String>,
    pub updated_at_ms: i64,
}
