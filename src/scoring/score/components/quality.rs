use super::helpers::push_component;
use super::*;

pub(super) fn push_quality_score_components(
    components: &mut Vec<ScoreComponent>,
    packet: &StructuredIntelPacket,
    policy: &ScoringPolicy,
    admission: &AdmissionState,
) {
    if admission.has_data_quality_summary {
        push_component(
            components,
            "data_quality_good",
            policy.weight("data_quality_good"),
            "market data quality summary present",
        );
    }
    if packet
        .symbol_resolution_trace
        .iter()
        .any(|trace| trace.mapping_confidence.is_strong())
    {
        push_component(
            components,
            "symbol_resolution_strong",
            policy.weight("symbol_resolution_strong"),
            "strong symbol resolution trace",
        );
    }
    if admission.has_derivatives_metric_delta {
        push_component(
            components,
            "metric_delta_present",
            policy.weight("metric_delta_present"),
            "metric evidence includes delta_pct",
        );
    }
    push_component(
        components,
        "confidence_band",
        policy.weight(&format!(
            "confidence_{}",
            packet.confidence_band.as_policy_key()
        )),
        "packet confidence band",
    );
}
