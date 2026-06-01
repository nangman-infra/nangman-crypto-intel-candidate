#!/usr/bin/env bash
set -euo pipefail

RECOVERY_PLAN_FILE="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_PLAN_FILE:-${1:-}}"
OUTPUT_FILE="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_EXECUTION_MANIFEST_OUTPUT:-${2:-}}"
READINESS_FILE="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_READINESS_FILE:-${3:-}}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/lib/market-l1-recovery-execution-output.sh
source "$SCRIPT_DIR/lib/market-l1-recovery-execution-output.sh"
# shellcheck source=scripts/lib/path-safety.sh
source "$SCRIPT_DIR/lib/path-safety.sh"

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

validate_recovery_plan_content() {
  local validation_error
  validation_error="$(jq -r '
    def positive_integer:
      (type == "number") and (. > 0) and ((floor) == .);
    def non_empty_string:
      (type == "string") and (length > 0);
    def absolute_path:
      non_empty_string and startswith("/");
    def window_error($symbol_index; $window_index; $window):
      if ($window | type) != "object" then
        "symbols[\($symbol_index)].recovery_windows[\($window_index)] must be an object"
      elif (($window.packet_id // "") | non_empty_string | not) then
        "symbols[\($symbol_index)].recovery_windows[\($window_index)].packet_id must be a non-empty string"
      elif (($window.recovery_input_start_ms | positive_integer) | not) then
        "symbols[\($symbol_index)].recovery_windows[\($window_index)].recovery_input_start_ms must be a positive integer"
      elif (($window.recovery_input_end_ms | positive_integer) | not) then
        "symbols[\($symbol_index)].recovery_windows[\($window_index)].recovery_input_end_ms must be a positive integer"
      elif $window.recovery_input_end_ms <= $window.recovery_input_start_ms then
        "symbols[\($symbol_index)].recovery_windows[\($window_index)] must have recovery_input_end_ms greater than recovery_input_start_ms"
      else
        empty
      end;
    def symbol_error($symbol_index; $symbol):
      if ($symbol | type) != "object" then
        "symbols[\($symbol_index)] must be an object"
      elif (($symbol.symbol // "") | non_empty_string | not) then
        "symbols[\($symbol_index)].symbol must be a non-empty string"
      elif (($symbol.market_symbol // "") | non_empty_string | not) then
        "symbols[\($symbol_index)].market_symbol must be a non-empty string"
      elif ($symbol.recovery_windows | type) != "array" then
        "symbols[\($symbol_index)].recovery_windows must be an array"
      else
        [
          range(0; ($symbol.recovery_windows | length)) as $window_index
          | window_error($symbol_index; $window_index; $symbol.recovery_windows[$window_index])
        ][0] // empty
      end;
    [
      if (.input | type) != "object" then
        "input must be an object"
      elif ((.input.market_ingest_app_root | absolute_path) | not) then
        "input.market_ingest_app_root must be an absolute path"
      elif ((.input.market_venue // "") | non_empty_string | not) then
        "input.market_venue must be a non-empty string"
      elif ((.input.market_window_ms | positive_integer) | not) then
        "input.market_window_ms must be a positive integer"
      elif ((.input.normalize_schedule_interval_ms | positive_integer) | not) then
        "input.normalize_schedule_interval_ms must be a positive integer"
      elif (.symbols | type) != "array" then
        "symbols must be an array"
      else
        empty
      end,
      if (.symbols | type) == "array" then
        range(0; (.symbols | length)) as $symbol_index
        | symbol_error($symbol_index; .symbols[$symbol_index])
      else
        empty
      end
    ]
    | map(select(length > 0))
    | .[0] // empty
  ' "$RECOVERY_PLAN_FILE")"

  if [[ -n "$validation_error" ]]; then
    echo "recovery plan validation failed: $validation_error" >&2
    exit 1
  fi
}

validate_readiness_file_content() {
  local validation_error
  validation_error="$(jq -r -s '
    def non_empty_string:
      (type == "string") and (length > 0);
    def non_negative_integer:
      (type == "number") and (. >= 0) and ((floor) == .);
    def boolean_field($path; $value):
      if ($value | type) != "boolean" then
        "\($path) must be a boolean"
      else
        empty
      end;
    def count_field($path; $value):
      if (($value | non_negative_integer) | not) then
        "\($path) must be a non-negative integer"
      else
        empty
      end;
    if length != 1 then
      "readiness file must contain exactly one JSON document"
    elif (.[0] | type) != "object" then
      "readiness file must be a JSON object"
    elif (.[0].schema_version // null) != "candidate_market_l1_recovery_readiness_v1" then
      "readiness file must have schema_version=candidate_market_l1_recovery_readiness_v1"
    elif ((.[0].verdict // "") | non_empty_string | not) then
      "verdict must be a non-empty string"
    elif (.[0].current_state | type) != "object" then
      "current_state must be an object"
    elif (.[0].evidence | type) != "object" then
      "evidence must be an object"
    else
      [
        boolean_field("current_state.l1_index_audit_completed"; .[0].current_state.l1_index_audit_completed),
        boolean_field("current_state.exchange_symbol_check_completed"; .[0].current_state.exchange_symbol_check_completed),
        boolean_field("current_state.promotion_passed"; .[0].current_state.promotion_passed),
        boolean_field("current_state.shadow_created"; .[0].current_state.shadow_created),
        boolean_field("current_state.paper_created"; .[0].current_state.paper_created),
        boolean_field("current_state.live_enabled"; .[0].current_state.live_enabled),
        count_field("evidence.audit_missing_index_pointer_count_total"; .[0].evidence.audit_missing_index_pointer_count_total),
        count_field("evidence.exchangeinfo_missing_symbol_count"; .[0].evidence.exchangeinfo_missing_symbol_count),
        count_field("evidence.exchangeinfo_non_trading_symbol_count"; .[0].evidence.exchangeinfo_non_trading_symbol_count)
      ]
      | map(select(length > 0))
      | .[0] // empty
    end
  ' "$READINESS_FILE")"

  if [[ -n "$validation_error" ]]; then
    echo "readiness file validation failed: $validation_error" >&2
    exit 1
  fi
}

require_command date
require_command jq
require_command mktemp

require_absolute_file "INTEL_CANDIDATE_MARKET_L1_RECOVERY_PLAN_FILE or first argument" "$RECOVERY_PLAN_FILE"
require_absolute_output_path "INTEL_CANDIDATE_MARKET_L1_RECOVERY_EXECUTION_MANIFEST_OUTPUT or second argument" "$OUTPUT_FILE"
if [[ -n "$READINESS_FILE" ]]; then
  require_absolute_file "INTEL_CANDIDATE_MARKET_L1_RECOVERY_READINESS_FILE or third argument" "$READINESS_FILE"
fi

jq -e '.schema_version == "candidate_market_l1_recovery_plan_v1"' "$RECOVERY_PLAN_FILE" >/dev/null || {
  echo "recovery plan must have schema_version=candidate_market_l1_recovery_plan_v1" >&2
  exit 1
}
validate_recovery_plan_content

if [[ -n "$READINESS_FILE" ]]; then
  validate_readiness_file_content
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

readiness_json="$tmp_dir/readiness.json"
if [[ -n "$READINESS_FILE" ]]; then
  cp "$READINESS_FILE" "$readiness_json"
else
  printf 'null\n' > "$readiness_json"
fi

tmp_output="$tmp_dir/market-l1-recovery-execution-manifest.json"

write_market_l1_recovery_execution_manifest "$tmp_output" "$readiness_json"

if [[ -n "$OUTPUT_FILE" ]]; then
  cp "$tmp_output" "$OUTPUT_FILE"
  {
    echo "market_l1_recovery_execution_manifest_output=$OUTPUT_FILE"
    jq -r '
      "source_recovery_window_count=\(.summary.source_recovery_window_count)",
      "context_recovery_symbol_count=\(.summary.context_recovery_symbol_count)",
      "terminal_missing_symbol_count=\(.summary.terminal_missing_symbol_count)",
      "pending_context_symbol_count=\(.summary.pending_context_symbol_count)",
      "backfill_step_count=\(.summary.backfill_step_count)",
      "normalize_step_count=\(.summary.normalize_step_count)",
      "post_audit_step_count=\(.summary.post_audit_step_count)",
      "approval_required=\(.approval_boundary.required)"
    ' "$OUTPUT_FILE"
  } >&2
else
  cat "$tmp_output"
fi
