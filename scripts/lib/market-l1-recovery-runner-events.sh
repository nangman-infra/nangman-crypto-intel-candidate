# shellcheck shell=bash

emit_event() {
  local event="$1"
  local payload="$2"
  jq -nc \
    --arg event "$event" \
    --arg run_id "${RUN_ID:-}" \
    --argjson timestamp_ms "$(date -u +%s000)" \
    --argjson payload "$payload" \
    '{
      schema_version:"candidate_market_l1_recovery_run_event_v1",
      event:$event,
      timestamp_ms:$timestamp_ms
    }
    + (if ($run_id | length) == 0 then {} else {run_id:$run_id} end)
    + $payload' \
    >> "$EVENTS_FILE"
}

completed_successfully() {
  local step_key="$1"
  local step_file="$2"
  [[ -f "$EVENTS_FILE" ]] || return 1
  jq -e --arg step_key "$step_key" --slurpfile step "$step_file" '
    def step_identity($step):
      {
        target:($step.target // null),
        start_ms:($step.start_ms // null),
        end_ms:($step.end_ms // null),
        market_symbol:($step.market_symbol // null),
        market_symbols:($step.market_symbols // []),
        command_args:($step.command_args // []),
        writes_s3:($step.writes_s3 // false),
        reads_s3:($step.reads_s3 // false)
      };
    step_identity($step[0]) as $identity
    | select(
        .event == "step_completed"
        and .step_key == $step_key
        and .exit_status == 0
        and .step_identity == $identity
      )
  ' "$EVENTS_FILE" >/dev/null 2>&1
}

write_summary() {
  local status="$1"
  jq -s \
    --arg status "$status" \
    --arg run_id "${RUN_ID:-}" \
    --arg manifest_file "$MANIFEST_FILE" \
    --arg output_dir "$OUTPUT_DIR" \
    '
      map(select(.run_id == $run_id)) as $run_events
      | def count_event($name):
          $run_events | map(select(.event == $name)) | length;
      def completed:
        $run_events | map(select(.event == "step_completed"));
      def completed_success:
        completed | map(select(.exit_status == 0));
      def completed_failed:
        completed | map(select(.exit_status != 0));
      {
        schema_version:"candidate_market_l1_recovery_run_summary_v1",
        run_id:$run_id,
        status:$status,
        created_at_ms:((now|floor) * 1000),
        manifest_file:$manifest_file,
        output_dir:$output_dir,
        safety:{
          bucket_values_redacted:true,
          aws_profile_redacted:true,
          dispatcher_mode_changed:false,
          research_run_task_started:false,
          shadow_paper_live_enabled:false
        },
        counts:{
          step_started:count_event("step_started"),
          step_completed:count_event("step_completed"),
          step_skipped:count_event("step_skipped"),
          step_dry_run:count_event("step_dry_run"),
          completed_success:(completed_success | length),
          completed_failed:(completed_failed | length)
        },
        by_phase:(
          completed
          | group_by(.phase)
          | map({
              phase:.[0].phase,
              completed:length,
              success:(map(select(.exit_status == 0)) | length),
              failed:(map(select(.exit_status != 0)) | length)
            })
        ),
        failed_steps:(
          completed_failed
          | map({
              phase,
              step_index,
              step_key,
              exit_status,
              stdout_log,
              stderr_log
            })
        )
      }
    ' "$EVENTS_FILE" > "$SUMMARY_FILE"
}

prepare_recovery_runner_output() {
  RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$$}"
  require_absolute_dir_path "INTEL_CANDIDATE_MARKET_L1_RECOVERY_RUN_OUTPUT_DIR or second argument" "$OUTPUT_DIR"
  mkdir -p "$OUTPUT_DIR/logs"
  require_absolute_dir_path "recovery run log directory" "$OUTPUT_DIR/logs"
  EVENTS_FILE="$OUTPUT_DIR/events.jsonl"
  SUMMARY_FILE="$OUTPUT_DIR/summary.redacted.json"
  require_safe_output_file_path "recovery run events file" "$EVENTS_FILE"
  require_safe_output_file_path "recovery run summary file" "$SUMMARY_FILE"
  if [[ ! -f "$EVENTS_FILE" ]]; then
    : > "$EVENTS_FILE"
  fi

  emit_event "run_started" "$(jq -nc \
    --arg manifest_file "$MANIFEST_FILE" \
    --arg output_dir "$OUTPUT_DIR" \
    --arg dry_run "$DRY_RUN" \
    --arg phases "$PHASES_CSV" \
    '{manifest_file:$manifest_file,output_dir:$output_dir,dry_run:$dry_run,phases:$phases,bucket_values_redacted:true,aws_profile_redacted:true}')"
}

print_recovery_run_summary() {
  jq -r '
    "market_l1_recovery_run_summary=\(.output_dir)/summary.redacted.json",
    "status=\(.status)",
    "step_completed=\(.counts.step_completed)",
    "step_skipped=\(.counts.step_skipped)",
    "step_dry_run=\(.counts.step_dry_run)",
    "completed_success=\(.counts.completed_success)",
    "completed_failed=\(.counts.completed_failed)"
  ' "$SUMMARY_FILE"
}
