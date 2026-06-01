use super::*;

#[test]
fn repair_s3_key_recovers_raw_event_id_from_partition() {
    let mut packet = packet_for_test();
    packet.raw_event_id.clear();
    let key = "structured-intel-packet/schema=structured_intel_packet_v1/dt=2026-05-22/hour=04/raw_event_id=intel_evt_abc/packet_id=intel_pkt_123/part-000001.jsonl";
    assert_eq!(repair_raw_event_id(&packet, key), "intel_evt_abc");
}

#[test]
fn repair_s3_key_falls_back_to_packet_id_when_raw_event_id_is_missing() {
    let mut packet = packet_for_test();
    packet.raw_event_id.clear();
    assert_eq!(
        repair_raw_event_id(
            &packet,
            "structured-intel-packet/schema=structured_intel_packet_v1/x.jsonl"
        ),
        packet.packet_id
    );
}
