use super::*;
use crate::model::{
    MarketFeatureDelta, MarketFeatureDeltaSummary, STRUCTURED_PACKET_SCHEMA_VERSION,
    STRUCTURED_POINTER_SCHEMA_VERSION, StructuredIntelPacket,
};
use crate::nats::{S3ObjectPointer, StructuredPointer};
use serde_json::json;
use std::path::Path;

#[test]
fn worker_defaults_follow_candidate_contract_names() {
    let args = WorkerArgs::default();
    assert_eq!(args.nats.input_stream, "STRUCTURED_INTEL");
    assert_eq!(args.nats.output_stream, "INTEL_CANDIDATE");
    assert_eq!(
        args.nats.hypothesis_state_subject,
        "intel_candidate_hypothesis_state.created"
    );
    assert_eq!(args.output_store.bucket, DEFAULT_OUTPUT_BUCKET);
    assert_eq!(
        args.policy_file,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("policies/scoring-policy.v1.json")
    );
}

#[test]
fn parses_required_nats_url() {
    let args = WorkerArgs::parse(
        [
            "--nats-url",
            "nats://127.0.0.1:4222",
            "--input-s3-bucket",
            "test-structured-l1",
            "--output-s3-bucket",
            "test-candidate",
            "--market-l1-s3-bucket",
            "test-market-l1",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .unwrap()
    .unwrap();
    assert_eq!(args.nats.url, "nats://127.0.0.1:4222");
}

#[test]
fn accepts_nats_url_from_environment() {
    unsafe {
        std::env::set_var("NATS_URL", "nats://127.0.0.1:4222");
    }
    let args = WorkerArgs::parse(
        [
            "--input-s3-bucket",
            "test-structured-l1",
            "--output-s3-bucket",
            "test-candidate",
            "--market-l1-s3-bucket",
            "test-market-l1",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .unwrap()
    .unwrap();
    assert_eq!(args.nats.url, "nats://127.0.0.1:4222");
    unsafe {
        std::env::remove_var("NATS_URL");
    }
}

#[test]
fn rejects_public_doc_bucket_placeholder() {
    let err = WorkerArgs::parse(
        [
            "--nats-url",
            "nats://127.0.0.1:4222",
            "--input-s3-bucket",
            DEFAULT_INPUT_BUCKET,
            "--output-s3-bucket",
            "test-candidate",
            "--market-l1-s3-bucket",
            "test-market-l1",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .unwrap_err();

    assert!(err.to_string().contains("--input-s3-bucket"));
    assert!(err.to_string().contains("public-doc placeholder"));
}

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
fn revision_index_key_is_scoped_by_scoring_policy() {
    assert_eq!(
        revision_index_key("family/001", "intel_candidate_scoring_v2", 1),
        "candidate-revision-index/schema=intel_candidate_revision_index_v1/packet_family_id=family_001/scoring_policy=intel_candidate_scoring_v2/revision=0000000001.json"
    );
}

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

#[test]
fn expands_market_feature_delta_summary_for_candidate_scoring() {
    let deltas = expand_market_feature_delta_summary(MarketFeatureDeltaSummary {
        schema_version: "market_feature_delta_summary_v1".to_owned(),
        feature_delta_summary_id: "summary_001".to_owned(),
        l1_run_id: "l1_001".to_owned(),
        detail_feature_delta_key: "market_feature_delta/run_id=l1_001/delta.json".to_owned(),
        window_start_ms: 1_000,
        window_end_ms: 2_000,
        known_as_of_ms: 2_100,
        detail_record_count: 900,
        summary_row_count: 1,
        rows: vec![crate::model::MarketFeatureDeltaSummaryRow {
            venue: "binance".to_owned(),
            symbol_native: "SUIUSDT".to_owned(),
            symbol_canonical: "SUI".to_owned(),
            market_type: "spot".to_owned(),
            window_start_ms: 1_000,
            window_end_ms: 2_000,
            known_as_of_ms: 2_100,
            quality_status: "complete".to_owned(),
            missing_reasons: Vec::new(),
            metrics: vec![crate::model::MarketFeatureDeltaSummaryMetric {
                metric_name: "price".to_owned(),
                value_now: 1.25,
                value_15m_ago: Some(1.2),
                value_1h_ago: Some(1.1),
                change_pct_15m: Some(4.16),
                change_pct_1h: Some(13.63),
                price_change_same_window: Some(13.63),
                volume_change_same_window: Some(5.0),
                oi_price_divergence: None,
                window_start_ms: 1_000,
                window_end_ms: 2_000,
                quality_status: "complete".to_owned(),
            }],
        }],
    });

    assert_eq!(deltas.len(), 1);
    assert_eq!(deltas[0].schema_version, "market_feature_delta_summary_v1");
    assert_eq!(deltas[0].l1_run_id, "l1_001");
    assert_eq!(deltas[0].symbol_canonical, "SUI");
    assert_eq!(deltas[0].metric_name, "price");
    assert_eq!(deltas[0].known_as_of_ms, 2_100);
    assert!(
        deltas[0]
            .feature_delta_id
            .starts_with("market_delta_summary_metric_")
    );
}

#[test]
fn summary_delta_without_decision_safe_metric_requires_detail_fallback() {
    let packet: StructuredIntelPacket = serde_json::from_value(json!({
        "packet_id": "packet_001",
        "cluster_id": "cluster_001",
        "decision_available_at_ms": 2_000,
        "normalized_symbols": ["SUIUSDT"]
    }))
    .expect("minimal packet uses serde defaults");
    let future_delta = MarketFeatureDelta {
        schema_version: "market_feature_delta_summary_v1".to_owned(),
        feature_delta_id: "future_delta".to_owned(),
        l1_run_id: "l1_001".to_owned(),
        metric_name: "price".to_owned(),
        venue: "binance".to_owned(),
        symbol_native: "SUIUSDT".to_owned(),
        symbol_canonical: "SUI".to_owned(),
        market_type: "spot".to_owned(),
        value_now: 1.2,
        value_15m_ago: Some(1.1),
        value_1h_ago: Some(1.0),
        change_pct_15m: Some(9.0),
        change_pct_1h: Some(20.0),
        price_change_same_window: Some(20.0),
        volume_change_same_window: Some(5.0),
        oi_price_divergence: None,
        window_start_ms: 2_500,
        window_end_ms: 3_000,
        known_as_of_ms: 3_000,
        quality_status: "complete".to_owned(),
        missing_reasons: Vec::new(),
    };
    let valid_delta = MarketFeatureDelta {
        window_start_ms: 1_500,
        window_end_ms: 1_900,
        known_as_of_ms: 1_950,
        ..future_delta.clone()
    };

    assert!(!market_feature_deltas_satisfy_packet(
        &packet,
        &[future_delta]
    ));
    assert!(market_feature_deltas_satisfy_packet(
        &packet,
        &[valid_delta]
    ));
}

fn packet_for_test() -> StructuredIntelPacket {
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
