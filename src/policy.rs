use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

pub const DEFAULT_POLICY_PATH: &str =
    "/opt/nangman-crypto/intel-candidate/policies/scoring-policy.v1.json";

const RESEARCH_COMPATIBLE_HORIZONS: &[&str] = &["15m", "1h", "4h", "24h", "72h"];

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

fn default_true() -> bool {
    true
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

pub fn load_policy(path: &Path) -> AppResult<ScoringPolicy> {
    let bytes = std::fs::read(path)?;
    let policy = serde_json::from_slice(&bytes)?;
    validate_policy(&policy)?;
    Ok(policy)
}

fn validate_policy(policy: &ScoringPolicy) -> AppResult<()> {
    for (event_type, horizons) in &policy.event_type_to_allowed_horizons {
        if horizons.is_empty() {
            return Err(crate::error::AppError::config(format!(
                "event_type_to_allowed_horizons.{event_type} must not be empty"
            )));
        }
        for horizon in horizons {
            if !RESEARCH_COMPATIBLE_HORIZONS.contains(&horizon.as_str()) {
                return Err(crate::error::AppError::config(format!(
                    "event_type_to_allowed_horizons.{event_type} contains unsupported downstream research horizon {horizon}; allowed horizons are {}",
                    RESEARCH_COMPATIBLE_HORIZONS.join(",")
                )));
            }
        }
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_policy() -> ScoringPolicy {
        load_policy(&Path::new(env!("CARGO_MANIFEST_DIR")).join("policies/scoring-policy.v1.json"))
            .expect("repo policy must load")
    }

    #[test]
    fn packaged_policy_only_uses_research_compatible_horizons() {
        let policy = repo_policy();

        for horizons in policy.event_type_to_allowed_horizons.values() {
            assert!(!horizons.is_empty());
            for horizon in horizons {
                assert!(
                    RESEARCH_COMPATIBLE_HORIZONS.contains(&horizon.as_str()),
                    "{horizon} must be accepted by downstream research"
                );
            }
        }
    }

    #[test]
    fn general_intel_tracks_short_mid_and_daily_horizons() {
        let policy = repo_policy();

        assert_eq!(policy.policy_version, "intel_candidate_scoring_v2");
        assert_eq!(
            policy.event_type_to_allowed_horizons.get("other"),
            Some(&vec!["1h".to_owned(), "4h".to_owned(), "24h".to_owned()])
        );
    }

    #[test]
    fn policy_validation_rejects_horizon_beyond_research_contract() {
        let mut policy = repo_policy();
        policy
            .event_type_to_allowed_horizons
            .insert("project_notice".to_owned(), vec!["7d".to_owned()]);

        let error = validate_policy(&policy).expect_err("7d should be rejected");

        assert!(
            error
                .to_string()
                .contains("unsupported downstream research horizon 7d")
        );
    }
}
