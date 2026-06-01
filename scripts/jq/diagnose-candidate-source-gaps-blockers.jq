def reason_group:
  if . == "missing_market_feature_delta"
    or . == "derivatives_metric_delta_missing"
  then "market_context_materialization"
  elif . == "missing_market_regime_context"
    or . == "missing_data_quality_summary"
  then "market_context_materialization"
  elif . == "market_context_not_research_admissible"
  then "market_context_admissibility"
  elif . == "missing_point_in_time_universe"
    or . == "not_admitted_universe"
  then "point_in_time_universe_admission"
  elif . == "weak_symbol_resolution"
    or . == "missing_symbol_resolution_trace"
  then "symbol_resolution"
  elif . == "missing_evidence"
  then "evidence_quality"
  elif . == "missing_source_independence"
    or . == "insufficient_source_independence"
    or . == "single_source_only"
  then "source_independence"
  else "other"
  end;

def dominant_blocker_group($groups):
  ($groups[0].blocker_group // null);

def primary_blocker($status; $groups; $market_context_gap):
    if $status == "no_structured_intel_seen"
    then "structured_intel_absent"
    elif $status == "structured_intel_without_screening"
    then "candidate_worker_input_gap"
    elif $status == "candidate_evidence_outside_research_batch_selection"
    then "research_batch_scan_window"
    elif $status == "candidate_evidence_present_not_in_research_gap"
    then "research_manifest_reconciliation"
    elif (($market_context_gap.pending_materialization_required // false)
      and any($groups[]?; .blocker_group == "market_context_materialization"))
    then "pending_market_context_materialization"
    elif (($market_context_gap.market_context_basis_missing_required // false)
      and any($groups[]?; .blocker_group == "market_context_materialization"))
    then "market_context_basis_missing"
    elif (($market_context_gap.historical_backfill_required // false)
      and any($groups[]?; .blocker_group == "market_context_materialization"))
    then "historical_market_l1_backfill_required"
  else (dominant_blocker_group($groups)) as $dominant
  | if $dominant == "market_context_materialization"
    then "market_context_materialization"
    elif $dominant == "market_context_admissibility"
    then "market_context_admissibility"
    elif $dominant == "point_in_time_universe_admission"
    then "point_in_time_universe_admission"
    elif $dominant == "symbol_resolution"
    then "symbol_resolution"
    elif $dominant == "evidence_quality"
    then "evidence_quality"
    elif $dominant == "source_independence"
    then "source_independence"
    elif $dominant != null
    then $dominant
    elif $status == "screened_without_research_candidate"
    then "unclassified_screening_gap"
    else "no_candidate_gap_detected"
    end
  end;
