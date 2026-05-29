use super::super::*;

pub(in crate::scoring) fn source_independence_ok_for_research(
    summary: &SourceIndependenceSummary,
    policy: &ScoringPolicy,
) -> bool {
    summary.independent_source_count
        >= policy
            .admission_requirements
            .research_min_independent_source_count
        || (policy
            .admission_requirements
            .official_source_can_replace_min_independent_source_count
            && summary.official_source_present)
}

pub(in crate::scoring) fn source_independence_ok_for_strong(
    summary: &SourceIndependenceSummary,
    policy: &ScoringPolicy,
) -> bool {
    summary.independent_source_count
        >= policy
            .admission_requirements
            .strong_min_independent_source_count
        || (policy
            .admission_requirements
            .official_source_can_replace_min_independent_source_count
            && summary.official_source_present)
}

pub(in crate::scoring) fn symbol_resolution_ok(
    packet: &StructuredIntelPacket,
    allowed: &[String],
) -> bool {
    if packet.normalized_symbols.is_empty() || packet.symbol_resolution_trace.is_empty() {
        return false;
    }
    let allowed: BTreeSet<String> = allowed
        .iter()
        .map(|value| value.to_ascii_lowercase())
        .collect();
    packet.normalized_symbols.iter().all(|symbol| {
        let candidates = canonical_symbol_candidates(symbol);
        packet.symbol_resolution_trace.iter().any(|trace| {
            trace
                .ambiguity_reason
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
                && trace
                    .canonical_symbol
                    .as_ref()
                    .is_some_and(|canonical| candidates.contains(&canonical.to_ascii_uppercase()))
                && allowed.contains(trace.mapping_confidence.as_policy_key())
        })
    })
}

pub(in crate::scoring) fn approved_universe_symbols(
    packet: &StructuredIntelPacket,
    snapshot: &SymbolUniverseSnapshot,
) -> bool {
    if packet.normalized_symbols.is_empty() {
        return false;
    }
    packet.normalized_symbols.iter().all(|symbol| {
        let candidates = canonical_symbol_candidates(symbol);
        snapshot.included_symbols.iter().any(|member| {
            candidates.contains(&member.symbol_canonical.to_ascii_uppercase())
                && member.approved_universe_symbol
        })
    })
}

pub(in crate::scoring) fn canonical_symbol_candidates(symbol: &str) -> BTreeSet<String> {
    let upper = symbol.trim().to_ascii_uppercase();
    let mut values = BTreeSet::from([upper.clone()]);
    for suffix in ["USDT", "USDC", "USD", "BUSD", "BTC", "ETH"] {
        if upper.len() > suffix.len() && upper.ends_with(suffix) {
            values.insert(upper.trim_end_matches(suffix).to_owned());
        }
    }
    values
}
