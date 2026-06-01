import "diagnose-candidate-source-gaps-blockers" as blockers;
import "diagnose-candidate-source-gaps-evidence-contract" as evidence_contract;
import "diagnose-candidate-source-gaps-market-context-lib" as market_context;
import "diagnose-candidate-source-gaps-records" as records;
import "diagnose-candidate-source-gaps-revisions" as revisions;

def canonical_symbol: records::canonical_symbol;
def as_symbol_list: records::as_symbol_list;
def record_symbols: records::record_symbols;
def packet_id: records::packet_id;
def supersedes_packet_id: records::supersedes_packet_id;
def reasons: records::reasons;
def candidate_class: records::candidate_class;
def iso_ms: records::iso_ms;
def histogram($values; $key_name): records::histogram($values; $key_name);

def context_status: market_context::context_status;
def context_basis_ms: market_context::context_basis_ms;
def event_basis_ms: market_context::event_basis_ms;
def terminal_context_reason: market_context::terminal_context_reason;
def is_pending_market_context: market_context::is_pending_market_context;
def is_unavailable_market_context: market_context::is_unavailable_market_context;
def is_available_market_context: market_context::is_available_market_context;
def is_terminal_missing_market_context: market_context::is_terminal_missing_market_context;
def market_context_record($artifact_family): market_context::market_context_record($artifact_family);

def horizon_ms($h): evidence_contract::horizon_ms($h);
def evidence_horizon_contract_valid: evidence_contract::evidence_horizon_contract_valid;
def evidence_ref_values($matches): evidence_contract::evidence_ref_values($matches);
def evidence_contract_summary($matches): evidence_contract::evidence_contract_summary($matches);

def reason_group: blockers::reason_group;
def dominant_blocker_group($groups): blockers::dominant_blocker_group($groups);
def primary_blocker($status; $groups; $market_context_gap):
  blockers::primary_blocker($status; $groups; $market_context_gap);

def symbolized($records; $structured_records): revisions::symbolized($records; $structured_records);
def current_records($records; $superseded_packet_ids): revisions::current_records($records; $superseded_packet_ids);
def superseded_records($records; $superseded_packet_ids): revisions::superseded_records($records; $superseded_packet_ids);
def records_for_symbol($records; $symbol): revisions::records_for_symbol($records; $symbol);
