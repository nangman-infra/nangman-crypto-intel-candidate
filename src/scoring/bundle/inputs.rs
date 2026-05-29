use super::super::*;
use super::context::ResearchBundleInputs;

pub(in crate::scoring) fn research_bundle_inputs<'a>(
    packet: &StructuredIntelPacket,
    universe: Option<&'a SymbolUniverseSnapshot>,
) -> Option<ResearchBundleInputs<'a>> {
    Some(ResearchBundleInputs {
        universe: universe?,
        decision_available_at_ms: packet.decision_available_at_ms?,
        fetched_at_ms: packet.fetched_at_ms?,
        structured_at_ms: packet.structured_at_ms?,
        source_independence: packet.source_independence_summary.clone()?,
    })
}

pub(in crate::scoring) fn research_bundle_block_reasons(
    packet: &StructuredIntelPacket,
    universe: Option<&SymbolUniverseSnapshot>,
    candidate_id: Option<&str>,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if candidate_id.is_none() {
        reasons.push("missing_candidate_id_for_research_bundle".to_owned());
    }
    if universe.is_none() {
        reasons.push("missing_universe_for_research_bundle".to_owned());
    }
    if packet.decision_available_at_ms.is_none() {
        reasons.push("missing_decision_available_at_for_research_bundle".to_owned());
    }
    if packet.fetched_at_ms.is_none() {
        reasons.push("missing_fetched_at_for_research_bundle".to_owned());
    }
    if packet.structured_at_ms.is_none() {
        reasons.push("missing_structured_at_for_research_bundle".to_owned());
    }
    if packet.source_independence_summary.is_none() {
        reasons.push("missing_source_independence_for_research_bundle".to_owned());
    }
    reasons
}
