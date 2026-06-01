use super::*;

#[test]
fn cli_reads_market_artifact_files_for_research_admission() {
    let root = test_root("artifact-files");
    let input_file = root.join("structured.json");
    let universe_file = root.join("universe.json");
    let delta_file = root.join("delta.json");
    let regime_file = root.join("regime.json");
    let output_dir = root.join("out");
    write_json(&input_file, &json!([packet_json()]));
    write_json(&universe_file, &universe_json());
    write_json(&delta_file, &market_feature_delta_json());
    write_json(&regime_file, &market_regime_context_json());

    let args = parse_args(
        [
            "--input-file",
            input_file.to_str().expect("utf8 path"),
            "--policy-file",
            test_policy_path().to_str().expect("utf8 path"),
            "--universe-snapshot-file",
            universe_file.to_str().expect("utf8 path"),
            "--market-feature-delta-file",
            delta_file.to_str().expect("utf8 path"),
            "--market-regime-context-file",
            regime_file.to_str().expect("utf8 path"),
            "--output-dir",
            output_dir.to_str().expect("utf8 path"),
            "--now-ms",
            "7200000",
        ]
        .into_iter()
        .map(ToOwned::to_owned),
    )
    .expect("args parse")
    .expect("args exist");

    let summary = run(args).expect("cli run succeeds");

    assert_eq!(summary.processed_packets, 1);
    assert_eq!(summary.evidence_bundles_created, 1);
    assert_eq!(summary.hypothesis_states_created, 0);
    assert_eq!(summary.output_files.len(), 2);
    fs::remove_dir_all(root).ok();
}

#[test]
fn cli_without_market_artifact_files_blocks_research_admission() {
    let root = test_root("missing-artifact-files");
    let input_file = root.join("structured.json");
    let universe_file = root.join("universe.json");
    let output_dir = root.join("out");
    write_json(&input_file, &json!([packet_json()]));
    write_json(&universe_file, &universe_json());

    let args = Args {
        input_file,
        policy_file: test_policy_path(),
        universe_snapshot_file: Some(universe_file),
        market_feature_delta_file: None,
        market_regime_context_file: None,
        output_dir: Some(output_dir),
        now_ms: Some(7_200_000),
    };
    let summary = run(args).expect("cli run succeeds");

    assert_eq!(summary.processed_packets, 1);
    assert_eq!(summary.evidence_bundles_created, 0);
    assert_eq!(summary.hypothesis_states_created, 1);
    assert_eq!(summary.output_files.len(), 2);
    fs::remove_dir_all(root).ok();
}
