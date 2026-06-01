use super::super::super::args_support::{
    absolute_path_arg, next_string, positive_i64_arg, positive_u64_arg, positive_usize_arg,
    worker_help,
};
use super::super::WorkerArgs;
use crate::error::{AppError, AppResult};

pub(super) enum WorkerArgAction {
    Continue,
    Help,
}

pub(super) fn apply_worker_arg(
    arg: String,
    values: &mut impl Iterator<Item = String>,
    args: &mut WorkerArgs,
) -> AppResult<WorkerArgAction> {
    match arg.as_str() {
        "-h" | "--help" => Ok(WorkerArgAction::Help),
        "--nats-url" => {
            args.nats.url = next_string(values, "--nats-url requires a URL")?;
            Ok(WorkerArgAction::Continue)
        }
        "--input-stream" => {
            args.nats.input_stream = next_string(values, "--input-stream requires a stream name")?;
            Ok(WorkerArgAction::Continue)
        }
        "--input-subject" => {
            args.nats.input_subject = next_string(values, "--input-subject requires a subject")?;
            Ok(WorkerArgAction::Continue)
        }
        "--input-consumer" => {
            args.nats.input_consumer =
                next_string(values, "--input-consumer requires a durable name")?;
            Ok(WorkerArgAction::Continue)
        }
        "--output-stream" => {
            args.nats.output_stream =
                next_string(values, "--output-stream requires a stream name")?;
            Ok(WorkerArgAction::Continue)
        }
        "--bundle-subject" => {
            args.nats.bundle_subject = next_string(values, "--bundle-subject requires a subject")?;
            Ok(WorkerArgAction::Continue)
        }
        "--screening-subject" => {
            args.nats.screening_subject =
                next_string(values, "--screening-subject requires a subject")?;
            Ok(WorkerArgAction::Continue)
        }
        "--hypothesis-state-subject" => {
            args.nats.hypothesis_state_subject =
                next_string(values, "--hypothesis-state-subject requires a subject")?;
            Ok(WorkerArgAction::Continue)
        }
        "--input-s3-bucket" => {
            args.input_store.bucket = next_string(values, "--input-s3-bucket requires a bucket")?;
            Ok(WorkerArgAction::Continue)
        }
        "--output-s3-bucket" => {
            args.output_store.bucket = next_string(values, "--output-s3-bucket requires a bucket")?;
            Ok(WorkerArgAction::Continue)
        }
        "--market-l1-s3-bucket" => {
            args.market_store.bucket =
                next_string(values, "--market-l1-s3-bucket requires a bucket")?;
            Ok(WorkerArgAction::Continue)
        }
        "--aws-region" => {
            let region = next_string(values, "--aws-region requires a region")?;
            args.input_store.region = region.clone();
            args.output_store.region = region.clone();
            args.market_store.region = region;
            Ok(WorkerArgAction::Continue)
        }
        "--input-s3-region" => {
            args.input_store.region = next_string(values, "--input-s3-region requires a region")?;
            Ok(WorkerArgAction::Continue)
        }
        "--output-s3-region" => {
            args.output_store.region = next_string(values, "--output-s3-region requires a region")?;
            Ok(WorkerArgAction::Continue)
        }
        "--market-l1-s3-region" => {
            args.market_store.region =
                next_string(values, "--market-l1-s3-region requires a region")?;
            Ok(WorkerArgAction::Continue)
        }
        "--aws-profile" => {
            let profile = Some(next_string(values, "--aws-profile requires a name")?);
            args.input_store.profile = profile.clone();
            args.output_store.profile = profile.clone();
            args.market_store.profile = profile;
            Ok(WorkerArgAction::Continue)
        }
        "--policy-file" => {
            args.policy_file =
                absolute_path_arg(values.next(), "--policy-file requires an absolute path")?;
            Ok(WorkerArgAction::Continue)
        }
        "--ack-wait-secs" => {
            args.nats.ack_wait_secs = positive_u64_arg(values.next(), "--ack-wait-secs")?;
            Ok(WorkerArgAction::Continue)
        }
        "--max-deliver" => {
            args.nats.max_deliver = positive_i64_arg(values.next(), "--max-deliver")?;
            Ok(WorkerArgAction::Continue)
        }
        "--batch-size" => {
            args.nats.batch_size = positive_usize_arg(values.next(), "--batch-size")?;
            Ok(WorkerArgAction::Continue)
        }
        "--max-messages" => {
            args.max_messages = Some(positive_usize_arg(values.next(), "--max-messages")?);
            Ok(WorkerArgAction::Continue)
        }
        "--exit-on-idle" => {
            args.exit_on_idle = true;
            Ok(WorkerArgAction::Continue)
        }
        "--no-ensure-output-stream" => {
            args.nats.ensure_output_stream = false;
            Ok(WorkerArgAction::Continue)
        }
        other => Err(AppError::config(format!(
            "unknown worker argument: {other}\n\n{}",
            worker_help()
        ))),
    }
}
