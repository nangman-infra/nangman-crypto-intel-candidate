use super::validation::{validate_output_dir, validate_output_key};
use crate::error::{AppError, AppResult};
use crate::model::CandidateProcessingResult;
use crate::scoring::{hypothesis_state_key, screening_event_key};
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

pub fn write_processing_results(
    output_dir: &Path,
    results: &[CandidateProcessingResult],
) -> AppResult<Vec<PathBuf>> {
    let mut written = Vec::new();
    for result in results {
        let screening_key = screening_event_key(
            result.screening_event.created_at_ms,
            &result.screening_event.screening_event_id,
        );
        written.push(write_record(
            output_dir,
            &screening_key,
            &result.screening_event,
        )?);
        if let Some(bundle) = &result.evidence_bundle {
            written.push(write_record(output_dir, &bundle.bundle_key, bundle)?);
        }
        if let Some(state) = &result.hypothesis_state {
            written.push(write_record(
                output_dir,
                &hypothesis_state_key(state.updated_at_ms, &state.hypothesis_id),
                state,
            )?);
        }
    }
    Ok(written)
}

pub(super) fn write_record<T>(output_dir: &Path, key: &str, record: &T) -> AppResult<PathBuf>
where
    T: Serialize,
{
    validate_output_dir(output_dir)?;
    validate_output_key(key)?;
    let path = output_dir.join(key);
    let mut file = create_output_file(output_dir, &path)?;
    serde_json::to_writer(&mut file, record)?;
    file.write_all(b"\n")?;
    Ok(path)
}

fn create_output_file(output_dir: &Path, path: &Path) -> AppResult<File> {
    let parent = path.parent().ok_or_else(|| {
        AppError::validation(format!("output path has no parent: {}", path.display()))
    })?;
    create_output_parent_dirs(output_dir, parent)?;
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(AppError::validation(format!(
            "output path must not be a symlink: {}",
            path.display()
        )));
    }
    Ok(File::create(path)?)
}

fn create_output_parent_dirs(output_dir: &Path, parent: &Path) -> AppResult<()> {
    ensure_directory_path(output_dir, "output dir")?;
    let relative_parent = parent.strip_prefix(output_dir).map_err(|_| {
        AppError::validation(format!(
            "output parent must stay under output dir: {}",
            parent.display()
        ))
    })?;

    let mut current = output_dir.to_path_buf();
    for component in relative_parent.components() {
        match component {
            Component::Normal(segment) => {
                current.push(segment);
                ensure_directory_path(&current, "output parent directory")?;
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                return Err(AppError::validation(format!(
                    "output parent contains unsafe path component: {}",
                    parent.display()
                )));
            }
        }
    }
    Ok(())
}

fn ensure_directory_path(path: &Path, label: &str) -> AppResult<()> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(AppError::validation(format!(
            "{label} must not be a symlink: {}",
            path.display()
        )));
    }
    if path.exists() && !path.is_dir() {
        return Err(AppError::validation(format!(
            "{label} must be a directory: {}",
            path.display()
        )));
    }
    fs::create_dir_all(path)?;
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(AppError::validation(format!(
            "{label} must not be a symlink: {}",
            path.display()
        )));
    }
    if !path.is_dir() {
        return Err(AppError::validation(format!(
            "{label} must be a directory: {}",
            path.display()
        )));
    }
    Ok(())
}
