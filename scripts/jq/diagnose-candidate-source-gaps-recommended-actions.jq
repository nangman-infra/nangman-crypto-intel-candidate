def source_gap_recommended_actions($symbol_diagnostics):
  [
    if any($symbol_diagnostics[]?; .status == "no_structured_intel_seen")
      then "increase_structured_intel_source_coverage_for_missing_major50_symbols"
      else empty
    end,
    if any($symbol_diagnostics[]?; .status == "structured_intel_without_screening")
      then "inspect_candidate_worker_input_scan_and_structured_pointer_delivery"
      else empty
    end,
    if any($symbol_diagnostics[]?; .status == "screened_without_research_candidate")
      then "inspect_scoring_rejection_reasons_before_widening_research_dispatch"
      else empty
    end,
    if any($symbol_diagnostics[]?; .status == "candidate_evidence_outside_research_batch_selection")
      then "widen_research_candidate_scan_or_build_focused_manifest_for_existing_evidence"
      else empty
    end,
    if any($symbol_diagnostics[]?; .primary_blocker == "market_context_materialization")
      then "repair_or_rehydrate_market_context_before_forcing_candidate_generation"
      else empty
    end,
    if any($symbol_diagnostics[]?; .primary_blocker == "pending_market_context_materialization")
      then "materialize_pending_market_context_or_refresh_structured_intel_before_research_dispatch"
      else empty
    end,
    if any($symbol_diagnostics[]?; .primary_blocker == "market_context_basis_missing")
      then "refresh_or_rehydrate_structured_intel_with_market_context_basis_before_research_dispatch"
      else empty
    end,
    if any($symbol_diagnostics[]?; .primary_blocker == "market_context_admissibility")
      then "inspect_market_context_admissibility_before_forcing_candidate_generation"
      else empty
    end,
    if any($symbol_diagnostics[]?; .market_context_gap.historical_terminal_missing_context_present // false)
      then "backfill_historical_market_l1_or_mark_stale_public_intel_before_research_dispatch"
      else empty
    end,
    if any($symbol_diagnostics[]?; .primary_blocker == "point_in_time_universe_admission")
      then "inspect_point_in_time_universe_snapshot_before_widening_candidate_policy"
      else empty
    end,
    if any($symbol_diagnostics[]?; .status == "candidate_evidence_present_not_in_research_gap")
      then "reconcile_candidate_evidence_artifacts_with_research_gap_status"
      else empty
    end,
    "keep_research_dispatch_current_approved_gated_until_candidate_source_gaps_are_closed",
    "do_not_open_shadow_paper_live_from_candidate_source_gap"
  ]
  | unique;
