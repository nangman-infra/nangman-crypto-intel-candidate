use super::*;

mod reasons;
mod signals;
mod state;

use reasons::{collect_observe_reasons, collect_quarantine_reasons, collect_reject_reasons};
use signals::build_admission_signals;
pub(super) use state::AdmissionState;

pub(super) use signals::{
    dedupe_strings, effective_market_context_status, is_medium_contradiction,
    is_usable_market_artifact_quality, normalize_market_artifact_quality,
};

pub(super) fn evaluate_admission(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    market_artifacts: MarketArtifactInputs<'_>,
    candidate_created_at_ms: i64,
) -> AdmissionState {
    let signals =
        build_admission_signals(packet, policy, market_artifacts, candidate_created_at_ms);
    let quarantine_reasons = collect_quarantine_reasons(policy, &signals);
    let reject_reasons = collect_reject_reasons(policy, &signals);
    let observe_reasons = collect_observe_reasons(packet, policy, &signals);

    AdmissionState {
        has_valid_schema: signals.has_valid_schema,
        has_required_times: signals.has_required_times,
        has_valid_time_order: signals.has_valid_time_order,
        has_evidence: signals.has_evidence,
        has_lineage: signals.has_lineage,
        has_symbols: signals.has_symbols,
        has_source_independence: signals.has_source_independence,
        source_independence_ok_for_research: signals.source_independence_ok_for_research,
        source_independence_ok_for_strong: signals.source_independence_ok_for_strong,
        has_symbol_resolution_trace: signals.has_symbol_resolution_trace,
        symbol_resolution_ok_for_research: signals.symbol_resolution_ok_for_research,
        symbol_resolution_ok_for_strong: signals.symbol_resolution_ok_for_strong,
        has_data_quality_summary: signals.has_data_quality_summary,
        has_market_feature_delta: signals.has_market_feature_delta,
        has_market_regime_context: signals.has_market_regime_context,
        selected_market_feature_delta: signals.selected_market_feature_delta,
        selected_market_regime_context: signals.selected_market_regime_context,
        has_point_in_time_universe: signals.has_point_in_time_universe,
        approved_universe_symbol: signals.approved_universe_symbol,
        has_derivatives_metric_delta: signals.has_derivatives_metric_delta,
        market_context_allows_research: signals.market_context_allows_research,
        market_context_allows_strong: signals.market_context_allows_strong,
        stale_market_context: signals.stale_market_context,
        social_only: signals.social_only,
        has_medium_or_high_contradiction: signals.has_medium_or_high_contradiction,
        forbidden_output_terms: signals.forbidden_output_terms,
        quarantine_reasons,
        reject_reasons,
        observe_reasons,
    }
}
