#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
JQ_DIR="$SCRIPT_DIR/jq"
JQ_PROGRAM="$JQ_DIR/diagnose-candidate-source-gaps.jq"

# shellcheck source=scripts/lib/source-gap-diagnosis-runtime.sh
source "$SCRIPT_DIR/lib/source-gap-diagnosis-runtime.sh"

COVERAGE_GAP_FILE="${INTEL_CANDIDATE_COVERAGE_GAP_FILE:-${1:-}}"
OUTPUT_FILE="${INTEL_CANDIDATE_SOURCE_GAP_OUTPUT:-${2:-}}"

STRUCTURED_PACKET_PATHS="${INTEL_CANDIDATE_STRUCTURED_PACKET_PATHS:-}"
SCREENING_EVENT_PATHS="${INTEL_CANDIDATE_SCREENING_EVENT_PATHS:-}"
HYPOTHESIS_STATE_PATHS="${INTEL_CANDIDATE_HYPOTHESIS_STATE_PATHS:-}"
EVIDENCE_BUNDLE_PATHS="${INTEL_CANDIDATE_EVIDENCE_BUNDLE_PATHS:-}"

require_command date
require_command find
require_command jq
require_command mktemp
require_command sort
require_absolute_file "diagnosis jq program" "$JQ_PROGRAM"
require_supported_coverage_gap_file "INTEL_CANDIDATE_COVERAGE_GAP_FILE or first argument" "$COVERAGE_GAP_FILE"
require_absolute_output_path "INTEL_CANDIDATE_SOURCE_GAP_OUTPUT or second argument" "$OUTPUT_FILE"

structured_paths=()
while IFS= read -r path; do
  structured_paths+=("$path")
done < <(collect_json_paths "INTEL_CANDIDATE_STRUCTURED_PACKET_PATHS" "$STRUCTURED_PACKET_PATHS")

screening_paths=()
while IFS= read -r path; do
  screening_paths+=("$path")
done < <(collect_json_paths "INTEL_CANDIDATE_SCREENING_EVENT_PATHS" "$SCREENING_EVENT_PATHS")

hypothesis_paths=()
while IFS= read -r path; do
  hypothesis_paths+=("$path")
done < <(collect_json_paths "INTEL_CANDIDATE_HYPOTHESIS_STATE_PATHS" "$HYPOTHESIS_STATE_PATHS")

evidence_paths=()
while IFS= read -r path; do
  evidence_paths+=("$path")
done < <(collect_json_paths "INTEL_CANDIDATE_EVIDENCE_BUNDLE_PATHS" "$EVIDENCE_BUNDLE_PATHS")

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

structured_records="$tmp_dir/structured.jsonl"
screening_records="$tmp_dir/screening.jsonl"
hypothesis_records="$tmp_dir/hypothesis.jsonl"
evidence_records="$tmp_dir/evidence.jsonl"
tmp_output="$tmp_dir/output.json"

if [[ "${#structured_paths[@]}" -gt 0 ]]; then
  write_json_records_or_empty "$structured_records" "${structured_paths[@]}"
else
  write_json_records_or_empty "$structured_records"
fi

if [[ "${#screening_paths[@]}" -gt 0 ]]; then
  write_json_records_or_empty "$screening_records" "${screening_paths[@]}"
else
  write_json_records_or_empty "$screening_records"
fi

if [[ "${#hypothesis_paths[@]}" -gt 0 ]]; then
  write_json_records_or_empty "$hypothesis_records" "${hypothesis_paths[@]}"
else
  write_json_records_or_empty "$hypothesis_records"
fi

if [[ "${#evidence_paths[@]}" -gt 0 ]]; then
  write_json_records_or_empty "$evidence_records" "${evidence_paths[@]}"
else
  write_json_records_or_empty "$evidence_records"
fi

jq -n \
  -L "$JQ_DIR" \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg coverage_gap_file "$COVERAGE_GAP_FILE" \
  --argjson structured_path_count "${#structured_paths[@]}" \
  --argjson screening_path_count "${#screening_paths[@]}" \
  --argjson hypothesis_path_count "${#hypothesis_paths[@]}" \
  --argjson evidence_path_count "${#evidence_paths[@]}" \
  --slurpfile coverage "$COVERAGE_GAP_FILE" \
  --slurpfile structured "$structured_records" \
  --slurpfile screening "$screening_records" \
  --slurpfile hypothesis "$hypothesis_records" \
  --slurpfile evidence "$evidence_records" \
  -f "$JQ_PROGRAM" > "$tmp_output"

emit_source_gap_diagnosis_result "$tmp_output" "$OUTPUT_FILE"

echo "candidate source gap diagnosis completed" >&2
