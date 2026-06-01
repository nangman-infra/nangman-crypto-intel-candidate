use crate::time::now_ms;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub(in crate::cli::tests) fn test_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "intel-candidate-cli-{name}-{}-{}",
        std::process::id(),
        now_ms()
    ))
}

pub(in crate::cli::tests) fn test_policy_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("policies/scoring-policy.v1.json")
}

pub(in crate::cli::tests) fn write_json(path: &Path, value: &Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("test parent directory is created");
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("test json serializes"),
    )
    .expect("test json is written");
}
