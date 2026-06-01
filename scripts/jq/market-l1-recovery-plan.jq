include "market-l1-recovery-plan-lib";
include "market-l1-recovery-plan-symbols";

($source[0]) as $gap
| recovery_symbols($gap) as $symbols
| ($symbols | map(.recovery_input_start_ms) | min) as $recovery_start_ms
| ($symbols | map(.recovery_input_end_ms) | max) as $recovery_end_ms
| {
    schema_version:"candidate_market_l1_recovery_plan_v1",
    generated_at:$generated_at,
    input:{
      source_gap_file:$source_gap_file,
      source_gap_schema:($gap.schema_version // null),
      observed_context_floor_ms:($gap.summary.global_market_context_gap.observed_context_floor_ms // null),
      observed_context_floor_at:($gap.summary.global_market_context_gap.observed_context_floor_at // null),
      market_ingest_app_root:$market_ingest_app_root,
      market_venue:$market_venue,
      market_quote_suffix:$market_quote_suffix,
      market_symbol_map_file:(
        if ($market_symbol_map_file | length) == 0 then null else $market_symbol_map_file end
      ),
      symbol_map_source:$symbol_map_source,
      recovery_margin_ms:$recovery_margin_ms,
      market_window_ms:$market_window_ms,
      normalize_schedule_interval_ms:$normalize_schedule_interval_ms
    },
    safety:{
      local_planning_only:true,
      s3_read:false,
      s3_write:false,
      ecs_task_started:false,
      dispatcher_mode_changed:false,
      research_run_task_started:false,
      shadow_paper_live_enabled:false,
      execution_ready:false,
      execution_requires_explicit_approval:true
    },
    summary:{
      approved_symbols_without_candidate:($gap.summary.approved_symbols_without_candidate // null),
      source_gap_primary_blocker_counts:($gap.summary.primary_blocker_counts // []),
      global_market_context_gap:($gap.summary.global_market_context_gap // {}),
      context_recovery_symbol_count:($symbols | length),
      terminal_missing_symbol_count:([$symbols[] | select((.terminal_missing_context_packets // 0) > 0)] | length),
      pending_context_symbol_count:([$symbols[] | select((.pending_context_packets // 0) > 0)] | length),
      historical_symbol_count:([$symbols[] | select(.historical_terminal_missing_present)] | length),
      historical_pending_context_symbol_count:([$symbols[] | select(.historical_pending_context_present)] | length),
      current_or_unknown_terminal_missing_symbol_count:([$symbols[] | select(.current_or_unknown_terminal_missing_present)] | length),
      current_or_unknown_pending_context_symbol_count:([$symbols[] | select(.current_or_unknown_pending_context_present)] | length),
      full_historical_backfill_symbol_count:([$symbols[] | select(.recovery_class == "full_historical_backfill_required")] | length),
      mixed_terminal_context_symbol_count:([$symbols[] | select(.recovery_class == "mixed_terminal_context_recovery")] | length),
      historical_terminal_context_symbol_count:([$symbols[] | select(.recovery_class == "historical_terminal_context_recovery")] | length),
      current_or_unknown_terminal_context_symbol_count:([$symbols[] | select(.recovery_class == "current_or_unknown_terminal_context_recovery")] | length),
      recovery_window_count:(reduce $symbols[] as $symbol (0; . + ($symbol.recovery_window_count // 0))),
      symbols_requiring_symbol_mapping_review:([$symbols[] | select(.requires_symbol_mapping_review)] | length),
      recovery_input_start_ms:$recovery_start_ms,
      recovery_input_start_at:($recovery_start_ms | iso_ms),
      recovery_input_end_ms:$recovery_end_ms,
      recovery_input_end_at:($recovery_end_ms | iso_ms)
    },
    execution_preconditions:[
      "review market_symbol values against /Volumes/WD/Developments/nangman-crypto/apps/market-ingest-app/config/universe.major-50.toml",
      "replace placeholder buckets and region in generated argument arrays",
      "run market-backfill and market-normalize only after explicit operator approval",
      "do not enable dispatcher run_task, shadow, paper, or live from this plan"
    ],
    market_ingest_contract:{
      historical_l0_worker:"market-backfill",
      historical_l1_worker:"market-normalize",
      l0_worker_writes_s3:true,
      l1_worker_writes_s3:true,
      plan_executes_workers:false,
      read_only_audit_worker:"market-normalize --audit-l1-index-*",
      expected_artifacts_after_approved_execution:[
        "raw_market_event L0",
        "normalized_market_slice_v1 L1",
        "l1_index success pointer",
        "normalization report",
        "symbol_universe_snapshot where applicable"
      ]
    },
    symbols:$symbols,
    recommended_next_actions:[
      "review_symbol_mapping_before_backfill",
      "choose_market_l1_recovery_or_stale_public_intel_marking",
      "rerun_source_gap_diagnosis_after_market_l1_repair",
      "keep_research_dispatcher_dry_run_until_candidate_source_gap_closes"
    ]
  }
