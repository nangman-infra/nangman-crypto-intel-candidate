use serde::{Deserialize, Serialize};

use super::market::MarketContextRef;
use super::quality::{
    ConfidenceBand, ContradictionFlag, EventType, EvidenceQualityReason, MarketContextStatus,
    MetricEvidence, SourceIndependenceSummary, SymbolResolutionTrace, TextEvidence,
    TimeRelevanceWindow,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct StructuredIntelPacket {
    pub packet_id: String,
    #[serde(default)]
    pub packet_family_id: String,
    #[serde(default)]
    pub raw_event_id: String,
    #[serde(default)]
    pub event_timestamp_ms: i64,
    #[serde(default)]
    pub revision: u32,
    #[serde(default)]
    pub supersedes_packet_id: Option<String>,
    pub cluster_id: String,
    #[serde(default)]
    pub source_event_ids: Vec<String>,
    #[serde(default)]
    pub published_at_ms: Option<i64>,
    #[serde(default)]
    pub fetched_at_ms: Option<i64>,
    #[serde(default)]
    pub structured_at_ms: Option<i64>,
    #[serde(default)]
    pub decision_available_at_ms: Option<i64>,
    #[serde(default)]
    pub normalized_symbols: Vec<String>,
    #[serde(default)]
    pub symbol_confidence_band: ConfidenceBand,
    #[serde(default)]
    pub symbol_resolution_trace: Vec<SymbolResolutionTrace>,
    #[serde(default)]
    pub event_type: EventType,
    #[serde(default)]
    pub topic_summary: String,
    #[serde(default)]
    pub stance_summary: String,
    #[serde(default)]
    pub risk_summary: String,
    #[serde(default)]
    pub regime_hint: String,
    #[serde(default)]
    pub scenario_hint: String,
    #[serde(default)]
    pub confidence_band: ConfidenceBand,
    #[serde(default)]
    pub novelty_score: f64,
    #[serde(default)]
    pub time_relevance_window: Option<TimeRelevanceWindow>,
    #[serde(default)]
    pub contradiction_flags: Vec<ContradictionFlag>,
    #[serde(default)]
    pub source_quality_summary: String,
    #[serde(default)]
    pub source_independence_summary: Option<SourceIndependenceSummary>,
    #[serde(default)]
    pub text_evidence: Vec<TextEvidence>,
    #[serde(default)]
    pub metric_evidence: Vec<MetricEvidence>,
    #[serde(default)]
    pub evidence_quality_reasons: Vec<EvidenceQualityReason>,
    #[serde(default)]
    pub market_context_status: MarketContextStatus,
    #[serde(default)]
    pub market_context_retry_after_ms: Option<i64>,
    #[serde(default)]
    pub market_context_expire_at_ms: Option<i64>,
    #[serde(default)]
    pub market_context_terminal_reason: Option<String>,
    #[serde(default)]
    pub market_context_ref: Option<MarketContextRef>,
    #[serde(default)]
    pub model_tier_used: String,
    #[serde(default)]
    pub terminal_decision: String,
    #[serde(default)]
    pub evidence_sentences: Vec<String>,
    #[serde(default)]
    pub schema_version: Option<String>,
}
