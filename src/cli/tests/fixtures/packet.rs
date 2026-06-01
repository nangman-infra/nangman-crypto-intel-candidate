use serde_json::{Value, json};

pub(in crate::cli::tests) fn packet_json() -> Value {
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
