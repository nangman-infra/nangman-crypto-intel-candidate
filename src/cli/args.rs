use super::help::help_text;
use crate::error::{AppError, AppResult};
use crate::path_validation::validate_unambiguous_absolute_path;
use crate::policy::DEFAULT_POLICY_PATH;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub input_file: PathBuf,
    pub policy_file: PathBuf,
    pub universe_snapshot_file: Option<PathBuf>,
    pub market_feature_delta_file: Option<PathBuf>,
    pub market_regime_context_file: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub now_ms: Option<i64>,
}

pub fn parse_args<I>(mut values: I) -> AppResult<Option<Args>>
where
    I: Iterator<Item = String>,
{
    let mut args = Args {
        input_file: PathBuf::new(),
        policy_file: PathBuf::from(DEFAULT_POLICY_PATH),
        universe_snapshot_file: None,
        market_feature_delta_file: None,
        market_regime_context_file: None,
        output_dir: None,
        now_ms: None,
    };

    while let Some(arg) = values.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--input-file" => {
                args.input_file =
                    absolute_path_arg(values.next(), "--input-file requires an absolute path")?;
            }
            "--policy-file" => {
                args.policy_file =
                    absolute_path_arg(values.next(), "--policy-file requires an absolute path")?;
            }
            "--universe-snapshot-file" => {
                args.universe_snapshot_file = Some(absolute_path_arg(
                    values.next(),
                    "--universe-snapshot-file requires an absolute path",
                )?);
            }
            "--market-feature-delta-file" => {
                args.market_feature_delta_file = Some(absolute_path_arg(
                    values.next(),
                    "--market-feature-delta-file requires an absolute path",
                )?);
            }
            "--market-regime-context-file" => {
                args.market_regime_context_file = Some(absolute_path_arg(
                    values.next(),
                    "--market-regime-context-file requires an absolute path",
                )?);
            }
            "--output-dir" => {
                args.output_dir = Some(absolute_path_arg(
                    values.next(),
                    "--output-dir requires an absolute path",
                )?);
            }
            "--now-ms" => {
                let raw = values
                    .next()
                    .ok_or_else(|| AppError::config("--now-ms requires a number"))?;
                let value = raw
                    .parse::<i64>()
                    .map_err(|_| AppError::config("--now-ms must be an integer"))?;
                if value < 0 {
                    return Err(AppError::config("--now-ms must be non-negative"));
                }
                args.now_ms = Some(value);
            }
            other => {
                return Err(AppError::config(format!(
                    "unknown argument: {other}\n\n{}",
                    help_text()
                )));
            }
        }
    }

    if args.input_file.as_os_str().is_empty() {
        return Err(AppError::config("--input-file is required"));
    }
    Ok(Some(args))
}

fn absolute_path_arg(value: Option<String>, message: &str) -> AppResult<PathBuf> {
    let value = value.ok_or_else(|| AppError::config(message))?;
    let path = PathBuf::from(value);
    let label = message.split(" requires").next().unwrap_or("path");
    validate_unambiguous_absolute_path(&path, label)
        .map_err(|error| AppError::config(format!("{message}; {error}")))?;
    Ok(path)
}
