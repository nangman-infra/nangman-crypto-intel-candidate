#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
JQ_PROGRAM="$SCRIPT_DIR/jq/diagnose-candidate-coverage-gaps.jq"
# shellcheck source=scripts/lib/path-safety.sh
source "$SCRIPT_DIR/lib/path-safety.sh"

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

require_command date
require_command jq
require_command mktemp
require_absolute_file "coverage gap jq program" "$JQ_PROGRAM"
require_absolute_file "INTEL_CANDIDATE_RESEARCH_STATUS_FILE or first argument" "$STATUS_FILE"
require_absolute_output_path "INTEL_CANDIDATE_COVERAGE_GAP_OUTPUT or second argument" "$OUTPUT_FILE"

tmp_output="$(mktemp)"
trap 'rm -f "$tmp_output"' EXIT

jq \
  -L "$SCRIPT_DIR/jq" \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg status_file "$STATUS_FILE" \
  -f "$JQ_PROGRAM" \
  "$STATUS_FILE" > "$tmp_output"

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
