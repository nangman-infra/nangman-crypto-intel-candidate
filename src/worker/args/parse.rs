use super::super::args_support::{
    absolute_path_arg, next_string, positive_i64_arg, positive_u64_arg, positive_usize_arg,
    validate_bucket_arg, worker_help,
};
use super::WorkerArgs;
use crate::error::{AppError, AppResult};

pub(super) fn parse_worker_args(
    mut values: impl Iterator<Item = String>,
) -> AppResult<Option<WorkerArgs>> {
    let mut args = WorkerArgs::default();
    while let Some(arg) = values.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--nats-url" => {
                args.nats.url = next_string(&mut values, "--nats-url requires a URL")?;
            }
            "--input-stream" => {
                args.nats.input_stream =
                    next_string(&mut values, "--input-stream requires a stream name")?;
            }
            "--input-subject" => {
                args.nats.input_subject =
                    next_string(&mut values, "--input-subject requires a subject")?;
            }
            "--input-consumer" => {
                args.nats.input_consumer =
                    next_string(&mut values, "--input-consumer requires a durable name")?;
            }
            "--output-stream" => {
                args.nats.output_stream =
                    next_string(&mut values, "--output-stream requires a stream name")?;
            }
            "--bundle-subject" => {
                args.nats.bundle_subject =
                    next_string(&mut values, "--bundle-subject requires a subject")?;
            }
            "--screening-subject" => {
                args.nats.screening_subject =
                    next_string(&mut values, "--screening-subject requires a subject")?;
            }
            "--hypothesis-state-subject" => {
                args.nats.hypothesis_state_subject =
                    next_string(&mut values, "--hypothesis-state-subject requires a subject")?;
            }
            "--input-s3-bucket" => {
                args.input_store.bucket =
                    next_string(&mut values, "--input-s3-bucket requires a bucket")?;
            }
            "--output-s3-bucket" => {
                args.output_store.bucket =
                    next_string(&mut values, "--output-s3-bucket requires a bucket")?;
            }
            "--market-l1-s3-bucket" => {
                args.market_store.bucket =
                    next_string(&mut values, "--market-l1-s3-bucket requires a bucket")?;
            }
            "--aws-region" => {
                let region = next_string(&mut values, "--aws-region requires a region")?;
                args.input_store.region = region.clone();
                args.output_store.region = region.clone();
                args.market_store.region = region;
            }
            "--input-s3-region" => {
                args.input_store.region =
                    next_string(&mut values, "--input-s3-region requires a region")?;
            }
            "--output-s3-region" => {
                args.output_store.region =
                    next_string(&mut values, "--output-s3-region requires a region")?;
            }
            "--market-l1-s3-region" => {
                args.market_store.region =
                    next_string(&mut values, "--market-l1-s3-region requires a region")?;
            }
            "--aws-profile" => {
                let profile = Some(next_string(&mut values, "--aws-profile requires a name")?);
                args.input_store.profile = profile.clone();
                args.output_store.profile = profile.clone();
                args.market_store.profile = profile;
            }
            "--policy-file" => {
                args.policy_file =
                    absolute_path_arg(values.next(), "--policy-file requires an absolute path")?;
            }
            "--ack-wait-secs" => {
                args.nats.ack_wait_secs = positive_u64_arg(values.next(), "--ack-wait-secs")?;
            }
            "--max-deliver" => {
                args.nats.max_deliver = positive_i64_arg(values.next(), "--max-deliver")?;
            }
            "--batch-size" => {
                args.nats.batch_size = positive_usize_arg(values.next(), "--batch-size")?;
            }
            "--max-messages" => {
                args.max_messages = Some(positive_usize_arg(values.next(), "--max-messages")?);
            }
            "--exit-on-idle" => {
                args.exit_on_idle = true;
            }
            "--no-ensure-output-stream" => {
                args.nats.ensure_output_stream = false;
            }
            other => {
                return Err(AppError::config(format!(
                    "unknown worker argument: {other}\n\n{}",
                    worker_help()
                )));
            }
        }
    }
    if args.nats.url.trim().is_empty() {
        args.nats.url = std::env::var("NATS_URL").unwrap_or_default();
    }
    if args.nats.url.trim().is_empty() {
        return Err(AppError::config("--nats-url or NATS_URL is required"));
    }
    validate_bucket_arg(&args.input_store.bucket, "--input-s3-bucket")?;
    validate_bucket_arg(&args.output_store.bucket, "--output-s3-bucket")?;
    validate_bucket_arg(&args.market_store.bucket, "--market-l1-s3-bucket")?;
    Ok(Some(args))
}
