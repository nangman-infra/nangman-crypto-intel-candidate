use super::*;

pub(super) fn packet_for_test() -> StructuredIntelPacket {
    serde_json::from_value(json!({
        "packet_id": "packet_001",
        "packet_family_id": "family_001",
        "raw_event_id": "raw_001",
        "cluster_id": "cluster_001",
        "source_event_ids": ["raw_001"],
        "schema_version": STRUCTURED_PACKET_SCHEMA_VERSION
    }))
    .expect("valid packet")
}
