use crate::model::{MarketContextRef, StructuredIntelPacket};

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

pub(super) fn feature_delta_artifact_type(packet: &StructuredIntelPacket) -> &'static str {
    packet
        .market_context_ref
        .as_ref()
        .and_then(|reference| {
            reference
                .market_feature_delta_summary_key
                .as_ref()
                .filter(|key| !key.trim().is_empty())
        })
        .map(|_| "market_feature_delta_summary")
        .unwrap_or("market_feature_delta")
}
