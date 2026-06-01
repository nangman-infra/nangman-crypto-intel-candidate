def coverage_gap_recommended_actions($approved_without_candidate; $candidate_without_replay; $replayed_without_promotion):
  [
    if ($approved_without_candidate | length) > 0
      then "increase_candidate_generation_for_approved_major50_symbols"
      else empty end,
    if ($approved_without_candidate | length) > 0
      then "inspect_structured_intel_source_coverage_for_missing_symbols"
      else empty end,
    if ($approved_without_candidate | length) > 0
      then "inspect_scoring_rejections_for_missing_symbols"
      else empty end,
    if ($candidate_without_replay | length) > 0
      then "send_unreplayed_candidate_symbols_to_research_manifest"
      else empty end,
    if ($replayed_without_promotion | length) > 0
      then "keep_accumulating_replay_evidence_before_shadow"
      else empty end,
    "do_not_open_shadow_paper_live_from_candidate_coverage_gap"
  ]
  | unique;

def coverage_gap_next_decision(
  $blocking_stage;
  $approved_without_candidate;
  $candidate_without_replay;
  $replayed_without_promotion;
  $candidate_symbols;
  $research_replayed_symbols;
  $promoted_symbols
):
  {
    schema_version:"intel_candidate_coverage_gap_decision_v1",
    verdict:(
      if $blocking_stage == "candidate_generation_coverage" then "INCREASE_CANDIDATE_GENERATION_COVERAGE"
      elif $blocking_stage == "research_replay_coverage" then "SEND_CANDIDATES_TO_RESEARCH_REPLAY"
      elif $blocking_stage == "promotion_evidence" then "ACCUMULATE_RESEARCH_REPLAY_EVIDENCE"
      elif $blocking_stage == "shadow_review_gate" then "WAIT_FOR_SHADOW_REVIEW_GATE"
      elif $blocking_stage == "paper_validation_gate" then "WAIT_FOR_PAPER_VALIDATION_GATE"
      else "NO_COVERAGE_GAP_DETECTED"
      end
    ),
    safe_next_actions:(
      [
        if ($approved_without_candidate | length) > 0
          then "inspect_structured_intel_source_coverage_for_missing_symbols"
          else empty end,
        if ($approved_without_candidate | length) > 0
          then "inspect_scoring_rejections_for_missing_symbols"
          else empty end,
        if ($candidate_without_replay | length) > 0
          then "send_unreplayed_candidate_symbols_to_research_manifest"
          else empty end,
        if ($replayed_without_promotion | length) > 0
          then "keep_accumulating_replay_evidence_before_shadow"
          else empty end
      ]
      | unique
    ),
    blocked_actions:[
      "do_not_create_shadow_without_promotion",
      "do_not_create_paper_without_completed_passed_shadow",
      "do_not_enable_live_from_candidate_coverage_gap"
    ],
    safety:{
      s3_write:false,
      ecs_task_started:false,
      dispatcher_mode_changed:false,
      local_diagnosis_only:true,
      shadow_paper_live_enabled:false,
      live_enabled:false,
      order_execution_enabled:false
    },
    evidence:{
      blocking_stage:$blocking_stage,
      approved_symbols_without_candidate_count:($approved_without_candidate | length),
      candidate_symbols_without_replay_count:($candidate_without_replay | length),
      replayed_symbols_without_promotion_count:($replayed_without_promotion | length),
      candidate_generated_symbol_count:($candidate_symbols | length),
      research_replayed_symbol_count:($research_replayed_symbols | length),
      promoted_symbol_count:($promoted_symbols | length)
    }
  };
