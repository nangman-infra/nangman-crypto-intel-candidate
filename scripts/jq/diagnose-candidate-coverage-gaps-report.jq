include "diagnose-candidate-coverage-gaps-decision";
include "diagnose-candidate-coverage-gaps-gaps";
include "diagnose-candidate-coverage-gaps-lib";
include "diagnose-candidate-coverage-gaps-symbols";

def coverage_gap_diagnosis($status; $generated_at; $status_file):
  (approved_symbols_without_candidate($status)) as $approved_without_candidate
  | (candidate_symbols_without_replay($status)) as $candidate_without_replay
  | (replayed_symbols_without_promotion($status)) as $replayed_without_promotion
  | (candidate_symbols($status)) as $candidate_symbols
  | (research_replayed_symbols($status)) as $research_replayed_symbols
  | (promoted_symbols($status)) as $promoted_symbols
  | (coverage_blocking_stage($status; $approved_without_candidate; $candidate_without_replay; $promoted_symbols)) as $blocking_stage
  | {
      schema_version:"intel_candidate_coverage_gap_diagnosis_v1",
      generated_at:$generated_at,
      input:{
        research_horizon_status_file:$status_file,
        research_horizon_status_schema:($status.schema_version // null),
        research_verdict:($status.verdict // $status.next_decision.verdict // null)
      },
      safety:{
        s3_write:false,
        ecs_task_started:false,
        dispatcher_mode_changed:false,
        local_diagnosis_only:true,
        shadow_paper_live_enabled:false
      },
      stage_state:{
        candidate_generated:($status.stage_state.candidate_generated // false),
        research_replay_completed:($status.stage_state.research_replay_completed // false),
        promotion_passed:($status.stage_state.promotion_passed // false),
        shadow_created:($status.stage_state.shadow_created // false),
        paper_created:($status.stage_state.paper_created // false),
        live_enabled:false
      },
      coverage:{
        blocking_stage:$blocking_stage,
        major50_observed_symbol_count:stage_count($status; "major50_observed"; ($status.major50_state.observed_symbol_count // $status.major50_universe.observed_symbol_count // null)),
        major50_approved_symbol_count:stage_count($status; "major50_approved"; ($status.major50_state.approved_symbol_count // $status.major50_universe.approved_symbol_count // null)),
        candidate_generated_symbol_count:stage_count($status; "candidate_generated"; ($candidate_symbols | length)),
        research_replayed_symbol_count:stage_count($status; "research_replayed"; ($research_replayed_symbols | length)),
        promotion_ready_symbol_count:stage_count($status; "promotion_ready"; 0),
        promoted_symbol_count:stage_count($status; "promoted"; ($promoted_symbols | length)),
        gap_counts:{
          approved_symbols_without_candidate:gap_count($status; "approved_symbols_without_candidate"; ($approved_without_candidate | length)),
          candidate_symbols_without_replay:gap_count($status; "candidate_symbols_without_replay"; ($candidate_without_replay | length)),
          replayed_symbols_without_promotion:gap_count($status; "replayed_symbols_without_promotion"; ($replayed_without_promotion | length))
        }
      },
      gaps:{
        approved_symbols_without_candidate:$approved_without_candidate,
        candidate_symbols_without_replay:$candidate_without_replay,
        replayed_symbols_without_promotion:$replayed_without_promotion
      },
      symbols:{
        candidate_generated:$candidate_symbols,
        research_replayed:$research_replayed_symbols,
        promoted:$promoted_symbols
      },
      recommended_actions:coverage_gap_recommended_actions($approved_without_candidate; $candidate_without_replay; $replayed_without_promotion),
      next_decision:coverage_gap_next_decision(
        $blocking_stage;
        $approved_without_candidate;
        $candidate_without_replay;
        $replayed_without_promotion;
        $candidate_symbols;
        $research_replayed_symbols;
        $promoted_symbols
      )
    };
