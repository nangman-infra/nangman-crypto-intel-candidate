use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const STRUCTURED_PACKET_SCHEMA_VERSION: &str = "structured_intel_packet_v1";
pub const STRUCTURED_POINTER_SCHEMA_VERSION: &str = "structured_pointer_v1";
pub const CANDIDATE_POINTER_SCHEMA_VERSION: &str = "intel_candidate_pointer_v1";
pub const CANDIDATE_BUNDLE_SCHEMA_VERSION: &str = "intel_candidate_evidence_bundle_v1";
pub const SCREENING_EVENT_SCHEMA_VERSION: &str = "intel_candidate_screening_event_v1";
pub const HYPOTHESIS_STATE_SCHEMA_VERSION: &str = "intel_candidate_hypothesis_state_v1";
pub const PRODUCER_APP: &str = "intel-candidate-app";
pub const CANDIDATE_REVISION_INDEX_SCHEMA_VERSION: &str = "intel_candidate_revision_index_v1";

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SymbolResolutionTrace {
    #[serde(default)]
    pub raw_mentions: Vec<String>,
    #[serde(default)]
    pub resolved_project: Option<String>,
    #[serde(default)]
    pub resolved_asset: Option<String>,
    #[serde(default)]
    pub canonical_symbol: Option<String>,
    #[serde(default)]
    pub venue_symbols: Vec<String>,
    #[serde(default)]
    pub mapping_confidence: ConfidenceBand,
    #[serde(default)]
    pub ambiguity_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SourceIndependenceSummary {
    pub source_event_count: usize,
    pub independent_source_count: usize,
    pub official_source_present: bool,
    #[serde(default)]
    pub duplicate_content_hashes: Vec<String>,
    #[serde(default)]
    pub syndicated_from: Option<String>,
    #[serde(default)]
    pub original_source_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TextEvidence {
    pub evidence_text: String,
    pub source_event_id: String,
    pub source_id: String,
    #[serde(default)]
    pub published_at_ms: Option<i64>,
    #[serde(default)]
    pub evidence_kind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MetricEvidence {
    pub metric_name: String,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    pub value: Option<f64>,
    #[serde(default)]
    pub previous_value: Option<f64>,
    #[serde(default)]
    pub delta_pct: Option<f64>,
    #[serde(default)]
    pub window_ms: Option<i64>,
    pub observed_at_ms: i64,
    pub source_event_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MarketContextRef {
    pub status: MarketContextStatus,
    #[serde(default)]
    pub basis_timestamp_ms: Option<i64>,
    #[serde(default)]
    pub basis_kind: String,
    #[serde(default)]
    pub window_start_ms: Option<i64>,
    #[serde(default)]
    pub window_end_ms: Option<i64>,
    #[serde(default)]
    pub manifest_key: Option<String>,
    #[serde(default)]
    pub output_object_keys: Vec<String>,
    #[serde(default)]
    pub market_data_quality_summary_key: Option<String>,
    #[serde(default)]
    pub market_feature_delta_key: Option<String>,
    #[serde(default)]
    pub market_feature_delta_summary_key: Option<String>,
    #[serde(default)]
    pub market_regime_context_key: Option<String>,
    #[serde(default)]
    pub symbol_universe_snapshot_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MarketFeatureDelta {
    pub schema_version: String,
    pub feature_delta_id: String,
    pub l1_run_id: String,
    pub metric_name: String,
    pub venue: String,
    pub symbol_native: String,
    pub symbol_canonical: String,
    pub market_type: String,
    pub value_now: f64,
    #[serde(default)]
    pub value_15m_ago: Option<f64>,
    #[serde(default)]
    pub value_1h_ago: Option<f64>,
    #[serde(default)]
    pub change_pct_15m: Option<f64>,
    #[serde(default)]
    pub change_pct_1h: Option<f64>,
    #[serde(default)]
    pub price_change_same_window: Option<f64>,
    #[serde(default)]
    pub volume_change_same_window: Option<f64>,
    #[serde(default)]
    pub oi_price_divergence: Option<f64>,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub known_as_of_ms: i64,
    #[serde(default)]
    pub quality_status: String,
    #[serde(default)]
    pub missing_reasons: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MarketFeatureDeltaSummary {
    pub schema_version: String,
    pub feature_delta_summary_id: String,
    pub l1_run_id: String,
    pub detail_feature_delta_key: String,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub known_as_of_ms: i64,
    pub detail_record_count: usize,
    pub summary_row_count: usize,
    #[serde(default)]
    pub rows: Vec<MarketFeatureDeltaSummaryRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MarketFeatureDeltaSummaryRow {
    pub venue: String,
    pub symbol_native: String,
    pub symbol_canonical: String,
    pub market_type: String,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub known_as_of_ms: i64,
    #[serde(default)]
    pub quality_status: String,
    #[serde(default)]
    pub missing_reasons: Vec<String>,
    #[serde(default)]
    pub metrics: Vec<MarketFeatureDeltaSummaryMetric>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MarketFeatureDeltaSummaryMetric {
    pub metric_name: String,
    pub value_now: f64,
    #[serde(default)]
    pub value_15m_ago: Option<f64>,
    #[serde(default)]
    pub value_1h_ago: Option<f64>,
    #[serde(default)]
    pub change_pct_15m: Option<f64>,
    #[serde(default)]
    pub change_pct_1h: Option<f64>,
    #[serde(default)]
    pub price_change_same_window: Option<f64>,
    #[serde(default)]
    pub volume_change_same_window: Option<f64>,
    #[serde(default)]
    pub oi_price_divergence: Option<f64>,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    #[serde(default)]
    pub quality_status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MarketRegimeContext {
    pub schema_version: String,
    pub regime_context_id: String,
    pub l1_run_id: String,
    pub scope: String,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    #[serde(default)]
    pub btc_return_same_window: Option<f64>,
    #[serde(default)]
    pub eth_return_same_window: Option<f64>,
    #[serde(default)]
    pub sector_return_same_window: Option<f64>,
    pub volatility_regime: String,
    #[serde(default)]
    pub correlation_to_btc: Option<f64>,
    pub known_as_of_ms: i64,
    #[serde(default)]
    pub quality_status: String,
    #[serde(default)]
    pub missing_reasons: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TimeRelevanceWindow {
    pub start_ms: i64,
    pub end_ms: i64,
    pub relevance_decay_hint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceBand {
    Weak,
    Low,
    Moderate,
    Medium,
    Strong,
    High,
    #[default]
    Unknown,
}

impl ConfidenceBand {
    pub fn as_policy_key(&self) -> &'static str {
        match self {
            Self::Strong | Self::High => "strong",
            Self::Moderate | Self::Medium => "moderate",
            Self::Weak | Self::Low | Self::Unknown => "weak",
        }
    }

    pub fn is_research_allowed(&self) -> bool {
        matches!(
            self,
            Self::Moderate | Self::Medium | Self::Strong | Self::High
        )
    }

    pub fn is_strong(&self) -> bool {
        matches!(self, Self::Strong | Self::High)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Listing,
    ExchangeListing,
    Delisting,
    ExchangeDelisting,
    DepositWithdrawal,
    Incident,
    Partnership,
    ProjectNotice,
    TokenUnlock,
    Governance,
    FundingShift,
    MacroEvent,
    Regulatory,
    SocialBacklash,
    SocialHype,
    #[default]
    Other,
}

impl EventType {
    pub fn as_policy_key(&self) -> &'static str {
        match self {
            Self::Listing => "listing",
            Self::ExchangeListing => "exchange_listing",
            Self::Delisting => "delisting",
            Self::ExchangeDelisting => "exchange_delisting",
            Self::DepositWithdrawal => "deposit_withdrawal",
            Self::Incident => "incident",
            Self::Partnership => "partnership",
            Self::ProjectNotice => "project_notice",
            Self::TokenUnlock => "token_unlock",
            Self::Governance => "governance",
            Self::FundingShift => "funding_shift",
            Self::MacroEvent => "macro_event",
            Self::Regulatory => "regulatory",
            Self::SocialBacklash => "social_backlash",
            Self::SocialHype => "social_hype",
            Self::Other => "other",
        }
    }

    pub fn is_derivatives_like(&self) -> bool {
        matches!(self, Self::FundingShift)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionFlag {
    TimeMismatch,
    SymbolAmbiguity,
    SourceClaimConflict,
    RumorVsOfficial,
    TitleBodyMismatch,
    EvidenceWeak,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceQualityReason {
    BaselineMissing,
    SingleNumericSnapshot,
    SingleSourceOnly,
    TitleOnly,
    SymbolAmbiguous,
    MarketContextMissing,
    DuplicateOrSyndicatedSource,
    ApprovedUniverseMissing,
}

impl EvidenceQualityReason {
    pub fn as_policy_key(&self) -> &'static str {
        match self {
            Self::BaselineMissing => "baseline_missing",
            Self::SingleNumericSnapshot => "single_numeric_snapshot",
            Self::SingleSourceOnly => "single_source_only",
            Self::TitleOnly => "title_only",
            Self::SymbolAmbiguous => "symbol_ambiguous",
            Self::MarketContextMissing => "market_context_missing",
            Self::DuplicateOrSyndicatedSource => "duplicate_or_syndicated_source",
            Self::ApprovedUniverseMissing => "approved_universe_missing",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum MarketContextStatus {
    Available,
    AvailableSymbolContext,
    AvailableGeneralContext,
    NearestAvailable,
    SymbolContextOnly,
    StaleButUsable,
    Pending,
    Unavailable,
    #[default]
    Unknown,
}

impl MarketContextStatus {
    pub fn as_policy_key(&self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::AvailableSymbolContext => "available_symbol_context",
            Self::AvailableGeneralContext => "available_general_context",
            Self::NearestAvailable => "nearest_available",
            Self::SymbolContextOnly => "symbol_context_only",
            Self::StaleButUsable => "stale_but_usable",
            Self::Pending => "pending",
            Self::Unavailable => "unavailable",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SymbolUniverseSnapshot {
    pub schema_version: String,
    pub symbol_universe_snapshot_id: String,
    pub universe_as_of_ms: i64,
    #[serde(default)]
    pub included_symbols: Vec<SymbolUniverseMember>,
    #[serde(default)]
    pub excluded_symbols: Vec<SymbolUniverseMember>,
    #[serde(default)]
    pub liquidity_rank_at_that_time: Vec<SymbolLiquidityRank>,
    pub selection_policy_version: String,
    pub venue_truth_policy_version: String,
    pub data_quality_cutoff_version: String,
    pub generated_at_ms: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SymbolUniverseMember {
    pub symbol_canonical: String,
    #[serde(default)]
    pub execution_symbol_native: Option<String>,
    #[serde(default)]
    pub reference_symbol_native: Option<String>,
    #[serde(default)]
    pub liquidity_rank_at_that_time: Option<i64>,
    pub approved_universe_symbol: bool,
    pub bootstrap_days_available: i64,
    #[serde(default)]
    pub median_spread_bps_30d: Option<f64>,
    #[serde(default)]
    pub median_traded_notional_30d: Option<f64>,
    #[serde(default)]
    pub gap_rate_30d: Option<f64>,
    pub mapping_confidence: String,
    pub status_reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SymbolLiquidityRank {
    pub symbol_canonical: String,
    pub liquidity_rank_at_that_time: i64,
    pub observed_traded_notional: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateClass {
    StrongCandidate,
    ResearchCandidate,
    WeakCandidate,
    ObserveOnly,
    Reject,
    Quarantine,
}

impl CandidateClass {
    pub fn is_research_eligible(&self) -> bool {
        matches!(self, Self::StrongCandidate | Self::ResearchCandidate)
    }

    pub fn as_policy_key(&self) -> &'static str {
        match self {
            Self::StrongCandidate => "strong_candidate",
            Self::ResearchCandidate => "research_candidate",
            Self::WeakCandidate => "weak_candidate",
            Self::ObserveOnly => "observe_only",
            Self::Reject => "reject",
            Self::Quarantine => "quarantine",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ScoreComponent {
    pub name: String,
    pub value: i64,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ScoreBreakdown {
    pub components: Vec<ScoreComponent>,
    pub final_score: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DataQualitySummaryRef {
    pub market_data_quality_summary_key: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SelectedMarketArtifactTrace {
    pub artifact_type: String,
    pub artifact_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub l1_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol_canonical: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub known_as_of_ms: i64,
    pub quality_status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ValidationRequirements {
    pub required_adapters: Vec<String>,
    pub optional_adapters: Vec<String>,
    pub min_unseen_windows: usize,
    pub include_fee: bool,
    pub include_slippage: bool,
    pub include_latency_assumption: bool,
    pub include_liquidity_filter: bool,
    pub required_train_validation_split: bool,
    pub max_adapter_runtime_minutes: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CandidateProcessingResult {
    pub screening_event: IntelCandidateScreeningEvent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_bundle: Option<IntelCandidateEvidenceBundle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hypothesis_state: Option<IntelCandidateHypothesisState>,
}

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CandidateRevisionIndex {
    pub schema_version: String,
    pub packet_family_id: String,
    pub latest_packet_revision: u32,
    pub latest_packet_id: String,
    pub latest_screening_event_id: String,
    #[serde(default)]
    pub latest_candidate_id: Option<String>,
    pub updated_at_ms: i64,
}
