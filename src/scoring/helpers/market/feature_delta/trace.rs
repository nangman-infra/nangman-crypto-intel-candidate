use crate::model::{MarketFeatureDelta, SelectedMarketArtifactTrace};
use crate::scoring::admission::normalize_market_artifact_quality;

pub(super) fn selected_feature_delta_trace(
    delta: &MarketFeatureDelta,
    artifact_type: &str,
    artifact_key: Option<String>,
) -> SelectedMarketArtifactTrace {
    SelectedMarketArtifactTrace {
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
    }
}
