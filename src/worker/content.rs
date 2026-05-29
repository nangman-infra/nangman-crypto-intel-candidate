use super::*;

pub(super) fn read_single_json_or_jsonl<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    display_path: &Path,
) -> AppResult<T> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| AppError::Json(format!("{}: {error}", display_path.display())))?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(format!(
            "{} is empty",
            display_path.display()
        )));
    }
    if trimmed.starts_with('{')
        && let Ok(value) = serde_json::from_str(trimmed)
    {
        return Ok(value);
    }
    let first_line = trimmed
        .lines()
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| {
            AppError::validation(format!("{} has no JSONL records", display_path.display()))
        })?;
    Ok(serde_json::from_str(first_line)?)
}

pub(super) fn sha256_prefixed(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256_hex(bytes))
}

pub(super) fn validate_pointer_content_hash(
    pointer: &StructuredPointer,
    bytes: &[u8],
) -> AppResult<()> {
    let actual = sha256_prefixed(bytes);
    if actual != pointer.storage_ref.content_sha256 {
        return Err(AppError::validation(format!(
            "structured packet content hash mismatch packet_id={} expected={} actual={}",
            pointer.packet_id, pointer.storage_ref.content_sha256, actual
        )));
    }
    Ok(())
}
