use crate::model::{STRUCTURED_PACKET_SCHEMA_VERSION, StructuredIntelPacket};
use crate::policy::ScoringPolicy;
use crate::scoring::helpers::symbol_resolution_ok;

pub(super) fn has_valid_schema(packet: &StructuredIntelPacket) -> bool {
    packet.schema_version.as_deref() == Some(STRUCTURED_PACKET_SCHEMA_VERSION)
}

pub(super) fn has_evidence(packet: &StructuredIntelPacket) -> bool {
    !packet.text_evidence.is_empty()
        || !packet.metric_evidence.is_empty()
        || !packet.evidence_sentences.is_empty()
}

pub(super) fn has_lineage(packet: &StructuredIntelPacket) -> bool {
    !packet.packet_id.trim().is_empty()
        && !packet.cluster_id.trim().is_empty()
        && !packet.source_event_ids.is_empty()
}

pub(super) fn has_symbols(packet: &StructuredIntelPacket) -> bool {
    !packet.normalized_symbols.is_empty()
}

pub(super) fn has_source_independence(packet: &StructuredIntelPacket) -> bool {
    packet.source_independence_summary.is_some()
}

pub(super) fn source_independence_ok_for_research(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
) -> bool {
    packet
        .source_independence_summary
        .as_ref()
        .is_some_and(|summary| super::super::source_independence_ok_for_research(summary, policy))
}

pub(super) fn source_independence_ok_for_strong(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
) -> bool {
    packet
        .source_independence_summary
        .as_ref()
        .is_some_and(|summary| super::super::source_independence_ok_for_strong(summary, policy))
}

pub(super) fn has_symbol_resolution_trace(packet: &StructuredIntelPacket) -> bool {
    !packet.symbol_resolution_trace.is_empty()
}

pub(super) fn symbol_resolution_ok_for_research(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
) -> bool {
    symbol_resolution_ok(
        packet,
        &policy
            .admission_requirements
            .research_allowed_symbol_mapping_confidence,
    )
}

pub(super) fn symbol_resolution_ok_for_strong(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
) -> bool {
    symbol_resolution_ok(
        packet,
        &policy
            .admission_requirements
            .strong_allowed_symbol_mapping_confidence,
    )
}

pub(super) fn has_data_quality_summary(packet: &StructuredIntelPacket) -> bool {
    packet
        .market_context_ref
        .as_ref()
        .and_then(|reference| reference.market_data_quality_summary_key.as_ref())
        .is_some_and(|key| !key.trim().is_empty())
}
