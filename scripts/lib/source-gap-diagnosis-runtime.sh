#!/usr/bin/env bash

SOURCE_GAP_DIAGNOSIS_LIB_DIR="${SOURCE_GAP_DIAGNOSIS_LIB_DIR:-$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)}"

# shellcheck source=scripts/lib/path-safety.sh
source "$SOURCE_GAP_DIAGNOSIS_LIB_DIR/path-safety.sh"

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
  if [[ -z "${2:-}" ]]; then
    return
  fi
  require_safe_output_file_path "$@"
}

require_supported_coverage_gap_file() {
  local name="$1"
  local path="$2"
  local expected_schema="intel_candidate_coverage_gap_diagnosis_v1"
  local validation_error

  require_absolute_file "$name" "$path"

  if ! validation_error="$(jq -r -s --arg expected_schema "$expected_schema" '
    if length != 1 then
      "must contain exactly one JSON document"
    elif (.[0] | type) != "object" then
      "must be a JSON object"
    elif (.[0].schema_version // null) != $expected_schema then
      "schema_version must be \($expected_schema); got \((.[0].schema_version // "missing") | tostring)"
    elif (.[0].coverage | type) != "object" then
      "coverage must be an object"
    elif (.[0].coverage.blocking_stage | type) != "string" then
      "coverage.blocking_stage must be a string"
    elif (.[0].coverage.gap_counts | type) != "object" then
      "coverage.gap_counts must be an object"
    elif (.[0].coverage.gap_counts.approved_symbols_without_candidate | type) != "number" then
      "coverage.gap_counts.approved_symbols_without_candidate must be a number"
    elif (.[0].gaps | type) != "object" then
      "gaps must be an object"
    elif (.[0].gaps.approved_symbols_without_candidate | type) != "array" then
      "gaps.approved_symbols_without_candidate must be an array"
    elif (.[0].gaps.approved_symbols_without_candidate | all(.[]; ((type == "string") and (length > 0)))) | not then
      "gaps.approved_symbols_without_candidate must contain only non-empty strings"
    elif .[0].coverage.gap_counts.approved_symbols_without_candidate != (.[0].gaps.approved_symbols_without_candidate | length) then
      "coverage.gap_counts.approved_symbols_without_candidate must match gaps.approved_symbols_without_candidate length"
    else
      empty
    end
  ' "$path")"; then
    echo "$name must be valid JSON: $path" >&2
    exit 1
  fi

  if [[ -n "$validation_error" ]]; then
    echo "$name is not a supported coverage gap diagnosis file: $path" >&2
    echo "$validation_error" >&2
    exit 1
  fi
}

collect_json_paths() {
  local name="$1"
  local raw="$2"
  if [[ -z "$raw" ]]; then
    return
  fi

  printf '%s\n' "$raw" | tr ',' '\n' | while IFS= read -r path; do
    [[ -z "$path" ]] && continue
    if [[ "$path" != /* ]]; then
      echo "$name must contain only absolute file or directory paths; got $path" >&2
      exit 1
    fi
    if [[ -f "$path" ]]; then
      printf '%s\n' "$path"
    elif [[ -d "$path" ]]; then
      find "$path" -type f \( -name '*.json' -o -name '*.jsonl' \) | sort
    else
      echo "$name path does not exist: $path" >&2
      exit 1
    fi
  done
}

append_json_records() {
  local destination="$1"
  shift

  : > "$destination"
  for path in "$@"; do
    jq -c 'if type == "array" then .[] else . end' "$path" >> "$destination"
  done
}

write_json_records_or_empty() {
  local destination="$1"
  shift

  if [[ "$#" -gt 0 ]]; then
    append_json_records "$destination" "$@"
  else
    : > "$destination"
  fi
}

emit_source_gap_diagnosis_result() {
  local tmp_output="$1"
  local output_file="$2"

  if [[ -n "$output_file" ]]; then
    cp "$tmp_output" "$output_file"
    {
      echo "source_gap_output=$output_file"
      jq -r '
        "approved_symbols_without_candidate=\(.summary.approved_symbols_without_candidate)",
        "status_counts=\(.summary.status_counts | map("\(.status):\(.count)") | join(","))",
        "primary_blockers=\(.summary.primary_blocker_counts | map("\(.primary_blocker):\(.count)") | join(","))",
        "top_rejection_reasons=\(.summary.global_rejection_reasons[0:5] | map("\(.reason):\(.count)") | join(","))"
      ' "$tmp_output"
    } >&2
  else
    cat "$tmp_output"
  fi
}
