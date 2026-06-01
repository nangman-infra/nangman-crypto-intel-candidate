#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
# shellcheck source=scripts/lib/source-gap-input-export.sh
source "$SCRIPT_DIR/lib/source-gap-input-export.sh"

OUTPUT_DIR="${INTEL_CANDIDATE_SOURCE_GAP_INPUT_OUTPUT_DIR:-${1:-}}"
SOURCE_DT="${INTEL_CANDIDATE_SOURCE_GAP_DT:-${2:-$(date -u +%F)}}"
ECS_CLUSTER="${INTEL_CANDIDATE_ECS_CLUSTER:-}"
ECS_SERVICE="${INTEL_CANDIDATE_ECS_SERVICE:-}"
TASK_DEFINITION="${INTEL_CANDIDATE_TASK_DEFINITION:-}"
ALLOW_EXISTING="${INTEL_CANDIDATE_SOURCE_GAP_ALLOW_EXISTING:-false}"
PRIORITIES="${INTEL_CANDIDATE_SOURCE_GAP_PRIORITIES:-p0,p1,p2}"

require_command aws
require_command date
require_command find
require_command jq
require_command mkdir
require_command tr
require_command wc
require_absolute_dir_path "output directory or first argument" "$OUTPUT_DIR"
validate_source_gap_export_inputs

resolve_task_definition
require_non_empty "task definition" "$TASK_DEFINITION"
ensure_export_output_dir

load_task_command_buckets
structured_dir="$OUTPUT_DIR/structured"
screening_dir="$OUTPUT_DIR/screening"
hypothesis_dir="$OUTPUT_DIR/hypothesis"
evidence_dir="$OUTPUT_DIR/evidence"
manifest_output="$OUTPUT_DIR/source-gap-input-manifest.json"

require_safe_output_file_path "source gap input manifest output" "$manifest_output"
prepare_source_gap_export_dirs
sync_source_gap_inputs
count_source_gap_inputs
task_family_revision="${TASK_DEFINITION##*/}"
write_source_gap_input_manifest
print_source_gap_export_summary
