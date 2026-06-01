use super::super::args::AgentArgs;
use super::cursor::RepairScanCursors;
use super::prefixes::{MILLIS_PER_DAY, repair_prefixes_for_cycle};
use super::scan::take_unique_keys;
use std::collections::HashSet;

fn agent_args(repair_input_prefix: &str) -> AgentArgs {
    AgentArgs::parse(
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
            repair_input_prefix,
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect("args parse")
    .expect("args present")
}

#[test]
fn repair_prefixes_prioritize_recent_structured_partitions() {
    let mut args = agent_args("structured-intel-packet/schema=structured_intel_packet_v1/");
    args.repair_recent_partition_days = 2;

    assert_eq!(
        repair_prefixes_for_cycle(&args, MILLIS_PER_DAY),
        vec![
            "structured-intel-packet/schema=structured_intel_packet_v1/dt=1970-01-02/",
            "structured-intel-packet/schema=structured_intel_packet_v1/dt=1970-01-01/",
            "structured-intel-packet/schema=structured_intel_packet_v1/"
        ]
    );
}

#[test]
fn repair_prefixes_leave_generic_prefixes_unexpanded() {
    let args = agent_args("custom-prefix/");

    assert_eq!(
        repair_prefixes_for_cycle(&args, MILLIS_PER_DAY),
        vec!["custom-prefix/"]
    );
}

#[test]
fn take_unique_keys_preserves_first_seen_order() {
    let mut seen = HashSet::new();

    let mut keys = take_unique_keys(
        &mut seen,
        vec![
            "recent-a".to_owned(),
            "recent-b".to_owned(),
            "recent-a".to_owned(),
        ],
    );
    keys.extend(take_unique_keys(
        &mut seen,
        vec!["old-a".to_owned(), "recent-b".to_owned()],
    ));

    assert_eq!(keys, vec!["recent-a", "recent-b", "old-a"]);
}

#[test]
fn repair_cursor_advances_until_page_exhaustion() {
    let mut cursors = RepairScanCursors::default();
    let prefix = "structured-intel-packet/schema=structured_intel_packet_v1/";

    assert_eq!(cursors.start_after(prefix), None);
    assert!(cursors.update_after_page(prefix, Some("key-000500.jsonl".to_owned())));
    assert_eq!(cursors.start_after(prefix), Some("key-000500.jsonl"));

    assert!(cursors.update_after_page(prefix, Some("key-001000.jsonl".to_owned())));
    assert_eq!(cursors.start_after(prefix), Some("key-001000.jsonl"));

    assert!(!cursors.update_after_page(prefix, None));
    assert_eq!(cursors.start_after(prefix), None);
}

#[test]
fn repair_cursors_are_tracked_per_prefix() {
    let mut cursors = RepairScanCursors::default();

    cursors.update_after_page("prefix-a/", Some("prefix-a/key-1.jsonl".to_owned()));
    cursors.update_after_page("prefix-b/", Some("prefix-b/key-1.jsonl".to_owned()));
    assert_eq!(
        cursors.start_after("prefix-a/"),
        Some("prefix-a/key-1.jsonl")
    );
    assert_eq!(
        cursors.start_after("prefix-b/"),
        Some("prefix-b/key-1.jsonl")
    );

    cursors.update_after_page("prefix-a/", None);
    assert_eq!(cursors.start_after("prefix-a/"), None);
    assert_eq!(
        cursors.start_after("prefix-b/"),
        Some("prefix-b/key-1.jsonl")
    );
}
