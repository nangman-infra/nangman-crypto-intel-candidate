use crate::error::{AppError, AppResult};

const MAX_S3_OBJECT_KEY_BYTES: usize = 1024;
const MIN_S3_BUCKET_NAME_BYTES: usize = 3;
const MAX_S3_BUCKET_NAME_BYTES: usize = 63;

pub fn validate_bucket_name(bucket: &str, label: &str) -> AppResult<()> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(format!("{label} is required")));
    }
    if bucket.contains('<') || bucket.contains('>') {
        return Err(AppError::config(format!(
            "{label} must be a real bucket name, not a public-doc placeholder"
        )));
    }
    if bucket.len() < MIN_S3_BUCKET_NAME_BYTES || bucket.len() > MAX_S3_BUCKET_NAME_BYTES {
        return Err(AppError::config(format!(
            "{label} must be between {MIN_S3_BUCKET_NAME_BYTES} and {MAX_S3_BUCKET_NAME_BYTES} characters"
        )));
    }
    if !bucket
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '-'))
    {
        return Err(AppError::config(format!(
            "{label} must contain only lowercase letters, numbers, periods, or hyphens"
        )));
    }
    if !bucket
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
        || !bucket
            .chars()
            .last()
            .is_some_and(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
    {
        return Err(AppError::config(format!(
            "{label} must begin and end with a lowercase letter or number"
        )));
    }
    if bucket.contains("..") || bucket.contains(".-") || bucket.contains("-.") {
        return Err(AppError::config(format!(
            "{label} must not contain adjacent periods or dashes next to periods"
        )));
    }
    if is_ipv4_address_shape(bucket) {
        return Err(AppError::config(format!(
            "{label} must not be formatted as an IP address"
        )));
    }
    if bucket.starts_with("xn--")
        || bucket.starts_with("sthree-")
        || bucket.starts_with("amzn-s3-demo-")
    {
        return Err(AppError::config(format!(
            "{label} must not use an S3 reserved prefix"
        )));
    }
    if bucket.ends_with("-s3alias")
        || bucket.ends_with("--ol-s3")
        || bucket.ends_with(".mrap")
        || bucket.ends_with("--x-s3")
        || bucket.ends_with("--table-s3")
    {
        return Err(AppError::config(format!(
            "{label} must not use an S3 reserved suffix"
        )));
    }
    Ok(())
}

pub fn validate_object_key(key: &str, label: &str) -> AppResult<()> {
    validate_key_shape(key, label, false)
}

pub fn validate_object_prefix(prefix: &str, label: &str) -> AppResult<()> {
    validate_key_shape(prefix, label, true)
}

fn validate_key_shape(value: &str, label: &str, allow_trailing_slash: bool) -> AppResult<()> {
    if value.is_empty() {
        return Err(AppError::validation(format!("{label} must not be empty")));
    }
    if value.trim() != value {
        return Err(AppError::validation(format!(
            "{label} must not include leading or trailing whitespace"
        )));
    }
    if value.len() > MAX_S3_OBJECT_KEY_BYTES {
        return Err(AppError::validation(format!(
            "{label} must be at most {MAX_S3_OBJECT_KEY_BYTES} bytes"
        )));
    }
    if value.starts_with('/') || value.to_ascii_lowercase().starts_with("s3://") {
        return Err(AppError::validation(format!(
            "{label} must be an object key, not a URI or absolute path"
        )));
    }
    if value.contains('?') || value.contains('#') {
        return Err(AppError::validation(format!(
            "{label} must not include query or fragment markers"
        )));
    }
    if value
        .chars()
        .any(|ch| ch.is_control() || ch.is_whitespace() || ch == '\\')
    {
        return Err(AppError::validation(format!(
            "{label} must not contain control characters, whitespace, or backslashes"
        )));
    }

    let normalized = if allow_trailing_slash {
        value.strip_suffix('/').unwrap_or(value)
    } else {
        value
    };
    if normalized.is_empty() || normalized.split('/').any(str::is_empty) {
        return Err(AppError::validation(format!(
            "{label} must not contain empty path segments"
        )));
    }
    if normalized
        .split('/')
        .any(|segment| matches!(segment, "." | ".."))
    {
        return Err(AppError::validation(format!(
            "{label} must not contain period-only path segments"
        )));
    }
    Ok(())
}

fn is_ipv4_address_shape(value: &str) -> bool {
    let mut parts = value.split('.');
    let Some(first) = parts.next() else {
        return false;
    };
    let Some(second) = parts.next() else {
        return false;
    };
    let Some(third) = parts.next() else {
        return false;
    };
    let Some(fourth) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }
    [first, second, third, fourth]
        .iter()
        .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
}
