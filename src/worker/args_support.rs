use crate::error::{AppError, AppResult};
use crate::path_validation::validate_unambiguous_absolute_path;
use crate::policy::DEFAULT_POLICY_PATH;
use crate::storage::validate_bucket_name;
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
    validate_bucket_name(value, name)
}

pub(super) fn validate_nats_url_arg(value: String, name: &str) -> AppResult<String> {
    if value.trim().is_empty() {
        return Err(AppError::config(format!("{name} is required")));
    }
    if value
        .chars()
        .any(|ch| ch.is_control() || ch.is_whitespace())
    {
        return Err(AppError::config(format!(
            "{name} must not contain whitespace or control characters"
        )));
    }
    let has_supported_scheme = ["nats://", "tls://", "ws://", "wss://"]
        .iter()
        .any(|scheme| {
            value
                .strip_prefix(scheme)
                .is_some_and(|server| !server.is_empty())
        });
    if !has_supported_scheme {
        return Err(AppError::config(format!(
            "{name} must start with nats://, tls://, ws://, or wss:// and include a server"
        )));
    }
    Ok(value)
}

pub(super) fn absolute_path_arg(value: Option<String>, message: &str) -> AppResult<PathBuf> {
    let value = value.ok_or_else(|| AppError::config(message))?;
    let path = PathBuf::from(value);
    let label = message.split(" requires").next().unwrap_or("path");
    validate_unambiguous_absolute_path(&path, label)
        .map_err(|error| AppError::config(format!("{message}; {error}")))?;
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
