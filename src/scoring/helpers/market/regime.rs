use crate::model::{MarketRegimeContext, SelectedMarketArtifactTrace, StructuredIntelPacket};
use crate::scoring::MarketArtifactInputs;
use crate::scoring::admission::{
    is_usable_market_artifact_quality, normalize_market_artifact_quality,
};

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
        .filter(|context| regime_context_matches(context, market_artifact_cutoff_ms))
        .max_by_key(|context| (context.window_end_ms, context.known_as_of_ms))
        .map(|context| selected_regime_context_trace(context, artifact_key))
}

fn regime_context_matches(context: &MarketRegimeContext, market_artifact_cutoff_ms: i64) -> bool {
    context.window_end_ms <= market_artifact_cutoff_ms
        && context.known_as_of_ms <= market_artifact_cutoff_ms
        && context.sector_return_same_window.is_some()
        && is_usable_market_artifact_quality(&context.quality_status)
}

fn selected_regime_context_trace(
    context: &MarketRegimeContext,
    artifact_key: Option<String>,
) -> SelectedMarketArtifactTrace {
    SelectedMarketArtifactTrace {
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
    }
}
