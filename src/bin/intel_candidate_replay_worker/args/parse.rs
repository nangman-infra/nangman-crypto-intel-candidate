use intel_candidate_app::error::{AppError, AppResult};
use intel_candidate_app::worker::WorkerArgs;

use super::types::{DEFAULT_MAX_KEYS, ReplayArgs};
use super::value::{next_value, parse_non_negative_i64, parse_positive_usize};

pub(crate) fn parse_args(values: impl Iterator<Item = String>) -> AppResult<Option<ReplayArgs>> {
    let raw = values.collect::<Vec<_>>();
    if raw.iter().any(|value| value == "-h" || value == "--help") {
        return Ok(None);
    }

    let mut input_prefixes = Vec::new();
    let mut max_keys_per_prefix = DEFAULT_MAX_KEYS;
    let mut created_at_ms = None;
    let mut report_prefix = "candidate-replay-report".to_owned();
    let mut result_prefix = "candidate-replay-result".to_owned();
    let mut fail_on_record_error = true;
    let mut write_artifacts = false;
    let mut worker_args = Vec::new();

    let mut index = 0usize;
    while index < raw.len() {
        match raw[index].as_str() {
            "--replay-input-prefix" => {
                index += 1;
                input_prefixes.push(next_value(&raw, index, "--replay-input-prefix")?);
            }
            "--replay-max-keys-per-prefix" => {
                index += 1;
                max_keys_per_prefix = parse_positive_usize(&next_value(
                    &raw,
                    index,
                    "--replay-max-keys-per-prefix",
                )?)?;
            }
            "--replay-created-at-ms" => {
                index += 1;
                created_at_ms = Some(parse_non_negative_i64(&next_value(
                    &raw,
                    index,
                    "--replay-created-at-ms",
                )?)?);
            }
            "--replay-report-prefix" => {
                index += 1;
                report_prefix = next_value(&raw, index, "--replay-report-prefix")?;
            }
            "--replay-result-prefix" => {
                index += 1;
                result_prefix = next_value(&raw, index, "--replay-result-prefix")?;
            }
            "--replay-continue-on-record-error" => {
                fail_on_record_error = false;
            }
            "--replay-write-artifacts" => {
                write_artifacts = true;
            }
            value => worker_args.push(value.to_owned()),
        }
        index += 1;
    }

    if input_prefixes.is_empty() {
        return Err(AppError::config("--replay-input-prefix is required"));
    }
    let Some(worker) = WorkerArgs::parse(worker_args.into_iter())? else {
        return Ok(None);
    };

    Ok(Some(ReplayArgs {
        worker,
        input_prefixes,
        max_keys_per_prefix,
        created_at_ms,
        report_prefix,
        result_prefix,
        fail_on_record_error,
        write_artifacts,
    }))
}
