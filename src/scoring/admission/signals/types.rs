use super::super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::scoring::admission) struct AdmissionSignals {
    pub(in crate::scoring::admission) has_valid_schema: bool,
    pub(in crate::scoring::admission) has_required_times: bool,
    pub(in crate::scoring::admission) has_valid_time_order: bool,
    pub(in crate::scoring::admission) has_evidence: bool,
    pub(in crate::scoring::admission) has_lineage: bool,
    pub(in crate::scoring::admission) has_symbols: bool,
    pub(in crate::scoring::admission) has_source_independence: bool,
    pub(in crate::scoring::admission) source_independence_ok_for_research: bool,
    pub(in crate::scoring::admission) source_independence_ok_for_strong: bool,
    pub(in crate::scoring::admission) has_symbol_resolution_trace: bool,
    pub(in crate::scoring::admission) symbol_resolution_ok_for_research: bool,
    pub(in crate::scoring::admission) symbol_resolution_ok_for_strong: bool,
    pub(in crate::scoring::admission) has_data_quality_summary: bool,
    pub(in crate::scoring::admission) has_market_feature_delta: bool,
    pub(in crate::scoring::admission) has_market_regime_context: bool,
    pub(in crate::scoring::admission) selected_market_feature_delta:
        Option<SelectedMarketArtifactTrace>,
    pub(in crate::scoring::admission) selected_market_regime_context:
        Option<SelectedMarketArtifactTrace>,
    pub(in crate::scoring::admission) has_point_in_time_universe: bool,
    pub(in crate::scoring::admission) approved_universe_symbol: bool,
    pub(in crate::scoring::admission) has_derivatives_metric_delta: bool,
    pub(in crate::scoring::admission) market_context_allows_research: bool,
    pub(in crate::scoring::admission) market_context_allows_strong: bool,
    pub(in crate::scoring::admission) stale_market_context: bool,
    pub(in crate::scoring::admission) social_only: bool,
    pub(in crate::scoring::admission) has_medium_or_high_contradiction: bool,
    pub(in crate::scoring::admission) forbidden_output_terms: Vec<String>,
}
