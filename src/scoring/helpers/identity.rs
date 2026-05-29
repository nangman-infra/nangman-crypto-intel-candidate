use super::super::*;

pub fn effective_packet_family_id(packet: &StructuredIntelPacket) -> &str {
    if !packet.packet_family_id.trim().is_empty() {
        packet.packet_family_id.as_str()
    } else if !packet.raw_event_id.trim().is_empty() {
        packet.raw_event_id.as_str()
    } else {
        packet.packet_id.as_str()
    }
}

pub(in crate::scoring) fn candidate_id(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    universe: Option<&SymbolUniverseSnapshot>,
) -> String {
    let decision_available_at_ms = packet.decision_available_at_ms.unwrap_or_default();
    let event_bucket = hour_bucket_ms(decision_available_at_ms).to_string();
    let decision_key = decision_available_at_ms.to_string();
    let symbols = packet.normalized_symbols.join(",");
    let universe_id = universe
        .map(|snapshot| snapshot.symbol_universe_snapshot_id.as_str())
        .unwrap_or("missing_universe");
    let hypothesis_type = policy
        .event_type_to_hypothesis_type
        .get(packet.event_type.as_policy_key())
        .map(String::as_str)
        .unwrap_or("general_intel_observation");
    stable_id(
        "cand",
        &[
            policy.policy_version.as_str(),
            symbols.as_str(),
            packet.event_type.as_policy_key(),
            packet.cluster_id.as_str(),
            event_bucket.as_str(),
            decision_key.as_str(),
            universe_id,
            hypothesis_type,
        ],
    )
}

pub(in crate::scoring) fn hypothesis_type(
    policy: &ScoringPolicy,
    event_type: &EventType,
) -> String {
    policy
        .event_type_to_hypothesis_type
        .get(event_type.as_policy_key())
        .cloned()
        .unwrap_or_else(|| "general_intel_observation".to_owned())
}

pub(in crate::scoring) fn dirty_triggers(packet: &StructuredIntelPacket) -> Vec<String> {
    let mut triggers = vec![
        "scoring_policy_version_changed".to_owned(),
        "candidate_app_version_changed".to_owned(),
    ];
    if packet.market_context_ref.is_some()
        || !matches!(packet.market_context_status, MarketContextStatus::Unknown)
    {
        triggers.push("market_context_updated".to_owned());
        triggers.push("market_feature_delta_updated".to_owned());
    }
    if matches!(packet.event_type, EventType::FundingShift) {
        triggers.push("derivatives_rollup_updated".to_owned());
    }
    triggers
}

pub(in crate::scoring) fn harness_queue_hint(packet: &StructuredIntelPacket) -> String {
    match packet.event_type {
        EventType::FundingShift => "derivatives_delta_persistence".to_owned(),
        EventType::SocialHype | EventType::SocialBacklash => "attention_reaction_smoke".to_owned(),
        EventType::Listing
        | EventType::ExchangeListing
        | EventType::Delisting
        | EventType::ExchangeDelisting => "venue_event_reaction".to_owned(),
        _ => "event_reaction_smoke".to_owned(),
    }
}

pub(in crate::scoring) fn hypothesis_lineage_refs(packet: &StructuredIntelPacket) -> Vec<String> {
    let mut refs = parent_artifact_ids(packet);
    if let Some(reference) = &packet.market_context_ref {
        refs.extend(reference.output_object_keys.clone());
        refs.extend(reference.market_data_quality_summary_key.clone());
        refs.extend(reference.market_feature_delta_key.clone());
        refs.extend(reference.market_feature_delta_summary_key.clone());
        refs.extend(reference.market_regime_context_key.clone());
        refs.extend(reference.symbol_universe_snapshot_key.clone());
    }
    dedupe_strings(refs)
}

pub(in crate::scoring) fn parent_artifact_ids(packet: &StructuredIntelPacket) -> Vec<String> {
    let mut ids = vec![packet.packet_id.clone(), packet.cluster_id.clone()];
    ids.extend(packet.source_event_ids.clone());
    dedupe_strings(ids)
}
