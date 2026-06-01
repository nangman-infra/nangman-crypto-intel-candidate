#!/usr/bin/env bash

write_market_l1_recovery_plan() {
  jq -n \
    -L "$SCRIPT_DIR/jq" \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg source_gap_file "$SOURCE_GAP_FILE" \
    --arg market_ingest_app_root "$MARKET_INGEST_APP_ROOT" \
    --arg market_venue "$MARKET_VENUE" \
    --arg market_quote_suffix "$MARKET_QUOTE_SUFFIX" \
    --arg market_symbol_map_file "$MARKET_SYMBOL_MAP_FILE" \
    --arg symbol_map_source "$symbol_map_source" \
    --argjson recovery_margin_ms "$RECOVERY_MARGIN_MS" \
    --argjson market_window_ms "$MARKET_WINDOW_MS" \
    --argjson normalize_schedule_interval_ms "$NORMALIZE_SCHEDULE_INTERVAL_MS" \
    --slurpfile source "$SOURCE_GAP_FILE" \
    --slurpfile symbol_map "$symbol_map_json" \
    -f "$SCRIPT_DIR/jq/market-l1-recovery-plan.jq" \
    > "$tmp_output"
}

emit_market_l1_recovery_plan_result() {
  if [[ -n "$OUTPUT_FILE" ]]; then
    cp "$tmp_output" "$OUTPUT_FILE"
    {
      echo "market_l1_recovery_plan_output=$OUTPUT_FILE"
      jq -r '
        "terminal_missing_symbol_count=\(.summary.terminal_missing_symbol_count)",
        "pending_context_symbol_count=\(.summary.pending_context_symbol_count)",
        "historical_symbol_count=\(.summary.historical_symbol_count)",
        "historical_pending_context_symbol_count=\(.summary.historical_pending_context_symbol_count)",
        "current_or_unknown_terminal_missing_symbol_count=\(.summary.current_or_unknown_terminal_missing_symbol_count)",
        "current_or_unknown_pending_context_symbol_count=\(.summary.current_or_unknown_pending_context_symbol_count)",
        "full_historical_backfill_symbol_count=\(.summary.full_historical_backfill_symbol_count)",
        "mixed_terminal_context_symbol_count=\(.summary.mixed_terminal_context_symbol_count)",
        "historical_terminal_context_symbol_count=\(.summary.historical_terminal_context_symbol_count)",
        "current_or_unknown_terminal_context_symbol_count=\(.summary.current_or_unknown_terminal_context_symbol_count)",
        "recovery_window_count=\(.summary.recovery_window_count)",
        "symbols_requiring_symbol_mapping_review=\(.summary.symbols_requiring_symbol_mapping_review)"
      ' "$OUTPUT_FILE"
    } >&2
  else
    cat "$tmp_output"
  fi
}
