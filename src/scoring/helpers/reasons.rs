use super::super::*;

pub(in crate::scoring) fn retryable_reasons(reasons: &[String]) -> Vec<String> {
    let retryable_markers = [
        "missing_market_feature_delta",
        "missing_market_regime_context",
        "market_context_not_research_admissible",
        "derivatives_metric_delta_missing",
        "baseline_missing",
        "single_numeric_snapshot",
        "single_source_only",
        "missing_source_independence",
        "insufficient_source_independence",
        "missing_symbol_resolution_trace",
        "weak_symbol_resolution",
    ];
    reasons
        .iter()
        .filter(|reason| retryable_markers.contains(&reason.as_str()))
        .cloned()
        .collect()
}

pub(in crate::scoring) fn terminal_reasons(reasons: &[String]) -> Vec<String> {
    let terminal_markers = [
        "invalid_schema_version",
        "invalid_replay_time_order",
        "forbidden_output_term",
        "not_admitted_universe",
    ];
    reasons
        .iter()
        .filter(|reason| {
            terminal_markers.contains(&reason.as_str())
                || reason.starts_with("forbidden_generated_term:")
        })
        .cloned()
        .collect()
}

pub(in crate::scoring) fn next_hypothesis_action(
    class: &CandidateClass,
    retryable_reasons: &[String],
) -> String {
    if matches!(class, CandidateClass::Reject) && retryable_reasons.is_empty() {
        "archive_until_revision".to_owned()
    } else if retryable_reasons.iter().any(|reason| {
        reason == "missing_market_feature_delta" || reason == "derivatives_metric_delta_missing"
    }) {
        "rerun_when_market_feature_delta_updates".to_owned()
    } else if retryable_reasons
        .iter()
        .any(|reason| reason == "market_context_not_research_admissible")
    {
        "rerun_when_market_context_updates".to_owned()
    } else {
        "enqueue_cheap_harness".to_owned()
    }
}
