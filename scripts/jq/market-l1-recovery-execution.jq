include "market-l1-recovery-execution-lib";
include "market-l1-recovery-execution-static";

($plan[0]) as $plan
| ($readiness[0]) as $readiness
| ($plan.input.market_ingest_app_root) as $root
| ($plan.input.market_venue) as $venue
| ($plan.input.market_window_ms) as $window_ms
| ($plan.input.normalize_schedule_interval_ms) as $schedule_interval_ms
| [
    $plan.symbols[]?
    | .symbol as $symbol
    | .market_symbol as $market_symbol
    | .recovery_windows[]?
    | {
        symbol:$symbol,
        market_symbol:$market_symbol,
        start_ms:.recovery_input_start_ms,
        end_ms:.recovery_input_end_ms,
        packet_ids:[.packet_id],
        symbols:[$symbol],
        market_symbols:[$market_symbol],
        source_window_count:1
      }
  ] as $flat_windows
| ($flat_windows
    | group_by(.market_symbol)
    | map({
        symbol:.[0].symbol,
        market_symbol:.[0].market_symbol,
        ranges:(map({
          start_ms,
          end_ms,
          packet_ids,
          symbols,
          market_symbols,
          source_window_count
        }) | merge_ranges)
      })
  ) as $by_market_symbol
| ($by_market_symbol
    | map(
        . as $group
        | $group.ranges[]
        | {
            symbol:$group.symbol,
            market_symbol:$group.market_symbol,
            start_ms,
            start_at:(.start_ms | iso_ms),
            end_ms,
            end_at:(.end_ms | iso_ms),
            source_window_count,
            packet_ids,
            command_args:backfill_args($root; $venue; $group.market_symbol; .start_ms; .end_ms),
            writes_s3:true,
            target:"market_l0"
          }
      )
  ) as $backfill_steps
| ($flat_windows
    | map({
        start_ms,
        end_ms,
        packet_ids,
        symbols,
        market_symbols,
        source_window_count
      })
    | merge_ranges
    | map({
        start_ms,
        start_at:(.start_ms | iso_ms),
        end_ms,
        end_at:(.end_ms | iso_ms),
        source_window_count,
        packet_ids,
        symbols,
        market_symbols,
        command_args:normalize_args($root; .start_ms; .end_ms; $window_ms; $schedule_interval_ms),
        writes_s3:true,
        target:"market_l1"
      })
  ) as $normalize_steps
| ($normalize_steps
    | map({
        start_ms,
        start_at,
        end_ms,
        end_at,
        source_window_count,
        packet_ids,
        symbols,
        market_symbols,
        command_args:audit_args($root; .start_ms; .end_ms; $window_ms),
        reads_s3:true,
        writes_s3:false,
        target:"market_l1_index"
      })
  ) as $post_audit_steps
| {
    schema_version:"candidate_market_l1_recovery_execution_manifest_v1",
    generated_at:$generated_at,
    input:{
      recovery_plan_file:$recovery_plan_file,
      readiness_file:(if ($readiness_file | length) == 0 then null else $readiness_file end),
      market_ingest_app_root:$root,
      market_venue:$venue,
      market_window_ms:$window_ms,
      normalize_schedule_interval_ms:$schedule_interval_ms
    },
    safety:recovery_execution_safety,
    readiness_gate:readiness_gate($readiness),
    required_environment:required_recovery_environment,
    approval_boundary:recovery_approval_boundary,
    summary:{
      source_recovery_window_count:($flat_windows | length),
      backfill_step_count:($backfill_steps | length),
      normalize_step_count:($normalize_steps | length),
      post_audit_step_count:($post_audit_steps | length),
      context_recovery_symbol_count:($plan.summary.context_recovery_symbol_count // null),
      terminal_missing_symbol_count:($plan.summary.terminal_missing_symbol_count // null),
      pending_context_symbol_count:($plan.summary.pending_context_symbol_count // null),
      historical_symbol_count:($plan.summary.historical_symbol_count // null),
      historical_pending_context_symbol_count:($plan.summary.historical_pending_context_symbol_count // null),
      current_or_unknown_terminal_missing_symbol_count:($plan.summary.current_or_unknown_terminal_missing_symbol_count // null),
      current_or_unknown_pending_context_symbol_count:($plan.summary.current_or_unknown_pending_context_symbol_count // null),
      market_symbol_count:($by_market_symbol | length),
      market_symbols:($by_market_symbol | map(.market_symbol)),
      recovery_input_start_ms:($plan.summary.recovery_input_start_ms // null),
      recovery_input_start_at:($plan.summary.recovery_input_start_at // null),
      recovery_input_end_ms:($plan.summary.recovery_input_end_ms // null),
      recovery_input_end_at:($plan.summary.recovery_input_end_at // null)
    },
    execution_order:recovery_execution_order,
    backfill_steps:$backfill_steps,
    normalize_steps:$normalize_steps,
    post_audit_steps:$post_audit_steps,
    post_repair_checks:recovery_post_repair_checks
  }
