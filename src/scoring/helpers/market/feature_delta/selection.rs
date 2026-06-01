use std::collections::BTreeSet;

use crate::model::{MarketFeatureDelta, SelectedMarketArtifactTrace, StructuredIntelPacket};
use crate::scoring::MarketArtifactInputs;
use crate::scoring::admission::is_usable_market_artifact_quality;
use crate::scoring::helpers::symbols::canonical_symbol_candidates;

use super::metrics::is_derivatives_market_metric;
use super::reference::{feature_delta_artifact_type, market_feature_delta_artifact_key};
use super::trace::selected_feature_delta_trace;

pub(in crate::scoring) fn selected_derivatives_market_feature_delta(
    packet: &StructuredIntelPacket,
    market_artifacts: MarketArtifactInputs<'_>,
    market_artifact_cutoff_ms: Option<i64>,
) -> Option<SelectedMarketArtifactTrace> {
    selected_market_feature_delta(
        packet,
        market_artifacts,
        market_artifact_cutoff_ms,
        is_derivatives_market_metric,
    )
}

pub(in crate::scoring) fn selected_market_feature_delta(
    packet: &StructuredIntelPacket,
    market_artifacts: MarketArtifactInputs<'_>,
    market_artifact_cutoff_ms: Option<i64>,
    metric_allowed: impl Fn(&str) -> bool,
) -> Option<SelectedMarketArtifactTrace> {
    let market_artifact_cutoff_ms = market_artifact_cutoff_ms?;
    let artifact_key = packet
        .market_context_ref
        .as_ref()
        .and_then(market_feature_delta_artifact_key)
        .cloned();
    let artifact_type = feature_delta_artifact_type(packet);
    let metric_allowed_ref = &metric_allowed;
    packet
        .normalized_symbols
        .iter()
        .flat_map(|symbol| {
            let candidates = canonical_symbol_candidates(symbol);
            market_artifacts
                .market_feature_deltas
                .iter()
                .filter(move |delta| {
                    feature_delta_matches_packet(
                        delta,
                        &candidates,
                        market_artifact_cutoff_ms,
                        metric_allowed_ref,
                    )
                })
        })
        .max_by_key(|delta| (delta.window_end_ms, delta.known_as_of_ms))
        .map(|delta| selected_feature_delta_trace(delta, artifact_type, artifact_key))
}

fn feature_delta_matches_packet(
    delta: &MarketFeatureDelta,
    candidates: &BTreeSet<String>,
    market_artifact_cutoff_ms: i64,
    metric_allowed: &impl Fn(&str) -> bool,
) -> bool {
    candidates.contains(&delta.symbol_canonical.to_ascii_uppercase())
        && metric_allowed(delta.metric_name.as_str())
        && delta.window_end_ms <= market_artifact_cutoff_ms
        && delta.known_as_of_ms <= market_artifact_cutoff_ms
        && is_usable_market_artifact_quality(&delta.quality_status)
        && (delta.change_pct_1h.is_some() || delta.change_pct_15m.is_some())
}
