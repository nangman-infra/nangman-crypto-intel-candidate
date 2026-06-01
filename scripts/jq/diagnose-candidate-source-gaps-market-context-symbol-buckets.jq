include "diagnose-candidate-source-gaps-lib";

def before_observed_context_floor($observed_market_context_floor_ms):
  select(
    $observed_market_context_floor_ms != null
    and .event_basis_ms != null
    and .event_basis_ms < $observed_market_context_floor_ms
  );

def current_or_unknown_context_floor($observed_market_context_floor_ms):
  select(
    ($observed_market_context_floor_ms == null)
    or (.event_basis_ms == null)
    or (.event_basis_ms >= $observed_market_context_floor_ms)
  );

def market_context_gap_buckets($market_context_matches; $observed_market_context_floor_ms):
  ([
    $market_context_matches[]
    | select(is_terminal_missing_market_context)
  ]) as $terminal_missing_context_matches
  | ([
    $market_context_matches[]
    | select(is_pending_market_context)
  ]) as $pending_context_matches
  | ([
    $market_context_matches[]
    | select(is_unavailable_market_context)
  ]) as $unavailable_context_matches
  | ([
    $terminal_missing_context_matches[]
    | before_observed_context_floor($observed_market_context_floor_ms)
  ]) as $historical_terminal_missing_context_matches
  | ([
    $terminal_missing_context_matches[]
    | current_or_unknown_context_floor($observed_market_context_floor_ms)
  ]) as $current_or_unknown_terminal_missing_context_matches
  | ([
    $pending_context_matches[]
    | before_observed_context_floor($observed_market_context_floor_ms)
  ]) as $historical_pending_context_matches
  | ([
    $pending_context_matches[]
    | current_or_unknown_context_floor($observed_market_context_floor_ms)
  ]) as $current_or_unknown_pending_context_matches
  | ([
    $market_context_matches[]
    | select(is_available_market_context)
    | .packet_id
  ] | unique | sort) as $available_context_packet_ids
  | {
      terminal_missing_context_matches:$terminal_missing_context_matches,
      pending_context_matches:$pending_context_matches,
      unavailable_context_matches:$unavailable_context_matches,
      historical_terminal_missing_context_matches:$historical_terminal_missing_context_matches,
      current_or_unknown_terminal_missing_context_matches:$current_or_unknown_terminal_missing_context_matches,
      historical_pending_context_matches:$historical_pending_context_matches,
      current_or_unknown_pending_context_matches:$current_or_unknown_pending_context_matches,
      available_context_packet_ids:$available_context_packet_ids
    };
