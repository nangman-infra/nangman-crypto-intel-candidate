include "diagnose-candidate-source-gaps-lib";
import "diagnose-candidate-source-gaps-market-context-symbol-buckets" as buckets;

def market_context_gap_summary($market_context_matches; $observed_market_context_floor_ms):
  buckets::market_context_gap_buckets($market_context_matches; $observed_market_context_floor_ms) as $gap_buckets
  | $gap_buckets.terminal_missing_context_matches as $terminal_missing_context_matches
  | $gap_buckets.pending_context_matches as $pending_context_matches
  | $gap_buckets.unavailable_context_matches as $unavailable_context_matches
  | $gap_buckets.historical_terminal_missing_context_matches as $historical_terminal_missing_context_matches
  | $gap_buckets.current_or_unknown_terminal_missing_context_matches as $current_or_unknown_terminal_missing_context_matches
  | $gap_buckets.historical_pending_context_matches as $historical_pending_context_matches
  | $gap_buckets.current_or_unknown_pending_context_matches as $current_or_unknown_pending_context_matches
  | $gap_buckets.available_context_packet_ids as $available_context_packet_ids
  | {
      observed_context_floor_ms:$observed_market_context_floor_ms,
      observed_context_floor_at:($observed_market_context_floor_ms | iso_ms),
      terminal_missing_context_packets:($terminal_missing_context_matches | length),
      terminal_missing_before_observed_context_floor:($historical_terminal_missing_context_matches | length),
      terminal_missing_at_or_after_observed_context_floor:($current_or_unknown_terminal_missing_context_matches | length),
      terminal_missing_unknown_event_basis:(
        [
          $terminal_missing_context_matches[]
          | select(.event_basis_ms == null)
        ]
        | length
      ),
      pending_context_packets:($pending_context_matches | length),
      pending_before_observed_context_floor:($historical_pending_context_matches | length),
      pending_at_or_after_observed_context_floor:($current_or_unknown_pending_context_matches | length),
      pending_unknown_event_basis:(
        [
          $pending_context_matches[]
          | select(.event_basis_ms == null)
        ]
        | length
      ),
      unavailable_context_packets:($unavailable_context_matches | length),
      unavailable_unknown_event_basis:(
        [
          $unavailable_context_matches[]
          | select(.event_basis_ms == null)
        ]
        | length
      ),
      available_context_packets:($available_context_packet_ids | length),
      historical_terminal_missing_context_present:(($historical_terminal_missing_context_matches | length) > 0),
      current_or_unknown_terminal_missing_context_present:(($current_or_unknown_terminal_missing_context_matches | length) > 0),
      historical_pending_context_present:(($historical_pending_context_matches | length) > 0),
      current_or_unknown_pending_context_present:(($current_or_unknown_pending_context_matches | length) > 0),
      historical_terminal_missing_event_basis_min_ms:(
        [
          $historical_terminal_missing_context_matches[]
          | .event_basis_ms
          | select(. != null)
        ]
        | min
      ),
      historical_terminal_missing_event_basis_min_at:(
        [
          $historical_terminal_missing_context_matches[]
          | .event_basis_ms
          | select(. != null)
        ]
        | min
        | iso_ms
      ),
      historical_terminal_missing_event_basis_max_ms:(
        [
          $historical_terminal_missing_context_matches[]
          | .event_basis_ms
          | select(. != null)
        ]
        | max
      ),
      historical_terminal_missing_event_basis_max_at:(
        [
          $historical_terminal_missing_context_matches[]
          | .event_basis_ms
          | select(. != null)
        ]
        | max
        | iso_ms
      ),
      historical_backfill_required:(
        ($terminal_missing_context_matches | length) > 0
        and $observed_market_context_floor_ms != null
        and ($historical_terminal_missing_context_matches | length) == ($terminal_missing_context_matches | length)
      ),
      pending_materialization_required:(($pending_context_matches | length) > 0),
      market_context_basis_missing_required:(
        ($unavailable_context_matches | length) > 0
        and ([
          $unavailable_context_matches[]
          | select(.event_basis_ms == null)
        ] | length) == ($unavailable_context_matches | length)
      ),
      sample_terminal_missing_context:(
        $terminal_missing_context_matches
        | sort_by(.event_basis_ms // 0)
        | .[0:10]
      ),
      sample_pending_context:(
        $pending_context_matches
        | sort_by(.event_basis_ms // 0)
        | .[0:10]
      ),
      sample_unavailable_context:(
        $unavailable_context_matches
        | sort_by(.event_basis_ms // 0)
        | .[0:10]
      ),
      historical_terminal_missing_context_records:(
        $historical_terminal_missing_context_matches
        | sort_by(.event_basis_ms // 0)
      ),
      current_or_unknown_terminal_missing_context_records:(
        $current_or_unknown_terminal_missing_context_matches
        | sort_by(.event_basis_ms // 0)
      ),
      historical_pending_context_records:(
        $historical_pending_context_matches
        | sort_by(.event_basis_ms // 0)
      ),
      current_or_unknown_pending_context_records:(
        $current_or_unknown_pending_context_matches
        | sort_by(.event_basis_ms // 0)
      )
    };
