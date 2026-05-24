#!/usr/bin/env bash
set -euo pipefail

MANIFEST_FILE="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_EXECUTION_MANIFEST_FILE:-${1:-}}"
OUTPUT_DIR="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_RUN_OUTPUT_DIR:-${2:-}}"
APPROVAL="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_APPROVAL:-}"
DRY_RUN="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_DRY_RUN:-false}"
RESUME="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_RESUME:-true}"

PHASES_CSV="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_PHASES:-backfill,normalize,post_audit}"

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

require_absolute_dir_path() {
  local name="$1"
  local path="$2"
  if [[ -z "$path" || "$path" != /* ]]; then
    echo "$name must be an absolute directory path" >&2
    exit 1
  fi
}

require_real_env() {
  local name="$1"
  local value="${!name:-}"
  if [[ -z "$value" || "$value" == *"<"* || "$value" == *">"* ]]; then
    echo "$name must be set to a real value before non-dry-run execution" >&2
    exit 1
  fi
}

is_true() {
  case "$1" in
    1 | true | TRUE | yes | YES) return 0 ;;
    *) return 1 ;;
  esac
}

phase_array_name() {
  case "$1" in
    backfill) printf 'backfill_steps\n' ;;
    normalize) printf 'normalize_steps\n' ;;
    post_audit) printf 'post_audit_steps\n' ;;
    *)
      echo "unknown phase: $1" >&2
      exit 1
      ;;
  esac
}

replace_placeholder() {
  case "$1" in
    '${AWS_REGION}') printf '%s\n' "$AWS_REGION" ;;
    '${MARKET_L0_BUCKET}') printf '%s\n' "$MARKET_L0_BUCKET" ;;
    '${MARKET_L1_BUCKET}') printf '%s\n' "$MARKET_L1_BUCKET" ;;
    '${MARKET_L0_SPOOL_ROOT}') printf '%s\n' "$MARKET_L0_SPOOL_ROOT" ;;
    '${MARKET_L1_SPOOL_ROOT}') printf '%s\n' "$MARKET_L1_SPOOL_ROOT" ;;
    '${MARKET_NORMALIZE_CATCHUP_TMP_ROOT}') printf '%s\n' "$MARKET_NORMALIZE_CATCHUP_TMP_ROOT" ;;
    *) printf '%s\n' "$1" ;;
  esac
}

json_string() {
  jq -Rn --arg value "$1" '$value'
}

emit_event() {
  local event="$1"
  local payload="$2"
  jq -nc \
    --arg event "$event" \
    --argjson timestamp_ms "$(date -u +%s000)" \
    --argjson payload "$payload" \
    '{schema_version:"candidate_market_l1_recovery_run_event_v1",event:$event,timestamp_ms:$timestamp_ms} + $payload' \
    >> "$EVENTS_FILE"
}

completed_successfully() {
  local step_key="$1"
  [[ -f "$EVENTS_FILE" ]] || return 1
  jq -e --arg step_key "$step_key" '
    select(.event == "step_completed" and .step_key == $step_key and .exit_status == 0)
  ' "$EVENTS_FILE" >/dev/null 2>&1
}

write_summary() {
  local status="$1"
  jq -s \
    --arg status "$status" \
    --arg manifest_file "$MANIFEST_FILE" \
    --arg output_dir "$OUTPUT_DIR" \
    '
      def count_event($name):
        map(select(.event == $name)) | length;
      def completed:
        map(select(.event == "step_completed"));
      def completed_success:
        completed | map(select(.exit_status == 0));
      def completed_failed:
        completed | map(select(.exit_status != 0));
      {
        schema_version:"candidate_market_l1_recovery_run_summary_v1",
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

require_command date
require_command jq
require_command mkdir
require_command mktemp

require_absolute_file "INTEL_CANDIDATE_MARKET_L1_RECOVERY_EXECUTION_MANIFEST_FILE or first argument" "$MANIFEST_FILE"
require_absolute_dir_path "INTEL_CANDIDATE_MARKET_L1_RECOVERY_RUN_OUTPUT_DIR or second argument" "$OUTPUT_DIR"

jq -e '.schema_version == "candidate_market_l1_recovery_execution_manifest_v1"' "$MANIFEST_FILE" >/dev/null || {
  echo "manifest must have schema_version=candidate_market_l1_recovery_execution_manifest_v1" >&2
  exit 1
}

approval_phrase="$(jq -r '.approval_boundary.approval_phrase // empty' "$MANIFEST_FILE")"
if [[ -z "$approval_phrase" ]]; then
  echo "manifest approval_boundary.approval_phrase is missing" >&2
  exit 1
fi

if ! is_true "$DRY_RUN"; then
  if [[ "$APPROVAL" != "$approval_phrase" ]]; then
    echo "set INTEL_CANDIDATE_MARKET_L1_RECOVERY_APPROVAL=$approval_phrase to execute writes" >&2
    exit 1
  fi
  require_real_env AWS_PROFILE
  require_real_env AWS_REGION
  require_real_env MARKET_L0_BUCKET
  require_real_env MARKET_L1_BUCKET
  require_real_env MARKET_L0_SPOOL_ROOT
  require_real_env MARKET_L1_SPOOL_ROOT
  require_real_env MARKET_NORMALIZE_CATCHUP_TMP_ROOT
fi

mkdir -p "$OUTPUT_DIR/logs"
EVENTS_FILE="$OUTPUT_DIR/events.jsonl"
SUMMARY_FILE="$OUTPUT_DIR/summary.redacted.json"
if [[ ! -f "$EVENTS_FILE" ]]; then
  : > "$EVENTS_FILE"
fi

emit_event "run_started" "$(jq -nc \
  --arg manifest_file "$MANIFEST_FILE" \
  --arg output_dir "$OUTPUT_DIR" \
  --arg dry_run "$DRY_RUN" \
  --arg phases "$PHASES_CSV" \
  '{manifest_file:$manifest_file,output_dir:$output_dir,dry_run:$dry_run,phases:$phases,bucket_values_redacted:true,aws_profile_redacted:true}')"

IFS=',' read -r -a phases <<< "$PHASES_CSV"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

for phase in "${phases[@]}"; do
  phase="$(printf '%s' "$phase" | xargs)"
  [[ -n "$phase" ]] || continue
  array_name="$(phase_array_name "$phase")"
  step_count="$(jq -r ".${array_name} | length" "$MANIFEST_FILE")"
  for ((index = 0; index < step_count; index += 1)); do
    step_file="$tmp_dir/${phase}-${index}.json"
    jq ".${array_name}[${index}]" "$MANIFEST_FILE" > "$step_file"
    step_key="$(jq -r --arg phase "$phase" --argjson index "$index" '
      [
        $phase,
        ($index | tostring),
        (.target // "unknown"),
        (.market_symbol // (.market_symbols // [] | join("+")) // "none"),
        (.start_ms | tostring),
        (.end_ms | tostring)
      ] | join(":")
    ' "$step_file")"

    if is_true "$RESUME" && completed_successfully "$step_key"; then
      emit_event "step_skipped" "$(jq -nc \
        --arg phase "$phase" \
        --argjson step_index "$index" \
        --arg step_key "$step_key" \
        '{phase:$phase,step_index:$step_index,step_key:$step_key,reason:"already_completed"}')"
      continue
    fi

    if is_true "$DRY_RUN"; then
      emit_event "step_dry_run" "$(jq -c \
        --arg phase "$phase" \
        --argjson step_index "$index" \
        --arg step_key "$step_key" \
        '{phase:$phase,step_index:$step_index,step_key:$step_key,target,command_args,writes_s3:(.writes_s3 // false),reads_s3:(.reads_s3 // false)}' \
        "$step_file")"
      continue
    fi

    command_args=()
    while IFS= read -r arg; do
      command_args+=("$arg")
    done < <(jq -r '.command_args[]' "$step_file")
    expanded_args=()
    for arg in "${command_args[@]}"; do
      expanded_args+=("$(replace_placeholder "$arg")")
    done

    safe_log_name="${phase}-${index}"
    stdout_log="$OUTPUT_DIR/logs/${safe_log_name}.stdout.log"
    stderr_log="$OUTPUT_DIR/logs/${safe_log_name}.stderr.log"
    started_at_ms="$(date -u +%s000)"
    emit_event "step_started" "$(jq -nc \
      --arg phase "$phase" \
      --argjson step_index "$index" \
      --arg step_key "$step_key" \
      --arg stdout_log "$stdout_log" \
      --arg stderr_log "$stderr_log" \
      --slurpfile step "$step_file" \
      '{phase:$phase,step_index:$step_index,step_key:$step_key,target:$step[0].target,writes_s3:($step[0].writes_s3 // false),reads_s3:($step[0].reads_s3 // false),stdout_log:$stdout_log,stderr_log:$stderr_log,bucket_values_redacted:true,aws_profile_redacted:true}')"

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
      '{phase:$phase,step_index:$step_index,step_key:$step_key,target:$step[0].target,writes_s3:($step[0].writes_s3 // false),reads_s3:($step[0].reads_s3 // false),exit_status:$exit_status,duration_ms:$duration_ms,stdout_log:$stdout_log,stderr_log:$stderr_log,bucket_values_redacted:true,aws_profile_redacted:true}')"

    if [[ "$exit_status" -ne 0 ]]; then
      write_summary "failed"
      echo "market L1 recovery failed at $step_key; see $SUMMARY_FILE" >&2
      exit "$exit_status"
    fi
  done
done

emit_event "run_completed" "$(jq -nc '{status:"success"}')"
write_summary "success"
jq -r '
  "market_l1_recovery_run_summary=\(.output_dir)/summary.redacted.json",
  "status=\(.status)",
  "step_completed=\(.counts.step_completed)",
  "step_skipped=\(.counts.step_skipped)",
  "step_dry_run=\(.counts.step_dry_run)",
  "completed_success=\(.counts.completed_success)",
  "completed_failed=\(.counts.completed_failed)"
' "$SUMMARY_FILE"
