use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ScoringPolicy {
    pub policy_version: String,
    pub schema_version: String,
    pub hard_gates: HardGates,
    pub thresholds: Thresholds,
    pub weights: BTreeMap<String, i64>,
    pub event_type_to_hypothesis_type: BTreeMap<String, String>,
    pub event_type_to_allowed_horizons: BTreeMap<String, Vec<String>>,
    pub market_context_pending_policy: MarketContextPendingPolicy,
    pub market_context_status_policy: MarketContextStatusPolicy,
    pub admission_requirements: AdmissionRequirements,
    pub evidence_quality_reason_penalties: BTreeMap<String, i64>,
    pub validation_requirement_defaults: ValidationRequirementDefaults,
    pub forbidden_output_terms: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct HardGates {
    pub forbid_missing_evidence: bool,
    pub forbid_missing_lineage: bool,
    pub forbid_invalid_schema: bool,
    pub forbid_forbidden_output_terms: bool,
    pub forbid_strong_when_market_context_pending: bool,
    pub forbid_strong_when_market_context_not_symbol_context: bool,
    pub forbid_strong_when_contradiction_medium_or_high: bool,
    pub forbid_strong_when_social_only: bool,
    pub require_decision_available_at_ms: bool,
    pub require_point_in_time_universe: bool,
    pub require_approved_universe_for_research: bool,
    pub require_data_quality_summary_for_research: bool,
    #[serde(default = "default_true")]
    pub require_market_feature_delta_for_research: bool,
    #[serde(default = "default_true")]
    pub require_market_regime_context_for_research: bool,
    pub require_source_independence_for_research: bool,
    pub require_symbol_resolution_trace_for_research: bool,
    pub forbid_source_event_count_as_diversity: bool,
    pub forbid_research_without_metric_delta_for_derivatives: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Thresholds {
    pub strong_candidate: i64,
    pub research_candidate: i64,
    pub weak_candidate: i64,
    pub observe_only_below: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MarketContextPendingPolicy {
    pub default_class: String,
    pub allow_research_candidate_for: Vec<String>,
    pub research_candidate_still_requires: Vec<String>,
    pub observe_only_by_default_for: Vec<String>,
    pub max_class_for_social_hype: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MarketContextStatusPolicy {
    pub strong_requires: String,
    pub research_allows: Vec<String>,
    pub pending_resolution: String,
    pub observe_only_by_default_for: Vec<String>,
    pub max_class_for_stale_but_usable: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AdmissionRequirements {
    pub strong_requires_approved_universe_symbol: bool,
    pub research_requires_approved_universe_symbol: bool,
    pub strong_min_independent_source_count: usize,
    pub research_min_independent_source_count: usize,
    pub official_source_can_replace_min_independent_source_count: bool,
    pub strong_allowed_symbol_mapping_confidence: Vec<String>,
    pub research_allowed_symbol_mapping_confidence: Vec<String>,
    pub derivatives_requires_metric_delta_for_research: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ValidationRequirementDefaults {
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

impl ScoringPolicy {
    pub fn weight(&self, key: &str) -> i64 {
        self.weights.get(key).copied().unwrap_or(0)
    }

    pub fn evidence_penalty(&self, key: &str) -> i64 {
        self.evidence_quality_reason_penalties
            .get(key)
            .copied()
            .unwrap_or(0)
    }
}

fn default_true() -> bool {
    true
}
