use crate::error::{AppError, AppResult};
use crate::model::{
    CandidateProcessingResult, IntelCandidateEvidenceBundle, IntelCandidateHypothesisState,
    IntelCandidateScreeningEvent, MarketFeatureDelta, MarketRegimeContext, StructuredIntelPacket,
    SymbolUniverseSnapshot,
};
use crate::scoring::{hypothesis_state_key, screening_event_key};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;
mod validation;

use validation::{validate_output_dir, validate_output_key};

pub fn read_structured_packets(path: &Path) -> AppResult<Vec<StructuredIntelPacket>> {
    read_json_array_or_jsonl(path)
}

pub fn read_universe_snapshot(path: &Path) -> AppResult<SymbolUniverseSnapshot> {
    read_json_value(path)
}

pub fn read_market_feature_deltas(path: &Path) -> AppResult<Vec<MarketFeatureDelta>> {
    read_json_array_or_jsonl(path)
}

pub fn read_market_regime_contexts(path: &Path) -> AppResult<Vec<MarketRegimeContext>> {
    read_json_array_or_jsonl(path)
}

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

fn read_json_array_or_jsonl<T>(path: &Path) -> AppResult<Vec<T>>
where
    T: DeserializeOwned,
{
    let bytes = fs::read(path)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| AppError::Json(format!("{}: {error}", path.display())))?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(format!(
            "{} must not be empty",
            path.display()
        )));
    }
    if trimmed.starts_with('[') {
        return Ok(serde_json::from_str(trimmed)?);
    }
    if trimmed.starts_with('{')
        && let Ok(value) = serde_json::from_str(trimmed)
    {
        return Ok(vec![value]);
    }
    let mut values = Vec::new();
    for (index, line) in trimmed.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        values.push(serde_json::from_str(line).map_err(|error| {
            AppError::Json(format!(
                "{} line {} is not valid JSON: {error}",
                path.display(),
                index + 1
            ))
        })?);
    }
    Ok(values)
}

fn read_json_value<T>(path: &Path) -> AppResult<T>
where
    T: DeserializeOwned,
{
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn write_record<T>(output_dir: &Path, key: &str, record: &T) -> AppResult<PathBuf>
where
    T: Serialize,
{
    validate_output_dir(output_dir)?;
    validate_output_key(key)?;
    let path = output_dir.join(key);
    let parent = path.parent().ok_or_else(|| {
        AppError::validation(format!("output path has no parent: {}", path.display()))
    })?;
    fs::create_dir_all(parent)?;
    let mut file = File::create(&path)?;
    serde_json::to_writer(&mut file, record)?;
    file.write_all(b"\n")?;
    Ok(path)
}

#[allow(dead_code)]
fn _assert_record_types(
    _: &IntelCandidateScreeningEvent,
    _: &IntelCandidateEvidenceBundle,
    _: &IntelCandidateHypothesisState,
) {
}
