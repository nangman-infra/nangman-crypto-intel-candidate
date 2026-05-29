mod artifacts;
mod basics;
mod market;
mod text;
mod time;
mod types;

use crate::model::{MarketContextStatus, StructuredIntelPacket};
use crate::policy::ScoringPolicy;
use crate::scoring::MarketArtifactInputs;
use crate::scoring::helpers::approved_universe_symbols;
use artifacts::{
    has_derivatives_metric_delta, selected_market_feature_delta_when_referenced,
    selected_market_regime_context_when_referenced,
};
pub(in crate::scoring) use artifacts::{
    is_usable_market_artifact_quality, normalize_market_artifact_quality,
};
use basics::{
    has_data_quality_summary, has_evidence, has_lineage, has_source_independence,
    has_symbol_resolution_trace, has_symbols, has_valid_schema,
    source_independence_ok_for_research, source_independence_ok_for_strong,
    symbol_resolution_ok_for_research, symbol_resolution_ok_for_strong,
};
pub(in crate::scoring) use market::effective_market_context_status;
use market::market_context_allows_research;
pub(in crate::scoring) use text::{dedupe_strings, is_medium_contradiction};
use text::{forbidden_generated_terms, has_social_only_source};
use time::{has_required_replay_times, has_valid_replay_time_order, market_artifact_cutoff_ms};
pub(super) use types::AdmissionSignals;

pub(super) fn build_admission_signals(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    market_artifacts: MarketArtifactInputs<'_>,
    candidate_created_at_ms: i64,
) -> AdmissionSignals {
    let universe = market_artifacts.universe;
    let market_artifact_cutoff_ms = market_artifact_cutoff_ms(packet, candidate_created_at_ms);
    let has_valid_schema = has_valid_schema(packet);
    let forbidden_output_terms = forbidden_generated_terms(packet, policy);
    let has_required_times = has_required_replay_times(packet);
    let has_valid_time_order = has_valid_replay_time_order(packet);
    let has_evidence = has_evidence(packet);
    let has_lineage = has_lineage(packet);
    let has_symbols = has_symbols(packet);
    let has_source_independence = has_source_independence(packet);
    let source_independence_ok_for_research = source_independence_ok_for_research(packet, policy);
    let source_independence_ok_for_strong = source_independence_ok_for_strong(packet, policy);
    let has_symbol_resolution_trace = has_symbol_resolution_trace(packet);
    let symbol_resolution_ok_for_research = symbol_resolution_ok_for_research(packet, policy);
    let symbol_resolution_ok_for_strong = symbol_resolution_ok_for_strong(packet, policy);
    let has_data_quality_summary = has_data_quality_summary(packet);
    let selected_market_feature_delta = selected_market_feature_delta_when_referenced(
        packet,
        market_artifacts,
        market_artifact_cutoff_ms,
    );
    let selected_market_regime_context = selected_market_regime_context_when_referenced(
        packet,
        market_artifacts,
        market_artifact_cutoff_ms,
    );
    let has_market_feature_delta = selected_market_feature_delta.is_some();
    let has_market_regime_context = selected_market_regime_context.is_some();
    let has_point_in_time_universe = universe.is_some();
    let approved_universe_symbol =
        universe.is_some_and(|snapshot| approved_universe_symbols(packet, snapshot));
    let has_derivatives_metric_delta =
        has_derivatives_metric_delta(packet, market_artifacts, market_artifact_cutoff_ms);
    let market_status = effective_market_context_status(packet);
    let market_context_allows_strong =
        market_status.as_policy_key() == policy.market_context_status_policy.strong_requires;
    let market_context_allows_research =
        market_context_allows_research(packet, policy, &market_status);
    let stale_market_context = matches!(market_status, MarketContextStatus::StaleButUsable);
    let social_only = has_social_only_source(packet);
    let has_medium_or_high_contradiction = packet
        .contradiction_flags
        .iter()
        .any(is_medium_contradiction);

    AdmissionSignals {
        has_valid_schema,
        has_required_times,
        has_valid_time_order,
        has_evidence,
        has_lineage,
        has_symbols,
        has_source_independence,
        source_independence_ok_for_research,
        source_independence_ok_for_strong,
        has_symbol_resolution_trace,
        symbol_resolution_ok_for_research,
        symbol_resolution_ok_for_strong,
        has_data_quality_summary,
        has_market_feature_delta,
        has_market_regime_context,
        selected_market_feature_delta,
        selected_market_regime_context,
        has_point_in_time_universe,
        approved_universe_symbol,
        has_derivatives_metric_delta,
        market_context_allows_research,
        market_context_allows_strong,
        stale_market_context,
        social_only,
        has_medium_or_high_contradiction,
        forbidden_output_terms,
    }
}
