use super::signals::AdmissionSignals;
use super::*;

pub(super) fn collect_quarantine_reasons(
    policy: &ScoringPolicy,
    signals: &AdmissionSignals,
) -> Vec<String> {
    let mut reasons = gated_reasons([(
        policy.hard_gates.forbid_invalid_schema && !signals.has_valid_schema,
        "invalid_schema_version",
    )]);
    if policy.hard_gates.forbid_forbidden_output_terms && !signals.forbidden_output_terms.is_empty()
    {
        reasons.push(format!(
            "forbidden_output_terms:{}",
            signals.forbidden_output_terms.join(",")
        ));
    }
    reasons
}

pub(super) fn collect_reject_reasons(
    policy: &ScoringPolicy,
    signals: &AdmissionSignals,
) -> Vec<String> {
    gated_reasons([
        (
            policy.hard_gates.forbid_missing_evidence && !signals.has_evidence,
            "missing_evidence",
        ),
        (
            policy.hard_gates.forbid_missing_lineage && !signals.has_lineage,
            "missing_lineage",
        ),
        (!signals.has_symbols, "missing_normalized_symbols"),
    ])
}

pub(super) fn collect_observe_reasons(
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    signals: &AdmissionSignals,
) -> Vec<String> {
    dedupe_strings(gated_reasons([
        (
            policy.hard_gates.require_decision_available_at_ms && !signals.has_required_times,
            "missing_replay_time_contract",
        ),
        (
            signals.has_required_times && !signals.has_valid_time_order,
            "invalid_replay_time_order",
        ),
        (
            policy.hard_gates.require_point_in_time_universe && !signals.has_point_in_time_universe,
            "missing_point_in_time_universe",
        ),
        (
            policy.hard_gates.require_approved_universe_for_research
                && !signals.approved_universe_symbol,
            "not_admitted_universe",
        ),
        (
            policy.hard_gates.require_data_quality_summary_for_research
                && !signals.has_data_quality_summary,
            "missing_data_quality_summary",
        ),
        (
            policy.hard_gates.require_market_feature_delta_for_research
                && !signals.has_market_feature_delta,
            "missing_market_feature_delta",
        ),
        (
            policy.hard_gates.require_market_regime_context_for_research
                && !signals.has_market_regime_context,
            "missing_market_regime_context",
        ),
        (
            policy.hard_gates.require_source_independence_for_research
                && !signals.has_source_independence,
            "missing_source_independence",
        ),
        (
            signals.has_source_independence && !signals.source_independence_ok_for_research,
            "insufficient_source_independence",
        ),
        (
            policy
                .hard_gates
                .require_symbol_resolution_trace_for_research
                && !signals.has_symbol_resolution_trace,
            "missing_symbol_resolution_trace",
        ),
        (
            signals.has_symbol_resolution_trace && !signals.symbol_resolution_ok_for_research,
            "weak_symbol_resolution",
        ),
        (
            policy
                .hard_gates
                .forbid_research_without_metric_delta_for_derivatives
                && packet.event_type.is_derivatives_like()
                && !signals.has_derivatives_metric_delta,
            "derivatives_metric_delta_missing",
        ),
        (
            !signals.market_context_allows_research,
            "market_context_not_research_admissible",
        ),
    ]))
}

fn gated_reasons<const N: usize>(checks: [(bool, &str); N]) -> Vec<String> {
    checks
        .into_iter()
        .filter(|(enabled, _reason)| *enabled)
        .map(|(_enabled, reason)| reason.to_owned())
        .collect()
}
