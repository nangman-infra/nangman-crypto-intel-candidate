use crate::error::{AppError, AppResult};

const MAX_S3_OBJECT_KEY_BYTES: usize = 1024;
const MIN_S3_BUCKET_NAME_BYTES: usize = 3;
const MAX_S3_BUCKET_NAME_BYTES: usize = 63;
const RESERVED_BUCKET_PREFIXES: &[&str] = &["xn--", "sthree-", "amzn-s3-demo-"];
const RESERVED_BUCKET_SUFFIXES: &[&str] = &["-s3alias", "--ol-s3", ".mrap", "--x-s3", "--table-s3"];

pub fn validate_bucket_name(bucket: &str, label: &str) -> AppResult<()> {
    validate_bucket_presence(bucket, label)?;
    validate_bucket_length(bucket, label)?;
    validate_bucket_characters(bucket, label)?;
    validate_bucket_boundaries(bucket, label)?;
    validate_bucket_period_dash_sequence(bucket, label)?;
    validate_bucket_not_ipv4(bucket, label)?;
    validate_bucket_reserved_affixes(bucket, label)?;
    Ok(())
}

pub fn validate_object_key(key: &str, label: &str) -> AppResult<()> {
    validate_key_shape(key, label, false)
}

pub fn validate_object_prefix(prefix: &str, label: &str) -> AppResult<()> {
    validate_key_shape(prefix, label, true)
}

fn validate_bucket_presence(bucket: &str, label: &str) -> AppResult<()> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(format!("{label} is required")));
    }
    if bucket.contains('<') || bucket.contains('>') {
        return Err(AppError::config(format!(
            "{label} must be a real bucket name, not a public-doc placeholder"
        )));
    }
    Ok(())
}

fn validate_bucket_length(bucket: &str, label: &str) -> AppResult<()> {
    if bucket.len() < MIN_S3_BUCKET_NAME_BYTES || bucket.len() > MAX_S3_BUCKET_NAME_BYTES {
        return Err(AppError::config(format!(
            "{label} must be between {MIN_S3_BUCKET_NAME_BYTES} and {MAX_S3_BUCKET_NAME_BYTES} characters"
        )));
    }
    Ok(())
}

fn validate_bucket_characters(bucket: &str, label: &str) -> AppResult<()> {
    if !bucket
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '-'))
    {
        return Err(AppError::config(format!(
            "{label} must contain only lowercase letters, numbers, periods, or hyphens"
        )));
    }
    Ok(())
}

fn validate_bucket_boundaries(bucket: &str, label: &str) -> AppResult<()> {
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
    Ok(())
}

fn validate_bucket_period_dash_sequence(bucket: &str, label: &str) -> AppResult<()> {
    if bucket.contains("..") || bucket.contains(".-") || bucket.contains("-.") {
        return Err(AppError::config(format!(
            "{label} must not contain adjacent periods or dashes next to periods"
        )));
    }
    Ok(())
}

fn validate_bucket_not_ipv4(bucket: &str, label: &str) -> AppResult<()> {
    if is_ipv4_address_shape(bucket) {
        return Err(AppError::config(format!(
            "{label} must not be formatted as an IP address"
        )));
    }
    Ok(())
}

fn validate_bucket_reserved_affixes(bucket: &str, label: &str) -> AppResult<()> {
    if RESERVED_BUCKET_PREFIXES
        .iter()
        .any(|prefix| bucket.starts_with(prefix))
    {
        return Err(AppError::config(format!(
            "{label} must not use an S3 reserved prefix"
        )));
    }
    if RESERVED_BUCKET_SUFFIXES
        .iter()
        .any(|suffix| bucket.ends_with(suffix))
    {
        return Err(AppError::config(format!(
            "{label} must not use an S3 reserved suffix"
        )));
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::is_ipv4_address_shape;

    #[test]
    fn ipv4_shape_requires_exactly_four_numeric_parts() {
        assert!(is_ipv4_address_shape("192.168.5.4"));
        assert!(!is_ipv4_address_shape(""));
        assert!(!is_ipv4_address_shape("192"));
        assert!(!is_ipv4_address_shape("192.168"));
        assert!(!is_ipv4_address_shape("192.168.5"));
        assert!(!is_ipv4_address_shape("192.168.5.4.1"));
        assert!(!is_ipv4_address_shape("192.168.5.bucket"));
    }
}
