use crate::error::{AppError, AppResult};
use crate::policy::DEFAULT_POLICY_PATH;
use std::path::PathBuf;

pub fn worker_help() -> String {
    format!(
        r#"intel-candidate-worker
Usage:
  intel-candidate-worker \
    --nats-url nats://REPLACE_WITH_S2S_NATS_HOST:4222 \
    --input-s3-bucket nangman-crypto-dev-intel-structuring-l1-<account-suffix> \
    --output-s3-bucket nangman-crypto-dev-intel-candidate-<account-suffix> \
    --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
    --policy-file {DEFAULT_POLICY_PATH}

The worker reads structured_intel_packet.created pointers, writes screening events
and research-eligible candidate evidence bundles idempotently. Non-research
candidates with enough lineage are preserved as hypothesis_state records so
the recomposition loop can rerun them after market, policy, or code changes.
The worker publishes output pointers and only then acknowledges the input
message."#
    )
}

pub(super) fn next_string(
    values: &mut impl Iterator<Item = String>,
    message: &'static str,
) -> AppResult<String> {
    let value = values.next().ok_or_else(|| AppError::config(message))?;
    if value.trim().is_empty() {
        return Err(AppError::config(message));
    }
    Ok(value)
}

pub(super) fn validate_bucket_arg(value: &str, name: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::config(format!("{name} requires a bucket")));
    }
    if value.contains('<') || value.contains('>') {
        return Err(AppError::config(format!(
            "{name} must be a real bucket name, not a public-doc placeholder"
        )));
    }
    Ok(())
}

pub(super) fn absolute_path_arg(value: Option<String>, message: &str) -> AppResult<PathBuf> {
    let value = value.ok_or_else(|| AppError::config(message))?;
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(AppError::config(format!(
            "{message}; got {}",
            path.display()
        )));
    }
    Ok(path)
}

pub(super) fn positive_u64_arg(value: Option<String>, name: &str) -> AppResult<u64> {
    let raw = value.ok_or_else(|| AppError::config(format!("{name} requires a number")))?;
    let parsed = raw
        .parse::<u64>()
        .map_err(|_| AppError::config(format!("{name} must be a positive integer")))?;
    if parsed == 0 {
        return Err(AppError::config(format!(
            "{name} must be greater than zero"
        )));
    }
    Ok(parsed)
}

pub(super) fn positive_i64_arg(value: Option<String>, name: &str) -> AppResult<i64> {
    let raw = value.ok_or_else(|| AppError::config(format!("{name} requires a number")))?;
    let parsed = raw
        .parse::<i64>()
        .map_err(|_| AppError::config(format!("{name} must be a positive integer")))?;
    if parsed <= 0 {
        return Err(AppError::config(format!(
            "{name} must be greater than zero"
        )));
    }
    Ok(parsed)
}

pub(super) fn positive_usize_arg(value: Option<String>, name: &str) -> AppResult<usize> {
    let raw = value.ok_or_else(|| AppError::config(format!("{name} requires a number")))?;
    let parsed = raw
        .parse::<usize>()
        .map_err(|_| AppError::config(format!("{name} must be a positive integer")))?;
    if parsed == 0 {
        return Err(AppError::config(format!(
            "{name} must be greater than zero"
        )));
    }
    Ok(parsed)
}
