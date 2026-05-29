use super::types::{
    DEFAULT_REPAIR_INTERVAL_SECS, DEFAULT_REPAIR_MAX_KEYS_PER_PREFIX,
    DEFAULT_REPAIR_MAX_PAGES_PER_PREFIX, DEFAULT_REPAIR_RECENT_PARTITION_DAYS,
};
use crate::policy::DEFAULT_POLICY_PATH;
use crate::worker::worker_help;

pub fn agent_help() -> String {
    format!(
        r#"intel-candidate-agent
Usage:
  intel-candidate-agent \
    --nats-url nats://REPLACE_WITH_S2S_NATS_HOST:4222 \
    --input-s3-bucket nangman-crypto-dev-intel-structuring-l1-<account-suffix> \
    --output-s3-bucket nangman-crypto-dev-intel-candidate-<account-suffix> \
    --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
    --policy-file {DEFAULT_POLICY_PATH} \
    --repair-input-prefix structured-intel-packet/schema=structured_intel_packet_v1/

The agent is the default AI-DLC execution unit. It continuously consumes live
structured intel pointers from NATS and runs bounded S3 repair scans inside the
same process when configured. S3 remains the canonical store; NATS remains the
pointer bus.

Agent-specific flags:
  --repair-input-prefix <s3-prefix>          Repeatable. Enables bounded S3 repair scans.
  --repair-interval-secs <positive>          Default: {DEFAULT_REPAIR_INTERVAL_SECS}
  --repair-max-keys-per-prefix <positive>    Default: {DEFAULT_REPAIR_MAX_KEYS_PER_PREFIX}
  --repair-max-pages-per-prefix <positive>   Default: {DEFAULT_REPAIR_MAX_PAGES_PER_PREFIX}
  --repair-recent-partition-days <count>     Default: {DEFAULT_REPAIR_RECENT_PARTITION_DAYS}. Adds recent dt=YYYY-MM-DD prefixes before broad scans.
  --disable-repair                           Disable repair scans even when prefixes are configured.

Worker flags:
{}
"#,
        worker_help()
    )
}
