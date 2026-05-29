use super::symbols::canonical_symbol_candidates;
use crate::model::{MarketContextRef, SelectedMarketArtifactTrace, StructuredIntelPacket};
use crate::scoring::MarketArtifactInputs;
use crate::scoring::admission::{
    is_usable_market_artifact_quality, normalize_market_artifact_quality,
};

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

pub(in crate::scoring) fn selected_market_feature_delta_metric_filter(
    packet: &StructuredIntelPacket,
) -> impl Fn(&str) -> bool + '_ {
    move |metric_name| {
        if packet.event_type.is_derivatives_like() {
            is_derivatives_market_metric(metric_name)
        } else {
            true
        }
    }
}

fn is_derivatives_market_metric(metric_name: &str) -> bool {
    matches!(
        metric_name,
        "open_interest" | "funding_rate" | "liquidation" | "long_short_ratio"
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
    let artifact_type = packet
        .market_context_ref
        .as_ref()
        .and_then(|reference| {
            reference
                .market_feature_delta_summary_key
                .as_ref()
                .filter(|key| !key.trim().is_empty())
        })
        .map(|_| "market_feature_delta_summary")
        .unwrap_or("market_feature_delta");
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
                    candidates.contains(&delta.symbol_canonical.to_ascii_uppercase())
                        && metric_allowed_ref(delta.metric_name.as_str())
                        && delta.window_end_ms <= market_artifact_cutoff_ms
                        && delta.known_as_of_ms <= market_artifact_cutoff_ms
                        && is_usable_market_artifact_quality(&delta.quality_status)
                        && (delta.change_pct_1h.is_some() || delta.change_pct_15m.is_some())
                })
        })
        .max_by_key(|delta| (delta.window_end_ms, delta.known_as_of_ms))
        .map(|delta| SelectedMarketArtifactTrace {
            artifact_type: artifact_type.to_owned(),
            artifact_id: delta.feature_delta_id.clone(),
            artifact_key,
            l1_run_id: Some(delta.l1_run_id.clone()),
            symbol_canonical: Some(delta.symbol_canonical.clone()),
            metric_name: Some(delta.metric_name.clone()),
            scope: None,
            window_start_ms: delta.window_start_ms,
            window_end_ms: delta.window_end_ms,
            known_as_of_ms: delta.known_as_of_ms,
            quality_status: normalize_market_artifact_quality(&delta.quality_status),
        })
}

pub(in crate::scoring) fn market_feature_delta_artifact_key(
    reference: &MarketContextRef,
) -> Option<&String> {
    reference
        .market_feature_delta_summary_key
        .as_ref()
        .filter(|key| !key.trim().is_empty())
        .or_else(|| {
            reference
                .market_feature_delta_key
                .as_ref()
                .filter(|key| !key.trim().is_empty())
        })
}

pub(in crate::scoring) fn selected_market_regime_context(
    packet: &StructuredIntelPacket,
    market_artifacts: MarketArtifactInputs<'_>,
    market_artifact_cutoff_ms: Option<i64>,
) -> Option<SelectedMarketArtifactTrace> {
    let market_artifact_cutoff_ms = market_artifact_cutoff_ms?;
    let artifact_key = packet
        .market_context_ref
        .as_ref()
        .and_then(|reference| reference.market_regime_context_key.clone());
    market_artifacts
        .market_regime_contexts
        .iter()
        .filter(|context| {
            context.window_end_ms <= market_artifact_cutoff_ms
                && context.known_as_of_ms <= market_artifact_cutoff_ms
                && context.sector_return_same_window.is_some()
                && is_usable_market_artifact_quality(&context.quality_status)
        })
        .max_by_key(|context| (context.window_end_ms, context.known_as_of_ms))
        .map(|context| SelectedMarketArtifactTrace {
            artifact_type: "market_regime_context".to_owned(),
            artifact_id: context.regime_context_id.clone(),
            artifact_key,
            l1_run_id: Some(context.l1_run_id.clone()),
            symbol_canonical: None,
            metric_name: None,
            scope: Some(context.scope.clone()),
            window_start_ms: context.window_start_ms,
            window_end_ms: context.window_end_ms,
            known_as_of_ms: context.known_as_of_ms,
            quality_status: normalize_market_artifact_quality(&context.quality_status),
        })
}
