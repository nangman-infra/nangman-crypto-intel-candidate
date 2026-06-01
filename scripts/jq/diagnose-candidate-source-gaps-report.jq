include "diagnose-candidate-source-gaps-lib";
import "diagnose-candidate-source-gaps-market-context-report" as market_context_report;
import "diagnose-candidate-source-gaps-recommended-actions" as recommended_actions;

def market_context_gap_summary($market_context_matches; $observed_market_context_floor_ms):
  market_context_report::market_context_gap_summary($market_context_matches; $observed_market_context_floor_ms);

def global_market_context_gap_summary($symbol_diagnostics; $observed_market_context_floor_ms):
  market_context_report::global_market_context_gap_summary($symbol_diagnostics; $observed_market_context_floor_ms);

def source_gap_recommended_actions($symbol_diagnostics):
  recommended_actions::source_gap_recommended_actions($symbol_diagnostics);
