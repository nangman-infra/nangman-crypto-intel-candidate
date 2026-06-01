include "market-l1-recovery-plan-lib";

def recovery_context_records:
  [
    ((.market_context_gap.historical_terminal_missing_context_records // [])[]
      | . + {recovery_reason:"terminal_missing_market_context"}),
    ((.market_context_gap.current_or_unknown_terminal_missing_context_records // [])[]
      | . + {recovery_reason:"terminal_missing_market_context"}),
    ((.market_context_gap.historical_pending_context_records // [])[]
      | . + {recovery_reason:"pending_market_context_materialization"}),
    ((.market_context_gap.current_or_unknown_pending_context_records // [])[]
      | . + {recovery_reason:"pending_market_context_materialization"})
  ] | map(select(.event_basis_ms != null));

def recovery_class:
  if (.market_context_gap.pending_context_packets // 0) > 0
  then "pending_market_context_materialization"
  elif .market_context_gap.historical_backfill_required // false
  then "full_historical_backfill_required"
  elif (.market_context_gap.historical_terminal_missing_context_present // false)
    and (.market_context_gap.current_or_unknown_terminal_missing_context_present // false)
  then "mixed_terminal_context_recovery"
  elif .market_context_gap.historical_terminal_missing_context_present // false
  then "historical_terminal_context_recovery"
  else "current_or_unknown_terminal_context_recovery"
  end;

def recovery_window($symbol):
  (.event_basis_ms // null) as $event_ms
  | (($event_ms - $recovery_margin_ms) | clamp_positive | align_floor($normalize_schedule_interval_ms)) as $window_start_ms
  | (($event_ms + $recovery_margin_ms) | clamp_positive | align_ceil($normalize_schedule_interval_ms)) as $window_end_ms
  | {
      packet_id:.packet_id,
      artifact_family:(.artifact_family // null),
      recovery_reason:(.recovery_reason // null),
      market_context_status:(.market_context_status // null),
      symbols:(.symbols // []),
      event_basis_ms:$event_ms,
      event_basis_at:($event_ms | iso_ms),
      recovery_input_start_ms:$window_start_ms,
      recovery_input_start_at:($window_start_ms | iso_ms),
      recovery_input_end_ms:$window_end_ms,
      recovery_input_end_at:($window_end_ms | iso_ms),
      market_backfill_args:market_backfill_args($symbol; $window_start_ms; $window_end_ms),
      market_normalize_args:market_normalize_args($window_start_ms; $window_end_ms),
      market_l1_index_audit_args:market_l1_index_audit_args($window_start_ms; $window_end_ms)
    };

def recovery_symbol:
  recovery_context_records as $context_records
  | select(($context_records | length) > 0)
  | (.symbol) as $symbol
  | ($context_records | map(.event_basis_ms) | min) as $min_ms
  | ($context_records | map(.event_basis_ms) | max) as $max_ms
  | ([$context_records[] | recovery_window($symbol)]) as $recovery_windows
  | ($recovery_windows | map(.recovery_input_start_ms) | min) as $recovery_start_ms
  | ($recovery_windows | map(.recovery_input_end_ms) | max) as $recovery_end_ms
  | {
      symbol:.symbol,
      recovery_class:recovery_class,
      primary_blocker:.primary_blocker,
      status:.status,
      terminal_missing_context_packets:(.market_context_gap.terminal_missing_context_packets // 0),
      terminal_missing_before_observed_context_floor:(.market_context_gap.terminal_missing_before_observed_context_floor // 0),
      terminal_missing_at_or_after_observed_context_floor:(.market_context_gap.terminal_missing_at_or_after_observed_context_floor // 0),
      terminal_missing_unknown_event_basis:(.market_context_gap.terminal_missing_unknown_event_basis // 0),
      pending_context_packets:(.market_context_gap.pending_context_packets // 0),
      pending_before_observed_context_floor:(.market_context_gap.pending_before_observed_context_floor // 0),
      pending_at_or_after_observed_context_floor:(.market_context_gap.pending_at_or_after_observed_context_floor // 0),
      pending_unknown_event_basis:(.market_context_gap.pending_unknown_event_basis // 0),
      planned_terminal_record_count:(
        [$context_records[] | select(.recovery_reason == "terminal_missing_market_context")]
        | length
      ),
      planned_pending_record_count:(
        [$context_records[] | select(.recovery_reason == "pending_market_context_materialization")]
        | length
      ),
      planned_context_record_count:($context_records | length),
      historical_terminal_missing_present:(.market_context_gap.historical_terminal_missing_context_present // false),
      current_or_unknown_terminal_missing_present:(.market_context_gap.current_or_unknown_terminal_missing_context_present // false),
      historical_pending_context_present:(.market_context_gap.historical_pending_context_present // false),
      current_or_unknown_pending_context_present:(.market_context_gap.current_or_unknown_pending_context_present // false),
      terminal_missing_event_basis_min_ms:$min_ms,
      terminal_missing_event_basis_min_at:($min_ms | iso_ms),
      terminal_missing_event_basis_max_ms:$max_ms,
      terminal_missing_event_basis_max_at:($max_ms | iso_ms),
      recovery_input_start_ms:$recovery_start_ms,
      recovery_input_start_at:($recovery_start_ms | iso_ms),
      recovery_input_end_ms:$recovery_end_ms,
      recovery_input_end_at:($recovery_end_ms | iso_ms),
      recovery_margin_ms:$recovery_margin_ms,
      venue:$market_venue,
      market_symbol:market_symbol_for(.symbol),
      symbol_mapping_status:mapping_status_for(.symbol),
      requires_symbol_mapping_review:(mapping_status_for(.symbol) == "derived_quote_suffix_unverified"),
      recovery_window_count:($recovery_windows | length),
      recovery_windows:$recovery_windows,
      post_repair_checks:[
        "rerun_candidate_source_gap_diagnosis_v2",
        "rebuild_current_approved_research_batch_manifest",
        "keep_dispatcher_shadow_paper_live_closed_until_research_gate_passes"
      ],
      sample_terminal_missing_context:(.market_context_gap.sample_terminal_missing_context // []),
      sample_pending_context:(.market_context_gap.sample_pending_context // [])
    };

def recovery_symbols($gap):
  [$gap.symbols[]? | recovery_symbol];
