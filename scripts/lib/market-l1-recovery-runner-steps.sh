# shellcheck shell=bash

run_recovery_phase_steps() {
  local phase="$1"
  local array_name
  local step_count
  local index
  local step_file
  local step_key

  array_name="$(phase_array_name "$phase")"
  step_count="$(jq -r ".${array_name} | length" "$MANIFEST_FILE")"
  for ((index = 0; index < step_count; index += 1)); do
    step_file="$tmp_dir/${phase}-${index}.json"
    jq ".${array_name}[${index}]" "$MANIFEST_FILE" > "$step_file"
    step_key="$(step_key_for_file "$phase" "$index" "$step_file")"

    if is_true "$RESUME" && completed_successfully "$step_key" "$step_file"; then
      emit_event "step_skipped" "$(jq -nc \
        --arg phase "$phase" \
        --argjson step_index "$index" \
        --arg step_key "$step_key" \
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
        {phase:$phase,step_index:$step_index,step_key:$step_key,reason:"already_completed",step_identity:($step[0] | step_identity)}
        ')"
      continue
    fi

    if is_true "$DRY_RUN"; then
      emit_recovery_step_dry_run "$phase" "$index" "$step_key" "$step_file"
      continue
    fi

    execute_recovery_step "$phase" "$index" "$step_key" "$step_file"
  done
}

run_recovery_manifest_steps() {
  local phases=()
  local phase

  IFS=',' read -r -a phases <<< "$PHASES_CSV"
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' EXIT

  for phase in "${phases[@]}"; do
    phase="$(printf '%s' "$phase" | xargs)"
    [[ -n "$phase" ]] || continue
    run_recovery_phase_steps "$phase"
  done
}
