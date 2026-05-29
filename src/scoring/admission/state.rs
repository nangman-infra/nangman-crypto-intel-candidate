use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::scoring) struct AdmissionState {
    pub(in crate::scoring) has_valid_schema: bool,
    pub(in crate::scoring) has_required_times: bool,
    pub(in crate::scoring) has_valid_time_order: bool,
    pub(in crate::scoring) has_evidence: bool,
    pub(in crate::scoring) has_lineage: bool,
    pub(in crate::scoring) has_symbols: bool,
    pub(in crate::scoring) has_source_independence: bool,
    pub(in crate::scoring) source_independence_ok_for_research: bool,
    pub(in crate::scoring) source_independence_ok_for_strong: bool,
    pub(in crate::scoring) has_symbol_resolution_trace: bool,
    pub(in crate::scoring) symbol_resolution_ok_for_research: bool,
    pub(in crate::scoring) symbol_resolution_ok_for_strong: bool,
    pub(in crate::scoring) has_data_quality_summary: bool,
    pub(in crate::scoring) has_market_feature_delta: bool,
    pub(in crate::scoring) has_market_regime_context: bool,
    pub(in crate::scoring) selected_market_feature_delta: Option<SelectedMarketArtifactTrace>,
    pub(in crate::scoring) selected_market_regime_context: Option<SelectedMarketArtifactTrace>,
    pub(in crate::scoring) has_point_in_time_universe: bool,
    pub(in crate::scoring) approved_universe_symbol: bool,
    pub(in crate::scoring) has_derivatives_metric_delta: bool,
    pub(in crate::scoring) market_context_allows_research: bool,
    pub(in crate::scoring) market_context_allows_strong: bool,
    pub(in crate::scoring) stale_market_context: bool,
    pub(in crate::scoring) social_only: bool,
    pub(in crate::scoring) has_medium_or_high_contradiction: bool,
    pub(in crate::scoring) forbidden_output_terms: Vec<String>,
    pub(in crate::scoring) quarantine_reasons: Vec<String>,
    pub(in crate::scoring) reject_reasons: Vec<String>,
    pub(in crate::scoring) observe_reasons: Vec<String>,
}

impl AdmissionState {
    pub(in crate::scoring) fn selected_market_artifacts(&self) -> Vec<SelectedMarketArtifactTrace> {
        let mut artifacts = Vec::new();
        if let Some(trace) = &self.selected_market_feature_delta {
            artifacts.push(trace.clone());
        }
        if let Some(trace) = &self.selected_market_regime_context {
            artifacts.push(trace.clone());
        }
        artifacts
    }

    pub(in crate::scoring) fn research_block_reasons(
        &self,
        packet: &StructuredIntelPacket,
    ) -> Vec<String> {
        let mut reasons = Vec::new();
        if !self.has_required_times {
            reasons.push("missing_replay_time_contract".to_owned());
        }
        if !self.has_valid_time_order {
            reasons.push("invalid_replay_time_order".to_owned());
        }
        if !self.has_point_in_time_universe {
            reasons.push("missing_point_in_time_universe".to_owned());
        }
        if !self.approved_universe_symbol {
            reasons.push("not_admitted_universe".to_owned());
        }
        if !self.has_data_quality_summary {
            reasons.push("missing_data_quality_summary".to_owned());
        }
        if !self.has_market_feature_delta {
            reasons.push("missing_market_feature_delta".to_owned());
        }
        if !self.has_market_regime_context {
            reasons.push("missing_market_regime_context".to_owned());
        }
        if !self.has_source_independence {
            reasons.push("missing_source_independence".to_owned());
        } else if !self.source_independence_ok_for_research {
            reasons.push("insufficient_source_independence".to_owned());
        }
        if !self.has_symbol_resolution_trace {
            reasons.push("missing_symbol_resolution_trace".to_owned());
        } else if !self.symbol_resolution_ok_for_research {
            reasons.push("weak_symbol_resolution".to_owned());
        }
        if packet.event_type.is_derivatives_like() && !self.has_derivatives_metric_delta {
            reasons.push("derivatives_metric_delta_missing".to_owned());
        }
        if !self.market_context_allows_research {
            reasons.push("market_context_not_research_admissible".to_owned());
        }
        reasons
    }

    pub(in crate::scoring) fn strong_block_reasons(&self) -> Vec<String> {
        let mut reasons = self.reject_reasons.clone();
        if !self.source_independence_ok_for_strong {
            reasons.push("strong_requires_source_independence_or_official_source".to_owned());
        }
        if !self.symbol_resolution_ok_for_strong {
            reasons.push("strong_requires_moderate_or_strong_symbol_resolution".to_owned());
        }
        if !self.market_context_allows_strong {
            reasons.push("strong_requires_available_symbol_context".to_owned());
        }
        if self.has_medium_or_high_contradiction {
            reasons.push("strong_blocked_by_contradiction".to_owned());
        }
        if self.social_only {
            reasons.push("strong_blocked_by_social_only".to_owned());
        }
        reasons
    }
}
