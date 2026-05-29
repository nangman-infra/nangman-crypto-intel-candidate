use serde::{Deserialize, Serialize};

use super::classification::CandidateClass;
use super::score::ScoreBreakdown;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct IntelCandidateScreeningEvent {
    pub screening_event_id: String,
    pub schema_version: String,
    pub producer_app: String,
    pub created_at_ms: i64,
    pub input_packet_id: String,
    pub input_packet_family_id: String,
    pub input_packet_revision: u32,
    pub source_structured_packet_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes_packet_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes_screening_event_id: Option<String>,
    pub scoring_policy_version: String,
    pub candidate_class: CandidateClass,
    pub candidate_score: i64,
    pub score_breakdown: ScoreBreakdown,
    pub research_eligible: bool,
    pub quarantine: bool,
    pub reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_id: Option<String>,
    pub idempotency_key: String,
}
