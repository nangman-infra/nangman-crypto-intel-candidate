use super::*;

pub(super) fn derivatives_numeric_baseline_resolved(
    packet: &StructuredIntelPacket,
    admission: &AdmissionState,
) -> bool {
    packet.event_type.is_derivatives_like() && admission.has_derivatives_metric_delta
}
