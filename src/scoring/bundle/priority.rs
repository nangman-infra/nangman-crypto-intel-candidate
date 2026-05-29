use super::super::*;

pub(super) fn research_priority(
    event_type: &EventType,
    confidence_band: &ConfidenceBand,
) -> String {
    match (event_type, confidence_band) {
        (
            EventType::Incident | EventType::Regulatory,
            ConfidenceBand::High | ConfidenceBand::Strong,
        ) => "p0_event_risk".to_owned(),
        (EventType::FundingShift, ConfidenceBand::High | ConfidenceBand::Strong) => {
            "p1_derivatives".to_owned()
        }
        (_, ConfidenceBand::High | ConfidenceBand::Strong) => "p1_high_confidence".to_owned(),
        _ => "p2_standard".to_owned(),
    }
}

pub(in crate::scoring) fn research_priority_partition(research_priority: &str) -> &str {
    match research_priority.split_once('_') {
        Some((partition, _)) if matches!(partition, "p0" | "p1" | "p2") => partition,
        _ => "p2",
    }
}
