use super::validation::validate_output_key;
use super::write_record;
use serde_json::json;
use std::path::Path;

#[test]
fn write_record_rejects_relative_output_dir() {
    let error = write_record(
        Path::new("relative-output"),
        "candidate-screening/schema=v1/part-000001.jsonl",
        &json!({"ok": true}),
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("absolute path"));
}

#[test]
fn write_record_rejects_traversal_key() {
    let output_dir = std::env::temp_dir().join(format!(
        "intel-candidate-io-traversal-test-{}",
        std::process::id()
    ));
    let error = write_record(&output_dir, "../escape.jsonl", &json!({"ok": false}))
        .unwrap_err()
        .to_string();

    assert!(error.contains("output key"));
    assert!(!output_dir.join("escape.jsonl").exists());
}

#[test]
fn output_key_validation_accepts_normal_partitioned_key() {
    validate_output_key(
        "candidate-screening/schema=intel_candidate_screening_v1/dt=2026-05-29/hour=12/screening_event_id=screen_001/part-000001.jsonl",
    )
    .unwrap();
}

#[test]
fn output_key_validation_rejects_escape_shapes() {
    for key in [
        "/tmp/part-000001.jsonl",
        "candidate-screening/./part-000001.jsonl",
        "candidate-screening/../part-000001.jsonl",
        "candidate-screening\\part-000001.jsonl",
        "candidate-screening/\n/part-000001.jsonl",
    ] {
        let error = validate_output_key(key).unwrap_err().to_string();
        assert!(
            error.contains("output key"),
            "expected output key error for {key:?}, got {error}"
        );
    }
}
