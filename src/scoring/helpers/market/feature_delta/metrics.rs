use crate::model::StructuredIntelPacket;

pub(in super::super::super::super) fn selected_market_feature_delta_metric_filter(
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

pub(super) fn is_derivatives_market_metric(metric_name: &str) -> bool {
    matches!(
        metric_name,
        "open_interest" | "funding_rate" | "liquidation" | "long_short_ratio"
    )
}
