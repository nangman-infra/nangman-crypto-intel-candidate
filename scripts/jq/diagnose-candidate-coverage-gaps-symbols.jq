include "diagnose-candidate-coverage-gaps-lib";

def approved_symbols($status):
  (
    $status.major50_state.approved_symbols
    // $status.major50_universe.approved_symbols
    // []
  )
  | normalized_symbol_list;

def candidate_symbols($status):
  (
    $status.research_factory_progression.symbols.candidate_generated
    // $status.recent_candidates.distinct_candidate_symbols
    // $status.selected_symbols
    // []
  )
  | normalized_symbol_list;

def research_replayed_symbols($status):
  (
    $status.research_factory_progression.symbols.research_replayed
    // $status.recent_research_report_coverage.replayed_symbols
    // $status.research_evidence.top_symbols
    // $status.best_current_approved_shard_batch.top_symbols
    // $status.latest_research_report.top_symbols
    // []
  )
  | normalized_symbol_list;

def promoted_symbols($status):
  (
    $status.research_factory_progression.symbols.promoted
    // []
  )
  | normalized_symbol_list;
