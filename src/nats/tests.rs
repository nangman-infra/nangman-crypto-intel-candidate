use super::consumer::deliver_policy;
use super::*;
use crate::model::{
    CANDIDATE_POINTER_SCHEMA_VERSION, SCREENING_EVENT_SCHEMA_VERSION,
    STRUCTURED_PACKET_SCHEMA_VERSION, STRUCTURED_POINTER_SCHEMA_VERSION,
};
use async_nats::jetstream::consumer::DeliverPolicy;
use serde_json::json;

#[test]
fn parses_supported_deliver_policies() {
    assert!(matches!(deliver_policy("all").unwrap(), DeliverPolicy::All));
    assert!(matches!(deliver_policy("new").unwrap(), DeliverPolicy::New));
    assert!(matches!(
        deliver_policy("last").unwrap(),
        DeliverPolicy::Last
    ));
    assert!(matches!(
        deliver_policy("last_per_subject").unwrap(),
        DeliverPolicy::LastPerSubject
    ));
}

#[test]
fn validates_structured_pointer_contract() {
    let pointer = StructuredPointer {
        schema_version: STRUCTURED_POINTER_SCHEMA_VERSION.to_owned(),
        packet_id: "packet_001".to_owned(),
        raw_event_id: "raw_001".to_owned(),
        terminal_decision: json!("high_confidence_structured"),
        storage_ref: S3ObjectPointer {
            bucket: "intel-l1".to_owned(),
            key: "structured-intel-packet/schema=structured_intel_packet_v1/x.jsonl".to_owned(),
            content_sha256: "sha256:abc".to_owned(),
            schema_version: STRUCTURED_PACKET_SCHEMA_VERSION.to_owned(),
        },
        manifest_key: "manifests/schema=intel_l1_manifest_v1/run.json".to_owned(),
        created_at_ms: 1,
    };
    assert!(pointer.validate().is_ok());
}

#[test]
fn rejects_legacy_structured_pointer_schema_name() {
    let legacy_schema = ["structured", "intel", "pointer", "v1"].join("_");
    let pointer = StructuredPointer {
        schema_version: legacy_schema,
        packet_id: "packet_001".to_owned(),
        raw_event_id: "raw_001".to_owned(),
        terminal_decision: json!("high_confidence_structured"),
        storage_ref: S3ObjectPointer {
            bucket: "intel-l1".to_owned(),
            key: "structured-intel-packet/schema=structured_intel_packet_v1/x.jsonl".to_owned(),
            content_sha256: "sha256:abc".to_owned(),
            schema_version: STRUCTURED_PACKET_SCHEMA_VERSION.to_owned(),
        },
        manifest_key: "manifests/schema=intel_l1_manifest_v1/run.json".to_owned(),
        created_at_ms: 1,
    };
    assert!(pointer.validate().is_err());
}

#[test]
fn validates_candidate_artifact_pointer_contract() {
    let pointer = CandidateArtifactPointer {
        schema_version: CANDIDATE_POINTER_SCHEMA_VERSION.to_owned(),
        artifact_family: "intel_candidate_screening_event".to_owned(),
        candidate_id: Some("cand_001".to_owned()),
        screening_event_id: "screen_001".to_owned(),
        candidate_class: "research_candidate".to_owned(),
        storage_ref: S3ObjectPointer {
            bucket: "candidate".to_owned(),
            key: "candidate-screening/schema=intel_candidate_screening_event_v1/x.jsonl".to_owned(),
            content_sha256: "sha256:def".to_owned(),
            schema_version: SCREENING_EVENT_SCHEMA_VERSION.to_owned(),
        },
        created_at_ms: 1,
    };
    assert!(pointer.validate().is_ok());
}
