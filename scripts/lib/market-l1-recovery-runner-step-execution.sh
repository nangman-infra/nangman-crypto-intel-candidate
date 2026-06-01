# shellcheck shell=bash

step_key_for_file() {
  local phase="$1"
  local index="$2"
  local step_file="$3"

  jq -r --arg phase "$phase" --argjson index "$index" '
    [
      $phase,
      ($index | tostring),
      (.target // "unknown"),
      (.market_symbol // (.market_symbols // [] | join("+")) // "none"),
      (.start_ms | tostring),
      (.end_ms | tostring)
    ] | join(":")
  ' "$step_file"
}

emit_recovery_step_dry_run() {
  local phase="$1"
  local index="$2"
  local step_key="$3"
  local step_file="$4"

  emit_event "step_dry_run" "$(jq -c \
    --arg phase "$phase" \
    --argjson step_index "$index" \
    --arg step_key "$step_key" \
    '
    def step_identity:
      {
        target:(.target // null),
        start_ms:(.start_ms // null),
        end_ms:(.end_ms // null),
        market_symbol:(.market_symbol // null),
        market_symbols:(.market_symbols // []),
        command_args:(.command_args // []),
        writes_s3:(.writes_s3 // false),
        reads_s3:(.reads_s3 // false)
      };
    {phase:$phase,step_index:$step_index,step_key:$step_key,target,command_args,writes_s3:(.writes_s3 // false),reads_s3:(.reads_s3 // false),step_identity:step_identity}
    ' \
    "$step_file")"
}

execute_recovery_step() {
  local phase="$1"
  local index="$2"
  local step_key="$3"
  local step_file="$4"
  local safe_log_name="${phase}-${index}"
  local stdout_log="$OUTPUT_DIR/logs/${safe_log_name}.stdout.log"
  local stderr_log="$OUTPUT_DIR/logs/${safe_log_name}.stderr.log"
  local command_args=()
  local expanded_args=()
  local arg
  local started_at_ms
  local completed_at_ms
  local duration_ms
  local exit_status

  while IFS= read -r arg; do
    command_args+=("$arg")
  done < <(jq -r '.command_args[]' "$step_file")
  for arg in "${command_args[@]}"; do
    expanded_args+=("$(replace_placeholder "$arg")")
  done
  require_safe_output_file_path "recovery step stdout log" "$stdout_log"
  require_safe_output_file_path "recovery step stderr log" "$stderr_log"

  started_at_ms="$(date -u +%s000)"
  emit_event "step_started" "$(jq -nc \
    --arg phase "$phase" \
    --argjson step_index "$index" \
    --arg step_key "$step_key" \
    --arg stdout_log "$stdout_log" \
    --arg stderr_log "$stderr_log" \
    --slurpfile step "$step_file" \
    '
    def step_identity:
      {
        target:(.target // null),
        start_ms:(.start_ms // null),
        end_ms:(.end_ms // null),
        market_symbol:(.market_symbol // null),
        market_symbols:(.market_symbols // []),
        command_args:(.command_args // []),
        writes_s3:(.writes_s3 // false),
        reads_s3:(.reads_s3 // false)
      };
    {phase:$phase,step_index:$step_index,step_key:$step_key,target:$step[0].target,writes_s3:($step[0].writes_s3 // false),reads_s3:($step[0].reads_s3 // false),stdout_log:$stdout_log,stderr_log:$stderr_log,bucket_values_redacted:true,aws_profile_redacted:true,step_identity:($step[0] | step_identity)}
    ')"

  set +e
  "${expanded_args[@]}" > "$stdout_log" 2> "$stderr_log"
  exit_status=$?
  set -e
  completed_at_ms="$(date -u +%s000)"
  duration_ms=$((completed_at_ms - started_at_ms))
  emit_event "step_completed" "$(jq -nc \
    --arg phase "$phase" \
    --argjson step_index "$index" \
    --arg step_key "$step_key" \
    --argjson exit_status "$exit_status" \
    --argjson duration_ms "$duration_ms" \
    --arg stdout_log "$stdout_log" \
    --arg stderr_log "$stderr_log" \
    --slurpfile step "$step_file" \
    '
    def step_identity:
      {
        target:(.target // null),
        start_ms:(.start_ms // null),
        end_ms:(.end_ms // null),
        market_symbol:(.market_symbol // null),
        market_symbols:(.market_symbols // []),
        command_args:(.command_args // []),
        writes_s3:(.writes_s3 // false),
        reads_s3:(.reads_s3 // false)
      };
    {phase:$phase,step_index:$step_index,step_key:$step_key,target:$step[0].target,writes_s3:($step[0].writes_s3 // false),reads_s3:($step[0].reads_s3 // false),exit_status:$exit_status,duration_ms:$duration_ms,stdout_log:$stdout_log,stderr_log:$stderr_log,bucket_values_redacted:true,aws_profile_redacted:true,step_identity:($step[0] | step_identity)}
    ')"

  if [[ "$exit_status" -ne 0 ]]; then
    write_summary "failed"
    echo "market L1 recovery failed at $step_key; see $SUMMARY_FILE" >&2
    exit "$exit_status"
  fi
}
