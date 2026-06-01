use super::types::{DEFAULT_REPAIR_MAX_PAGES_PER_PREFIX, DEFAULT_REPAIR_RECENT_PARTITION_DAYS};
use super::*;

#[test]
fn parse_agent_flags_and_worker_flags() {
    let args = AgentArgs::parse(
        [
            "--nats-url",
            "nats://127.0.0.1:4222",
            "--input-s3-bucket",
            "test-structured-l1",
            "--output-s3-bucket",
            "test-candidate",
            "--market-l1-s3-bucket",
            "test-market-l1",
            "--repair-input-prefix",
            "structured-intel-packet/schema=structured_intel_packet_v1/",
            "--repair-interval-secs",
            "60",
            "--repair-max-keys-per-prefix",
            "11",
            "--repair-max-pages-per-prefix",
            "3",
            "--repair-recent-partition-days",
            "2",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect("args parse")
    .expect("args present");
    assert_eq!(args.worker.nats.url, "nats://127.0.0.1:4222");
    assert_eq!(
        args.repair_input_prefixes,
        vec!["structured-intel-packet/schema=structured_intel_packet_v1/"]
    );
    assert_eq!(args.repair_interval_secs, 60);
    assert_eq!(args.repair_max_keys_per_prefix, 11);
    assert_eq!(args.repair_max_pages_per_prefix, 3);
    assert_eq!(args.repair_recent_partition_days, 2);
    assert!(args.repair_enabled);
}

#[test]
fn parse_disable_repair() {
    let args = AgentArgs::parse(
        [
            "--nats-url",
            "nats://127.0.0.1:4222",
            "--input-s3-bucket",
            "test-structured-l1",
            "--output-s3-bucket",
            "test-candidate",
            "--market-l1-s3-bucket",
            "test-market-l1",
            "--disable-repair",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect("args parse")
    .expect("args present");
    assert!(!args.repair_enabled);
    assert_eq!(
        args.repair_max_pages_per_prefix,
        DEFAULT_REPAIR_MAX_PAGES_PER_PREFIX
    );
    assert_eq!(
        args.repair_recent_partition_days,
        DEFAULT_REPAIR_RECENT_PARTITION_DAYS
    );
}

#[test]
fn rejects_unsafe_repair_prefix() {
    let err = AgentArgs::parse(
        [
            "--nats-url",
            "nats://127.0.0.1:4222",
            "--input-s3-bucket",
            "test-structured-l1",
            "--output-s3-bucket",
            "test-candidate",
            "--market-l1-s3-bucket",
            "test-market-l1",
            "--repair-input-prefix",
            "s3://bucket/structured-intel-packet/",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .unwrap_err();

    assert!(err.to_string().contains("--repair-input-prefix"));
    assert!(err.to_string().contains("object key"));
}
