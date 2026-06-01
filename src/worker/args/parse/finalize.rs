use super::super::super::args_support::{validate_bucket_arg, validate_nats_url_arg};
use super::super::WorkerArgs;
use crate::error::{AppError, AppResult};

pub(super) fn finalize_worker_args(
    mut args: WorkerArgs,
    nats_url_env: impl FnOnce() -> Option<String>,
) -> AppResult<WorkerArgs> {
    if args.nats.url.trim().is_empty() {
        args.nats.url = nats_url_env().unwrap_or_default();
    }
    if args.nats.url.trim().is_empty() {
        return Err(AppError::config("--nats-url or NATS_URL is required"));
    }
    args.nats.url = validate_nats_url_arg(args.nats.url, "--nats-url or NATS_URL")?;
    validate_bucket_arg(&args.input_store.bucket, "--input-s3-bucket")?;
    validate_bucket_arg(&args.output_store.bucket, "--output-s3-bucket")?;
    validate_bucket_arg(&args.market_store.bucket, "--market-l1-s3-bucket")?;
    Ok(args)
}
