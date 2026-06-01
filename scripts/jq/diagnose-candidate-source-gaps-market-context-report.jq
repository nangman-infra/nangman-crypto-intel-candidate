import "diagnose-candidate-source-gaps-market-context-global-report" as global_report;
import "diagnose-candidate-source-gaps-market-context-symbol-report" as symbol_report;

def market_context_gap_summary($market_context_matches; $observed_market_context_floor_ms):
  symbol_report::market_context_gap_summary($market_context_matches; $observed_market_context_floor_ms);

def global_market_context_gap_summary($symbol_diagnostics; $observed_market_context_floor_ms):
  global_report::global_market_context_gap_summary($symbol_diagnostics; $observed_market_context_floor_ms);
