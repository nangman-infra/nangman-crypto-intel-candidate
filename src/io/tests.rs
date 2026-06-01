use super::read::read_json_array_or_jsonl;
use super::validation::validate_output_key;
use super::write::write_record;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

fn unique_root(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "intel-candidate-io-{name}-{}-{nanos}",
        std::process::id()
    ))
}

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
fn write_record_rejects_ambiguous_absolute_output_dir() {
    let output_dir = std::env::temp_dir()
        .join("..")
        .join("intel-candidate-ambiguous-output");
    let error = write_record(
        &output_dir,
        "candidate-screening/schema=v1/part-000001.jsonl",
        &json!({"ok": true}),
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("relative path components"));
    assert!(!output_dir.join("candidate-screening").exists());
}

#[test]
fn write_record_rejects_traversal_key() {
    let output_dir = unique_root("traversal");
    let error = write_record(&output_dir, "../escape.jsonl", &json!({"ok": false}))
        .unwrap_err()
        .to_string();

    assert!(error.contains("output key"));
    assert!(!output_dir.join("escape.jsonl").exists());
}

#[cfg(unix)]
#[test]
fn write_record_rejects_symlink_output_file() {
    use std::os::unix::fs::symlink;

    let output_dir = unique_root("symlink-output");
    let outside = unique_root("symlink-output-outside");
    let outside_file = outside.join("outside.jsonl");
    let link_parent = output_dir.join("candidate-screening/schema=v1");
    fs::create_dir_all(&link_parent).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(&outside_file, b"outside\n").unwrap();
    symlink(&outside_file, link_parent.join("part-000001.jsonl")).unwrap();

    let error = write_record(
        &output_dir,
        "candidate-screening/schema=v1/part-000001.jsonl",
        &json!({"ok": false}),
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("output path must not be a symlink"));
    assert_eq!(fs::read(&outside_file).unwrap(), b"outside\n");
    fs::remove_dir_all(&output_dir).ok();
    fs::remove_dir_all(&outside).ok();
}

#[cfg(unix)]
#[test]
fn write_record_rejects_symlink_output_dir() {
    use std::os::unix::fs::symlink;

    let output_dir = unique_root("symlink-output-dir");
    let outside = unique_root("symlink-output-dir-outside");
    fs::create_dir_all(&outside).unwrap();
    symlink(&outside, &output_dir).unwrap();

    let error = write_record(
        &output_dir,
        "candidate-screening/schema=v1/part-000001.jsonl",
        &json!({"ok": false}),
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("output dir must not be a symlink"));
    assert!(!outside.join("candidate-screening").exists());
    fs::remove_file(&output_dir).ok();
    fs::remove_dir_all(&outside).ok();
}

#[cfg(unix)]
#[test]
fn write_record_rejects_symlink_output_parent_dir() {
    use std::os::unix::fs::symlink;

    let output_dir = unique_root("symlink-output-parent");
    let outside = unique_root("symlink-output-parent-outside");
    fs::create_dir_all(&output_dir).unwrap();
    fs::create_dir_all(&outside).unwrap();
    symlink(&outside, output_dir.join("candidate-screening")).unwrap();

    let error = write_record(
        &output_dir,
        "candidate-screening/schema=v1/part-000001.jsonl",
        &json!({"ok": false}),
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("output parent directory must not be a symlink"));
    assert!(!outside.join("schema=v1").exists());
    fs::remove_dir_all(&output_dir).ok();
    fs::remove_dir_all(&outside).ok();
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

#[test]
fn read_json_array_or_jsonl_accepts_array_single_object_and_jsonl() {
    let test_dir = std::env::temp_dir().join(format!(
        "intel-candidate-io-read-format-test-{}",
        std::process::id()
    ));
    fs::create_dir_all(&test_dir).unwrap();

    let array_path = test_dir.join("array.json");
    fs::write(&array_path, r#"[{"id":1},{"id":2}]"#).unwrap();
    let array_values: Vec<serde_json::Value> = read_json_array_or_jsonl(&array_path).unwrap();
    assert_eq!(array_values.len(), 2);

    let object_path = test_dir.join("object.json");
    fs::write(&object_path, r#"{"id":3}"#).unwrap();
    let object_values: Vec<serde_json::Value> = read_json_array_or_jsonl(&object_path).unwrap();
    assert_eq!(object_values, vec![json!({"id": 3})]);

    let jsonl_path = test_dir.join("records.jsonl");
    fs::write(&jsonl_path, "{\"id\":4}\n\n{\"id\":5}\n").unwrap();
    let jsonl_values: Vec<serde_json::Value> = read_json_array_or_jsonl(&jsonl_path).unwrap();
    assert_eq!(jsonl_values, vec![json!({"id": 4}), json!({"id": 5})]);
}
