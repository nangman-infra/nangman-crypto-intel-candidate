#!/usr/bin/env bash

MARKET_L1_RECOVERY_PLAN_VALIDATION_LIB_DIR="${MARKET_L1_RECOVERY_PLAN_VALIDATION_LIB_DIR:-$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)}"

# shellcheck source=scripts/lib/path-safety.sh
source "$MARKET_L1_RECOVERY_PLAN_VALIDATION_LIB_DIR/path-safety.sh"

require_market_l1_recovery_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

require_market_l1_recovery_absolute_file() {
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

require_market_l1_recovery_absolute_output_path() {
  require_absolute_output_path "$@"
}

require_market_l1_recovery_positive_integer() {
  local name="$1"
  local value="$2"
  if ! [[ "$value" =~ ^[0-9]+$ ]] || [[ "$value" == "0" ]]; then
    echo "$name must be a positive integer; got $value" >&2
    exit 1
  fi
}

validate_market_l1_recovery_plan_inputs() {
  require_market_l1_recovery_command date
  require_market_l1_recovery_command awk
  require_market_l1_recovery_command jq
  require_market_l1_recovery_command mktemp

  require_market_l1_recovery_absolute_file \
    "INTEL_CANDIDATE_SOURCE_GAP_FILE or first argument" \
    "$SOURCE_GAP_FILE"
  require_market_l1_recovery_absolute_output_path \
    "INTEL_CANDIDATE_MARKET_L1_RECOVERY_PLAN_OUTPUT or second argument" \
    "$OUTPUT_FILE"
  require_market_l1_recovery_positive_integer \
    "INTEL_CANDIDATE_MARKET_L1_RECOVERY_MARGIN_MS" \
    "$RECOVERY_MARGIN_MS"
  require_market_l1_recovery_positive_integer \
    "INTEL_CANDIDATE_MARKET_L1_WINDOW_MS" \
    "$MARKET_WINDOW_MS"
  require_market_l1_recovery_positive_integer \
    "INTEL_CANDIDATE_MARKET_L1_NORMALIZE_SCHEDULE_INTERVAL_MS" \
    "$NORMALIZE_SCHEDULE_INTERVAL_MS"

  if [[ "$MARKET_INGEST_APP_ROOT" != /* ]]; then
    echo "INTEL_CANDIDATE_MARKET_INGEST_APP_ROOT must be an absolute path" >&2
    exit 1
  fi
}

assert_market_l1_recovery_source_gap_schema() {
  local validation_error

  if ! validation_error="$(jq -r -s '
    def non_empty_string:
      (type == "string") and (length > 0);
    def non_negative_integer:
      (type == "number") and (. >= 0) and ((floor) == .);
    def context_array_error($symbol_index; $array_name; $records):
      if (($records // []) | type) != "array" then
        "symbols[\($symbol_index)].market_context_gap.\($array_name) must be an array"
      else
        [
          range(0; (($records // []) | length)) as $record_index
          | (($records // [])[$record_index]) as $record
          | if ($record | type) != "object" then
              "symbols[\($symbol_index)].market_context_gap.\($array_name)[\($record_index)] must be an object"
            elif (($record.event_basis_ms != null) and (($record.event_basis_ms | type) != "number")) then
              "symbols[\($symbol_index)].market_context_gap.\($array_name)[\($record_index)].event_basis_ms must be a number when present"
            elif (($record.event_basis_ms != null) and (($record.packet_id // "") | non_empty_string | not)) then
              "symbols[\($symbol_index)].market_context_gap.\($array_name)[\($record_index)].packet_id must be a non-empty string when event_basis_ms is present"
            else
              empty
            end
        ][0] // empty
      end;
    def symbol_error($symbol_index; $symbol):
      if ($symbol | type) != "object" then
        "symbols[\($symbol_index)] must be an object"
      elif (($symbol.symbol // "") | non_empty_string | not) then
        "symbols[\($symbol_index)].symbol must be a non-empty string"
      elif (($symbol.status // "") | non_empty_string | not) then
        "symbols[\($symbol_index)].status must be a non-empty string"
      elif (($symbol.primary_blocker // "") | non_empty_string | not) then
        "symbols[\($symbol_index)].primary_blocker must be a non-empty string"
      elif ($symbol.market_context_gap | type) != "object" then
        "symbols[\($symbol_index)].market_context_gap must be an object"
      else
        [
          context_array_error($symbol_index; "historical_terminal_missing_context_records"; $symbol.market_context_gap.historical_terminal_missing_context_records),
          context_array_error($symbol_index; "current_or_unknown_terminal_missing_context_records"; $symbol.market_context_gap.current_or_unknown_terminal_missing_context_records),
          context_array_error($symbol_index; "historical_pending_context_records"; $symbol.market_context_gap.historical_pending_context_records),
          context_array_error($symbol_index; "current_or_unknown_pending_context_records"; $symbol.market_context_gap.current_or_unknown_pending_context_records)
        ] | map(select(length > 0)) | .[0] // empty
      end;
    if length != 1 then
      "source gap diagnosis must contain exactly one JSON document"
    elif (.[0] | type) != "object" then
      "source gap diagnosis must be a JSON object"
    elif (.[0].schema_version // null) != "intel_candidate_source_gap_diagnosis_v2" then
      "source gap diagnosis must have schema_version=intel_candidate_source_gap_diagnosis_v2"
    elif (.[0].summary | type) != "object" then
      "summary must be an object"
    elif ((.[0].summary.approved_symbols_without_candidate | non_negative_integer) | not) then
      "summary.approved_symbols_without_candidate must be a non-negative integer"
    elif (.[0].summary.primary_blocker_counts | type) != "array" then
      "summary.primary_blocker_counts must be an array"
    elif (.[0].summary.global_market_context_gap | type) != "object" then
      "summary.global_market_context_gap must be an object"
    elif (.[0].symbols | type) != "array" then
      "symbols must be an array"
    elif .[0].summary.approved_symbols_without_candidate != (.[0].symbols | length) then
      "summary.approved_symbols_without_candidate must match symbols length"
    else
      [
        range(0; (.[0].symbols | length)) as $symbol_index
        | symbol_error($symbol_index; .[0].symbols[$symbol_index])
      ] | map(select(length > 0)) | .[0] // empty
    end
  ' "$SOURCE_GAP_FILE")"; then
    echo "source gap diagnosis must be valid JSON: $SOURCE_GAP_FILE" >&2
    exit 1
  fi

  if [[ -n "$validation_error" ]]; then
    echo "source gap diagnosis validation failed: $validation_error" >&2
    exit 1
  fi
}
