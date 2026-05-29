use super::super::*;

pub(super) fn selected_market_feature_delta_when_referenced(
    packet: &StructuredIntelPacket,
    market_artifacts: MarketArtifactInputs<'_>,
    market_artifact_cutoff_ms: Option<i64>,
) -> Option<SelectedMarketArtifactTrace> {
    let has_reference = packet
        .market_context_ref
        .as_ref()
        .and_then(market_feature_delta_artifact_key)
        .is_some_and(|key| !key.trim().is_empty());
    has_reference
        .then(|| {
            let metric_allowed = selected_market_feature_delta_metric_filter(packet);
            selected_market_feature_delta(
                packet,
                market_artifacts,
                market_artifact_cutoff_ms,
                |metric_name| metric_allowed(metric_name),
            )
        })
        .flatten()
}

pub(super) fn selected_market_regime_context_when_referenced(
    packet: &StructuredIntelPacket,
    market_artifacts: MarketArtifactInputs<'_>,
    market_artifact_cutoff_ms: Option<i64>,
) -> Option<SelectedMarketArtifactTrace> {
    let has_reference = packet
        .market_context_ref
        .as_ref()
        .and_then(|reference| reference.market_regime_context_key.as_ref())
        .is_some_and(|key| !key.trim().is_empty());
    has_reference
        .then(|| {
            selected_market_regime_context(packet, market_artifacts, market_artifact_cutoff_ms)
        })
        .flatten()
}

pub(super) fn has_derivatives_metric_delta(
    packet: &StructuredIntelPacket,
    market_artifacts: MarketArtifactInputs<'_>,
    market_artifact_cutoff_ms: Option<i64>,
) -> bool {
    packet.metric_evidence.iter().any(|metric| {
        metric.delta_pct.is_some()
            && matches!(
                metric.metric_name.as_str(),
                "open_interest" | "funding_rate" | "liquidation" | "long_short_ratio"
            )
    }) || selected_derivatives_market_feature_delta(
        packet,
        market_artifacts,
        market_artifact_cutoff_ms,
    )
    .is_some()
}

pub(in crate::scoring) fn is_usable_market_artifact_quality(status: &str) -> bool {
    matches!(status.trim(), "" | "complete" | "partial")
}

pub(in crate::scoring) fn normalize_market_artifact_quality(status: &str) -> String {
    let trimmed = status.trim();
    if trimmed.is_empty() {
        "unspecified".to_owned()
    } else {
        trimmed.to_owned()
    }
}
