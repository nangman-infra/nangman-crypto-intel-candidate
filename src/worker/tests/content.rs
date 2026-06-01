use super::*;

#[test]
fn reads_pretty_json_or_jsonl_first_record() {
    let json = br#"{"value":1}"#;
    let value: serde_json::Value = read_single_json_or_jsonl(json, Path::new("x")).unwrap();
    assert_eq!(value["value"], 1);

    let jsonl = br#"{"value":2}
{"value":3}
"#;
    let value: serde_json::Value = read_single_json_or_jsonl(jsonl, Path::new("x")).unwrap();
    assert_eq!(value["value"], 2);
}

#[test]
fn validates_pointer_content_hash_before_scoring() {
    let pointer = StructuredPointer {
        schema_version: STRUCTURED_POINTER_SCHEMA_VERSION.to_owned(),
        packet_id: "packet_001".to_owned(),
        raw_event_id: "raw_001".to_owned(),
        terminal_decision: serde_json::Value::String("high_confidence_structured".to_owned()),
        storage_ref: S3ObjectPointer {
            bucket: DEFAULT_INPUT_BUCKET.to_owned(),
            key: "structured-intel-packet/schema=structured_intel_packet_v1/x.jsonl".to_owned(),
            content_sha256: sha256_prefixed(b"payload"),
            schema_version: STRUCTURED_PACKET_SCHEMA_VERSION.to_owned(),
        },
        manifest_key: "manifests/schema=intel_l1_manifest_v1/run.json".to_owned(),
        created_at_ms: 1,
    };
    assert!(validate_pointer_content_hash(&pointer, b"payload").is_ok());
    assert!(validate_pointer_content_hash(&pointer, b"tampered").is_err());
}
