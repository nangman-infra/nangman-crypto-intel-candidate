include "diagnose-candidate-source-gaps-lib";

def market_context_gap($symbol_diagnostic):
  ($symbol_diagnostic.market_context_gap // {});

def market_context_gap_sum($symbol_diagnostics; $field):
  [
    $symbol_diagnostics[]
    | market_context_gap(.)
    | .[$field] // 0
  ]
  | add // 0;

def global_market_context_gap_summary($symbol_diagnostics; $observed_market_context_floor_ms):
  {
    observed_context_floor_ms:$observed_market_context_floor_ms,
    observed_context_floor_at:($observed_market_context_floor_ms | iso_ms),
    symbols_with_terminal_missing_context:(
      [
        $symbol_diagnostics[]
        | select((market_context_gap(.).terminal_missing_context_packets // 0) > 0)
      ]
      | length
    ),
    symbols_with_historical_terminal_missing_context:(
      [
        $symbol_diagnostics[]
        | select((market_context_gap(.).historical_terminal_missing_context_present // false) == true)
      ]
      | length
    ),
    symbols_with_current_or_unknown_terminal_missing_context:(
      [
        $symbol_diagnostics[]
        | select((market_context_gap(.).current_or_unknown_terminal_missing_context_present // false) == true)
      ]
      | length
    ),
    symbols_with_pending_context:(
      [
        $symbol_diagnostics[]
        | select((market_context_gap(.).pending_context_packets // 0) > 0)
      ]
      | length
    ),
    symbols_with_historical_pending_context:(
      [
        $symbol_diagnostics[]
        | select((market_context_gap(.).historical_pending_context_present // false) == true)
      ]
      | length
    ),
    symbols_with_current_or_unknown_pending_context:(
      [
        $symbol_diagnostics[]
        | select((market_context_gap(.).current_or_unknown_pending_context_present // false) == true)
      ]
      | length
    ),
    symbols_with_market_context_basis_missing:(
      [
        $symbol_diagnostics[]
        | select((market_context_gap(.).market_context_basis_missing_required // false) == true)
      ]
      | length
    ),
    symbols_requiring_full_historical_backfill:(
      [
        $symbol_diagnostics[]
        | select((market_context_gap(.).historical_backfill_required // false) == true)
      ]
      | length
    ),
    terminal_missing_context_packets:market_context_gap_sum($symbol_diagnostics; "terminal_missing_context_packets"),
    terminal_missing_before_observed_context_floor:market_context_gap_sum($symbol_diagnostics; "terminal_missing_before_observed_context_floor"),
    terminal_missing_at_or_after_observed_context_floor:market_context_gap_sum($symbol_diagnostics; "terminal_missing_at_or_after_observed_context_floor"),
    pending_context_packets:market_context_gap_sum($symbol_diagnostics; "pending_context_packets"),
    pending_before_observed_context_floor:market_context_gap_sum($symbol_diagnostics; "pending_before_observed_context_floor"),
    pending_at_or_after_observed_context_floor:market_context_gap_sum($symbol_diagnostics; "pending_at_or_after_observed_context_floor"),
    unavailable_context_packets:market_context_gap_sum($symbol_diagnostics; "unavailable_context_packets"),
    unavailable_unknown_event_basis:market_context_gap_sum($symbol_diagnostics; "unavailable_unknown_event_basis"),
    available_context_packets:market_context_gap_sum($symbol_diagnostics; "available_context_packets")
  };
