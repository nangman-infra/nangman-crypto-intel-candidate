use serde::{Deserialize, Serialize};

use super::classification::CandidateClass;
use super::market_artifact::SelectedMarketArtifactTrace;
use super::score::ScoreBreakdown;
use crate::model::market::MarketContextRef;
use crate::model::quality::{EvidenceQualityReason, MarketContextStatus};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct IntelCandidateHypothesisState {
    pub hypothesis_id: String,
    pub state_key: String,
    pub schema_version: String,
    pub producer_app: String,
    pub producer_version: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub input_packet_id: String,
    pub input_packet_family_id: String,
    pub input_packet_revision: u32,
    pub source_structured_packet_ids: Vec<String>,
    pub source_event_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes_packet_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes_hypothesis_id: Option<String>,
    pub latest_screening_event_id: String,
    pub scoring_policy_version: String,
    pub normalized_symbols: Vec<String>,
    pub event_type: String,
    pub hypothesis_type: String,
    pub current_state: CandidateClass,
    pub current_score: i64,
    pub previous_score: Option<i64>,
    pub research_eligible: bool,
    pub transition: String,
    pub next_action: String,
    pub reasons: Vec<String>,
    pub retryable_reasons: Vec<String>,
    pub terminal_reasons: Vec<String>,
    pub selected_market_artifacts: Vec<SelectedMarketArtifactTrace>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market_context_ref: Option<MarketContextRef>,
    pub market_context_status: MarketContextStatus,
    pub evidence_quality_reasons: Vec<EvidenceQualityReason>,
    pub score_breakdown: ScoreBreakdown,
    pub lineage_refs: Vec<String>,
    pub dirty_triggers: Vec<String>,
    pub harness_queue_hint: String,
    pub idempotency_key: String,
    pub checksum: String,
}
