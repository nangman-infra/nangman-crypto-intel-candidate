use super::super::*;
use std::collections::BTreeSet;

pub(super) fn forbidden_generated_terms(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
) -> Vec<String> {
    let terms: BTreeSet<String> = policy
        .forbidden_output_terms
        .iter()
        .map(|term| term.to_ascii_lowercase())
        .collect();
    let generated_text = [
        packet.topic_summary.as_str(),
        packet.stance_summary.as_str(),
        packet.risk_summary.as_str(),
        packet.regime_hint.as_str(),
        packet.scenario_hint.as_str(),
        packet.terminal_decision.as_str(),
    ]
    .join(" ");
    let tokens: BTreeSet<String> = generated_text
        .to_ascii_lowercase()
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    terms
        .into_iter()
        .filter(|term| tokens.contains(term))
        .collect()
}

pub(super) fn has_social_only_source(packet: &StructuredIntelPacket) -> bool {
    packet
        .source_quality_summary
        .to_ascii_lowercase()
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .any(|token| token == "social_only")
}

pub(in crate::scoring) fn is_medium_contradiction(flag: &ContradictionFlag) -> bool {
    matches!(
        flag,
        ContradictionFlag::SourceClaimConflict | ContradictionFlag::RumorVsOfficial
    )
}

pub(in crate::scoring) fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for value in values {
        if seen.insert(value.clone()) {
            deduped.push(value);
        }
    }
    deduped
}
