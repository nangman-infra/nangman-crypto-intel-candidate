include "diagnose-candidate-coverage-gaps-lib";
include "diagnose-candidate-coverage-gaps-symbols";

def approved_symbols_without_candidate($status):
  (
    $status.coverage_gaps.approved_symbols_without_candidate
    // $status.coverage_gaps.approved_symbols_without_recent_candidate
    // $status.major50_state.approved_symbols_without_selected_candidate
    // (approved_symbols($status) - candidate_symbols($status))
    // []
    | normalized_symbol_list
  );

def candidate_symbols_without_replay($status):
  (
    $status.coverage_gaps.candidate_symbols_without_replay
    // $status.coverage_gaps.recent_candidate_symbols_without_replay
    // (candidate_symbols($status) - research_replayed_symbols($status))
    // []
    | normalized_symbol_list
  );

def replayed_symbols_without_promotion($status):
  (
    $status.coverage_gaps.replayed_symbols_without_promotion
    // $status.coverage_gaps.replayed_symbols_without_promotion_ready
    // (research_replayed_symbols($status) - promoted_symbols($status))
    // []
    | normalized_symbol_list
  );

def coverage_blocking_stage($status; $approved_without_candidate; $candidate_without_replay; $promoted_symbols):
  if ($approved_without_candidate | length) > 0 then "candidate_generation_coverage"
  elif ($candidate_without_replay | length) > 0 then "research_replay_coverage"
  elif ($promoted_symbols | length) == 0 then "promotion_evidence"
  elif (($status.stage_state.shadow_created // false) != true) then "shadow_review_gate"
  elif (($status.stage_state.paper_created // false) != true) then "paper_validation_gate"
  else "no_gap_detected"
  end;
