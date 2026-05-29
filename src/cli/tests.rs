use super::*;
use crate::time::now_ms;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

fn test_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "intel-candidate-cli-{name}-{}-{}",
        std::process::id(),
        now_ms()
    ))
}

fn test_policy_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("policies/scoring-policy.v1.json")
}

fn write_json(path: &Path, value: &Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("test parent directory is created");
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("test json serializes"),
    )
    .expect("test json is written");
}

fn packet_json() -> Value {
    json!({
        "packet_id": "packet_cli_001",
        "cluster_id": "cluster_cli_001",
        "source_event_ids": ["source_cli_001"],
        "published_at_ms": 1000,
        "fetched_at_ms": 1100,
        "structured_at_ms": 1200,
        "decision_available_at_ms": 1300,
        "normalized_symbols": ["SUI"],
        "symbol_confidence_band": "strong",
        "symbol_resolution_trace": [{
            "raw_mentions": ["SUI"],
            "resolved_project": "Sui",
            "resolved_asset": "SUI",
            "canonical_symbol": "SUI",
            "venue_symbols": ["SUIUSDT"],
            "mapping_confidence": "strong"
        }],
        "event_type": "incident",
        "topic_summary": "official incident notice",
        "stance_summary": "risk watch",
        "risk_summary": "operational risk",
        "regime_hint": "neutral",
        "scenario_hint": "observe response",
        "confidence_band": "high",
        "novelty_score": 0.9,
        "source_quality_summary": "official_notice",
        "source_independence_summary": {
            "source_event_count": 1,
            "independent_source_count": 1,
            "official_source_present": true,
            "duplicate_content_hashes": [],
            "original_source_ids": ["official"]
        },
        "text_evidence": [
            {
                "evidence_text": "Official incident notice was published.",
                "source_event_id": "source_cli_001",
                "source_id": "official",
                "published_at_ms": 1000,
                "evidence_kind": "source_sentence"
            },
            {
                "evidence_text": "The notice names SUI directly.",
                "source_event_id": "source_cli_001",
                "source_id": "official",
                "published_at_ms": 1000,
                "evidence_kind": "source_sentence"
            }
        ],
        "market_context_status": "available_symbol_context",
        "market_context_ref": {
            "status": "available_symbol_context",
            "basis_timestamp_ms": 1300,
            "basis_kind": "exact",
            "window_start_ms": 1000,
            "window_end_ms": 2000,
            "manifest_key": "normalized-market-slice/manifest.json",
            "output_object_keys": ["normalized-market-slice/part.jsonl"],
            "market_data_quality_summary_key": "quality/summary.json",
            "market_feature_delta_key": "market-feature-delta/delta.json",
            "market_feature_delta_summary_key": "market-feature-delta-summary/summary.json",
            "market_regime_context_key": "market-regime/context.json",
            "symbol_universe_snapshot_key": "universe/snapshot.json"
        },
        "model_tier_used": "haiku",
        "terminal_decision": "structured_only",
        "schema_version": "structured_intel_packet_v1"
    })
}

fn universe_json() -> Value {
    json!({
        "schema_version": "symbol_universe_snapshot_v1",
        "symbol_universe_snapshot_id": "universe_cli_001",
        "universe_as_of_ms": 1000,
        "included_symbols": [{
            "symbol_canonical": "SUI",
            "execution_symbol_native": "SUIUSDT",
            "reference_symbol_native": "SUIUSDT",
            "liquidity_rank_at_that_time": 1,
            "approved_universe_symbol": true,
            "bootstrap_days_available": 30,
            "median_spread_bps_30d": 2.0,
            "median_traded_notional_30d": 10000000.0,
            "gap_rate_30d": 0.0,
            "mapping_confidence": "strong",
            "status_reason": "approved"
        }],
        "excluded_symbols": [],
        "liquidity_rank_at_that_time": [],
        "selection_policy_version": "symbol_universe_policy_v1",
        "venue_truth_policy_version": "venue_truth_policy_v1",
        "data_quality_cutoff_version": "data_quality_cutoff_v1",
        "generated_at_ms": 1000
    })
}

fn market_feature_delta_json() -> Value {
    json!([{
        "schema_version": "market_feature_delta_v1",
        "feature_delta_id": "delta_cli_price",
        "l1_run_id": "l1_cli_001",
        "metric_name": "price",
        "venue": "binance",
        "symbol_native": "SUIUSDT",
        "symbol_canonical": "SUI",
        "market_type": "spot",
        "value_now": 101.0,
        "value_15m_ago": 100.0,
        "value_1h_ago": 99.0,
        "change_pct_15m": 1.0,
        "change_pct_1h": 2.02,
        "price_change_same_window": 2.02,
        "volume_change_same_window": 10.0,
        "window_start_ms": 1000,
        "window_end_ms": 1200,
        "known_as_of_ms": 1250,
        "quality_status": "complete",
        "missing_reasons": []
    }])
}

fn market_regime_context_json() -> Value {
    json!([{
        "schema_version": "market_regime_context_v1",
        "regime_context_id": "regime_cli_001",
        "l1_run_id": "l1_cli_001",
        "scope": "market_all_symbols",
        "window_start_ms": 1000,
        "window_end_ms": 1200,
        "btc_return_same_window": 0.5,
        "eth_return_same_window": 0.7,
        "sector_return_same_window": 0.4,
        "volatility_regime": "medium",
        "correlation_to_btc": 0.8,
        "known_as_of_ms": 1250,
        "quality_status": "complete",
        "missing_reasons": []
    }])
}

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
