use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::classification::CandidateClass;
use super::market_artifact::{DataQualitySummaryRef, SelectedMarketArtifactTrace};
use super::score::ScoreBreakdown;
use super::validation::ValidationRequirements;
use crate::model::market::MarketContextRef;
use crate::model::quality::{
    ContradictionFlag, MetricEvidence, SourceIndependenceSummary, SymbolResolutionTrace,
    TextEvidence,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct IntelCandidateEvidenceBundle {
    pub candidate_id: String,
    pub candidate_lifecycle_key: String,
    pub bundle_key: String,
    pub producer_app: String,
    pub producer_run_id: String,
    pub created_at_ms: i64,
    pub event_time_ms: i64,
    pub published_at_ms: Option<i64>,
    pub fetched_at_ms: i64,
    pub structured_at_ms: i64,
    pub candidate_created_at_ms: i64,
    pub decision_available_at_ms: i64,
    pub forbidden_lookahead_boundary_ms: i64,
    pub schema_version: String,
    pub scoring_policy_version: String,
    pub normalized_symbols: Vec<String>,
    pub input_packet_family_id: String,
    pub input_packet_revision: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes_packet_id: Option<String>,
    pub symbol_universe_snapshot_id: String,
    pub universe_as_of_ms: i64,
    pub approved_universe_symbol: bool,
    pub event_types: Vec<String>,
    pub hypothesis_type: String,
    pub allowed_horizons: Vec<String>,
    pub source_story_cluster_ids: Vec<String>,
    pub source_structured_packet_ids: Vec<String>,
    pub source_context_flag_packet_ids: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub text_evidence: Vec<TextEvidence>,
    pub metric_evidence: Vec<MetricEvidence>,
    pub market_context_ref: Option<MarketContextRef>,
    pub data_quality_summary: DataQualitySummaryRef,
    #[serde(default)]
    pub selected_market_artifacts: Vec<SelectedMarketArtifactTrace>,
    pub candidate_class: CandidateClass,
    pub candidate_score: i64,
    pub score_breakdown: ScoreBreakdown,
    pub research_priority: String,
    pub research_eligible: bool,
    pub validation_requirements: ValidationRequirements,
    pub source_independence: SourceIndependenceSummary,
    pub symbol_resolution_trace: Vec<SymbolResolutionTrace>,
    pub confidence_summary: BTreeMap<String, String>,
    pub contradiction_summary: Vec<ContradictionFlag>,
    pub observe_or_reject_reasons: Vec<String>,
    pub parent_artifact_ids: Vec<String>,
    pub storage_uri: String,
    pub checksum: String,
    pub idempotency_key: String,
}
