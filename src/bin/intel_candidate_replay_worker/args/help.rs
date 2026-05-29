use intel_candidate_app::policy::DEFAULT_POLICY_PATH;
use intel_candidate_app::worker::worker_help;

use super::types::DEFAULT_MAX_KEYS;

pub(crate) fn replay_help() -> String {
    format!(
        r#"intel-candidate-replay-worker
Usage:
  intel-candidate-replay-worker \
    --nats-url nats://REPLACE_WITH_S2S_NATS_HOST:4222 \
    --input-s3-bucket nangman-crypto-dev-intel-structuring-l1-<account-suffix> \
    --output-s3-bucket nangman-crypto-dev-intel-candidate-<account-suffix> \
    --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
    --policy-file {DEFAULT_POLICY_PATH} \
    --replay-input-prefix structured-intel-packet/schema=structured_intel_packet_v1/

Replay-specific flags:
  --replay-input-prefix <s3-prefix>          Repeatable. Scans durable S3 inputs, not NATS retention.
  --replay-max-keys-per-prefix <positive>   Default: {DEFAULT_MAX_KEYS}
  --replay-created-at-ms <timestamp-ms>      Optional deterministic report/output time.
  --replay-report-prefix <s3-prefix>         Default: candidate-replay-report
  --replay-result-prefix <s3-prefix>         Default: candidate-replay-result
  --replay-write-artifacts                   Also writes screening/evidence/hypothesis artifacts for research handoff.
  --replay-continue-on-record-error          Write the report and exit 0 even when individual keys fail.

Worker flags:
{}
"#,
        worker_help()
    )
}
