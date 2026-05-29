#[path = "args/help.rs"]
mod help;
#[path = "args/parse.rs"]
mod parse;
#[path = "args/types.rs"]
mod types;
#[path = "args/value.rs"]
mod value;

pub(super) use help::replay_help;
pub(super) use parse::parse_args;
pub(super) use types::ReplayArgs;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_replay_flags_and_worker_flags() {
        let args = parse_args(
            [
                "--nats-url",
                "nats://127.0.0.1:4222",
                "--input-s3-bucket",
                "test-structured-l1",
                "--output-s3-bucket",
                "test-candidate",
                "--market-l1-s3-bucket",
                "test-market-l1",
                "--replay-input-prefix",
                "structured-intel-packet/schema=structured_intel_packet_v1/",
                "--replay-max-keys-per-prefix",
                "7",
                "--replay-created-at-ms",
                "123",
                "--replay-result-prefix",
                "candidate-replay/custom",
                "--replay-write-artifacts",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect("args parse")
        .expect("args present");
        assert_eq!(
            args.input_prefixes,
            vec!["structured-intel-packet/schema=structured_intel_packet_v1/"]
        );
        assert_eq!(args.max_keys_per_prefix, 7);
        assert_eq!(args.created_at_ms, Some(123));
        assert_eq!(args.result_prefix, "candidate-replay/custom");
        assert!(args.write_artifacts);
        assert_eq!(args.worker.nats.url, "nats://127.0.0.1:4222");
    }

    #[test]
    fn replay_help_uses_compiled_policy_path() {
        assert!(replay_help().contains(intel_candidate_app::policy::DEFAULT_POLICY_PATH));
    }
}
