#!/usr/bin/env bash
set -euo pipefail

STATUS_FILE="${INTEL_CANDIDATE_RESEARCH_STATUS_FILE:-${1:-}}"
OUTPUT_FILE="${INTEL_CANDIDATE_COVERAGE_GAP_OUTPUT:-${2:-}}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

require_absolute_file() {
  local name="$1"
  local path="$2"
  if [[ -z "$path" || "$path" != /* ]]; then
    echo "$name must be an absolute file path" >&2
    exit 1
  fi
  if [[ ! -f "$path" ]]; then
    echo "$name does not exist: $path" >&2
    exit 1
  fi
}

require_absolute_output_path() {
  local name="$1"
  local path="$2"
  if [[ -z "$path" ]]; then
    return
  fi
  case "$path" in
    /*) ;;
    *)
      echo "$name must be an absolute path; got $path" >&2
      exit 1
      ;;
  esac
}

require_command date
require_command jq
require_command mktemp
require_absolute_file "INTEL_CANDIDATE_RESEARCH_STATUS_FILE or first argument" "$STATUS_FILE"
require_absolute_output_path "INTEL_CANDIDATE_COVERAGE_GAP_OUTPUT or second argument" "$OUTPUT_FILE"

tmp_output="$(mktemp)"
trap 'rm -f "$tmp_output"' EXIT

jq \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg status_file "$STATUS_FILE" \
  '
    def unique_sorted: unique | sort;
    def canonical_symbol:
      (tostring | ascii_upcase | gsub("[^A-Z0-9]"; "")) as $symbol
      | if (($symbol | length) > 4 and ($symbol | endswith("USDT"))) then $symbol[0:-4]
        elif (($symbol | length) > 6 and ($symbol | endswith("USDC"))) then $symbol[0:-4]
        else $symbol
        end;
    def normalized_symbol_list:
      map(canonical_symbol)
      | map(select(length > 0))
      | unique_sorted;
    def require_supported_input:
      if .schema_version == "research_horizon_status_checkpoint_v1" then .
      elif (
        ((.stage_state? | type) == "object")
        and ((.major50_universe? | type) == "object")
        and ((.recent_candidates? | type) == "object")
      ) then .
      else
        error(
          "unsupported research status schema: "
          + (.schema_version // "unknown")
          + "; expected research_horizon_status_checkpoint_v1 or check-loop-state output"
        )
      end;
    def stage_count($name; $fallback):
      (.research_factory_gap_summary.stage_counts[$name] // $fallback);
    def gap_count($name; $fallback):
      (.research_factory_gap_summary.gap_counts[$name] // $fallback);
    def approved_symbols:
      (
        .major50_state.approved_symbols
        // .major50_universe.approved_symbols
        // []
      )
      | normalized_symbol_list;
    def candidate_symbols:
      (
        .research_factory_progression.symbols.candidate_generated
        // .recent_candidates.distinct_candidate_symbols
        // .selected_symbols
        // []
      )
      | normalized_symbol_list;
    def research_replayed_symbols:
      (
        .research_factory_progression.symbols.research_replayed
        // .recent_research_report_coverage.replayed_symbols
        // .research_evidence.top_symbols
        // .best_current_approved_shard_batch.top_symbols
        // .latest_research_report.top_symbols
        // []
      )
      | normalized_symbol_list;
    def promoted_symbols:
      (
        .research_factory_progression.symbols.promoted
        // []
      )
      | normalized_symbol_list;

    require_supported_input as $status
    | (
        $status.coverage_gaps.approved_symbols_without_candidate
        // $status.coverage_gaps.approved_symbols_without_recent_candidate
        // $status.major50_state.approved_symbols_without_selected_candidate
        // (approved_symbols - candidate_symbols)
        // []
        | normalized_symbol_list
      ) as $approved_without_candidate
    | (
        $status.coverage_gaps.candidate_symbols_without_replay
        // $status.coverage_gaps.recent_candidate_symbols_without_replay
        // (candidate_symbols - research_replayed_symbols)
        // []
        | normalized_symbol_list
      ) as $candidate_without_replay
    | (
        $status.coverage_gaps.replayed_symbols_without_promotion
        // $status.coverage_gaps.replayed_symbols_without_promotion_ready
        // (research_replayed_symbols - promoted_symbols)
        // []
        | normalized_symbol_list
      ) as $replayed_without_promotion
    | (candidate_symbols) as $candidate_symbols
    | (research_replayed_symbols) as $research_replayed_symbols
    | (promoted_symbols) as $promoted_symbols
    | (
        if ($approved_without_candidate | length) > 0 then "candidate_generation_coverage"
        elif ($candidate_without_replay | length) > 0 then "research_replay_coverage"
        elif ($promoted_symbols | length) == 0 then "promotion_evidence"
        elif (($status.stage_state.shadow_created // false) != true) then "shadow_review_gate"
        elif (($status.stage_state.paper_created // false) != true) then "paper_validation_gate"
        else "no_gap_detected"
        end
      ) as $blocking_stage
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
          major50_observed_symbol_count:stage_count("major50_observed"; ($status.major50_state.observed_symbol_count // $status.major50_universe.observed_symbol_count // null)),
          major50_approved_symbol_count:stage_count("major50_approved"; ($status.major50_state.approved_symbol_count // $status.major50_universe.approved_symbol_count // null)),
          candidate_generated_symbol_count:stage_count("candidate_generated"; ($candidate_symbols | length)),
          research_replayed_symbol_count:stage_count("research_replayed"; ($research_replayed_symbols | length)),
          promotion_ready_symbol_count:stage_count("promotion_ready"; 0),
          promoted_symbol_count:stage_count("promoted"; ($promoted_symbols | length)),
          gap_counts:{
            approved_symbols_without_candidate:gap_count("approved_symbols_without_candidate"; ($approved_without_candidate | length)),
            candidate_symbols_without_replay:gap_count("candidate_symbols_without_replay"; ($candidate_without_replay | length)),
            replayed_symbols_without_promotion:gap_count("replayed_symbols_without_promotion"; ($replayed_without_promotion | length))
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
        recommended_actions:(
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
          | unique
        ),
        next_decision:{
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
        }
      }
  ' "$STATUS_FILE" > "$tmp_output"

if [[ -n "$OUTPUT_FILE" ]]; then
  cp "$tmp_output" "$OUTPUT_FILE"
  {
    echo "coverage_gap_output=$OUTPUT_FILE"
    jq -r '
      "blocking_stage=\(.coverage.blocking_stage)",
      "approved_symbols_without_candidate=\(.coverage.gap_counts.approved_symbols_without_candidate)",
      "candidate_symbols_without_replay=\(.coverage.gap_counts.candidate_symbols_without_replay)",
      "replayed_symbols_without_promotion=\(.coverage.gap_counts.replayed_symbols_without_promotion)"
    ' "$tmp_output"
  } >&2
else
  cat "$tmp_output"
fi

echo "candidate coverage gap diagnosis completed" >&2
