use crate::error::{AppError, AppResult};
use std::path::{Component, Path};

const MAX_OUTPUT_KEY_BYTES: usize = 1024;

pub(super) fn validate_output_dir(output_dir: &Path) -> AppResult<()> {
    if output_dir.is_absolute() {
        Ok(())
    } else {
        Err(AppError::validation(format!(
            "output dir must be an absolute path: {}",
            output_dir.display()
        )))
    }
}

pub(super) fn validate_output_key(key: &str) -> AppResult<()> {
    if key.is_empty() {
        return Err(AppError::validation("output key is required"));
    }
    if key.len() > MAX_OUTPUT_KEY_BYTES {
        return Err(AppError::validation(format!(
            "output key must be at most {MAX_OUTPUT_KEY_BYTES} bytes"
        )));
    }
    if key.chars().any(|ch| ch.is_control() || ch == '\\') {
        return Err(AppError::validation(
            "output key must not contain control characters or backslashes",
        ));
    }
    if key.split('/').any(|segment| matches!(segment, "." | "..")) {
        return Err(AppError::validation(
            "output key must not contain period-only path segments",
        ));
    }
    for component in Path::new(key).components() {
        match component {
            Component::Normal(_) => {}
            Component::Prefix(_)
            | Component::RootDir
            | Component::CurDir
            | Component::ParentDir => {
                return Err(AppError::validation(
                    "output key must be a relative object key without path traversal",
                ));
            }
        }
    }
    Ok(())
}
