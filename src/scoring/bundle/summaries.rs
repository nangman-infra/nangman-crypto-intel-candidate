use std::collections::BTreeMap;

use crate::model::{DataQualitySummaryRef, StructuredIntelPacket};
use crate::scoring::admission::effective_market_context_status;

pub(super) fn data_quality_summary(packet: &StructuredIntelPacket) -> DataQualitySummaryRef {
    let market_data_quality_summary_key = packet
        .market_context_ref
        .as_ref()
        .and_then(|reference| reference.market_data_quality_summary_key.clone());
    DataQualitySummaryRef {
        status: if market_data_quality_summary_key.is_some() {
            "present".to_owned()
        } else {
            "missing".to_owned()
        },
        market_data_quality_summary_key,
    }
}

pub(super) fn confidence_summary(packet: &StructuredIntelPacket) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "symbol_confidence_band".to_owned(),
            packet.symbol_confidence_band.as_policy_key().to_owned(),
        ),
        (
            "packet_confidence_band".to_owned(),
            packet.confidence_band.as_policy_key().to_owned(),
        ),
        (
            "market_context_status".to_owned(),
            effective_market_context_status(packet)
                .as_policy_key()
                .to_owned(),
        ),
    ])
}
