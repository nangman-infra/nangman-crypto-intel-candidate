include "diagnose-candidate-source-gaps-records";

def context_status:
  (.market_context_status? // .market_context_ref.status? // .market_context.status? // "" | tostring);

def context_basis_ms:
  (.market_context_ref.basis_timestamp_ms? // .market_context.basis_timestamp_ms? // null);

def event_basis_ms:
  (
    .published_at_ms?
    // .fetched_at_ms?
    // .event_timestamp_ms?
    // .decision_available_at_ms?
    // context_basis_ms
    // null
  );

def terminal_context_reason:
  (.market_context_terminal_reason? // .market_context.terminal_reason? // "");

def is_pending_market_context:
  context_status == "pending";

def is_unavailable_market_context:
  context_status == "unavailable";

def is_available_market_context:
  (context_status) as $status
  | [
      "available_symbol_context",
      "available_general_context",
      "nearest_available",
      "stale_but_usable",
      "symbol_context_only"
    ]
  | index($status) != null;

def is_terminal_missing_market_context:
  context_status == "unavailable"
  and terminal_context_reason == "terminal_missing_market_context";

def market_context_record($artifact_family):
  {
    artifact_family:$artifact_family,
    packet_id:.__packet_id,
    symbols:.__symbols,
    market_context_status:context_status,
    market_context_terminal_reason:terminal_context_reason,
    published_at_ms:(.published_at_ms? // null),
    fetched_at_ms:(.fetched_at_ms? // null),
    event_basis_ms:event_basis_ms,
    event_basis_at:(event_basis_ms | iso_ms)
  };
