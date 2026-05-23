#!/usr/bin/env bash
set -euo pipefail

OUTPUT_DIR="${INTEL_CANDIDATE_SOURCE_GAP_INPUT_OUTPUT_DIR:-${1:-}}"
SOURCE_DT="${INTEL_CANDIDATE_SOURCE_GAP_DT:-${2:-$(date -u +%F)}}"
ECS_CLUSTER="${INTEL_CANDIDATE_ECS_CLUSTER:-}"
ECS_SERVICE="${INTEL_CANDIDATE_ECS_SERVICE:-}"
TASK_DEFINITION="${INTEL_CANDIDATE_TASK_DEFINITION:-}"
ALLOW_EXISTING="${INTEL_CANDIDATE_SOURCE_GAP_ALLOW_EXISTING:-false}"
PRIORITIES="${INTEL_CANDIDATE_SOURCE_GAP_PRIORITIES:-p0,p1,p2}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
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

require_non_empty() {
  local name="$1"
  local value="$2"
  if [[ -z "$value" || "$value" == "None" ]]; then
    echo "$name is required" >&2
    exit 1
  fi
}

get_arg() {
  local flag="$1"
  jq -r --arg flag "$flag" '
    (. as $args
      | [range(0; length - 1) | select($args[.] == $flag) | $args[. + 1]][0])
    // empty
  ' <<< "$command_json"
}

count_files() {
  local path="$1"
  find "$path" -type f | wc -l | tr -d ' '
}

sync_prefix() {
  local bucket="$1"
  local prefix="$2"
  local destination="$3"
  mkdir -p "$destination"
  aws s3 sync "s3://$bucket/$prefix" "$destination/" --only-show-errors
}

require_command aws
require_command date
require_command find
require_command jq
require_command mkdir
require_command tr
require_command wc
require_absolute_dir_path "output directory or first argument" "$OUTPUT_DIR"

if [[ -z "$TASK_DEFINITION" ]]; then
  require_non_empty "INTEL_CANDIDATE_ECS_CLUSTER" "$ECS_CLUSTER"
  require_non_empty "INTEL_CANDIDATE_ECS_SERVICE" "$ECS_SERVICE"
  TASK_DEFINITION="$(aws ecs describe-services \
    --cluster "$ECS_CLUSTER" \
    --services "$ECS_SERVICE" \
    --query 'services[0].taskDefinition' \
    --output text)"
fi
require_non_empty "task definition" "$TASK_DEFINITION"

if [[ -e "$OUTPUT_DIR" && "$ALLOW_EXISTING" != "true" ]]; then
  if [[ -n "$(find "$OUTPUT_DIR" -mindepth 1 -maxdepth 1 -print -quit)" ]]; then
    echo "output directory already contains files; set INTEL_CANDIDATE_SOURCE_GAP_ALLOW_EXISTING=true to reuse it" >&2
    exit 1
  fi
fi

mkdir -p "$OUTPUT_DIR"

command_json="$(aws ecs describe-task-definition \
  --task-definition "$TASK_DEFINITION" \
  --query 'taskDefinition.containerDefinitions[0].command' \
  --output json)"

input_bucket="$(get_arg "--input-s3-bucket")"
output_bucket="$(get_arg "--output-s3-bucket")"
require_non_empty "--input-s3-bucket in task definition command" "$input_bucket"
require_non_empty "--output-s3-bucket in task definition command" "$output_bucket"

structured_dir="$OUTPUT_DIR/structured"
screening_dir="$OUTPUT_DIR/screening"
hypothesis_dir="$OUTPUT_DIR/hypothesis"
evidence_dir="$OUTPUT_DIR/evidence"
manifest_output="$OUTPUT_DIR/source-gap-input-manifest.json"

sync_prefix \
  "$input_bucket" \
  "structured-intel-packet/schema=structured_intel_packet_v1/dt=$SOURCE_DT/" \
  "$structured_dir"
sync_prefix \
  "$output_bucket" \
  "candidate-screening/schema=intel_candidate_screening_event_v1/dt=$SOURCE_DT/" \
  "$screening_dir"
sync_prefix \
  "$output_bucket" \
  "hypothesis-state/schema=intel_candidate_hypothesis_state_v1/dt=$SOURCE_DT/" \
  "$hypothesis_dir"

printf '%s\n' "$PRIORITIES" | tr ',' '\n' | while IFS= read -r priority; do
  [[ -z "$priority" ]] && continue
  sync_prefix \
    "$output_bucket" \
    "candidate-evidence-bundle/priority=$priority/schema=intel_candidate_evidence_bundle_v1/dt=$SOURCE_DT/" \
    "$evidence_dir/$priority"
done

structured_files="$(count_files "$structured_dir")"
screening_files="$(count_files "$screening_dir")"
hypothesis_files="$(count_files "$hypothesis_dir")"
evidence_files="$(count_files "$evidence_dir")"
task_family_revision="${TASK_DEFINITION##*/}"

jq -n \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg source_dt "$SOURCE_DT" \
  --arg output_dir "$OUTPUT_DIR" \
  --arg task_family_revision "$task_family_revision" \
  --arg structured_dir "$structured_dir" \
  --arg screening_dir "$screening_dir" \
  --arg hypothesis_dir "$hypothesis_dir" \
  --arg evidence_dir "$evidence_dir" \
  --argjson structured_files "$structured_files" \
  --argjson screening_files "$screening_files" \
  --argjson hypothesis_files "$hypothesis_files" \
  --argjson evidence_files "$evidence_files" \
  '{
    schema_version:"intel_candidate_source_gap_input_export_v1",
    generated_at:$generated_at,
    input:{
      source_dt:$source_dt,
      task_definition_family_revision:$task_family_revision
    },
    safety:{
      s3_read:true,
      s3_write:false,
      ecs_task_started:false,
      dispatcher_mode_changed:false,
      local_export_only:true,
      shadow_paper_live_enabled:false,
      bucket_names_redacted:true
    },
    local_paths:{
      output_dir:$output_dir,
      structured_packets:$structured_dir,
      screening_events:$screening_dir,
      hypothesis_states:$hypothesis_dir,
      evidence_bundles:$evidence_dir
    },
    counts:{
      structured_files:$structured_files,
      screening_files:$screening_files,
      hypothesis_files:$hypothesis_files,
      evidence_files:$evidence_files
    }
  }' > "$manifest_output"

{
  echo "source_gap_input_manifest=$manifest_output"
  echo "structured_packet_paths=$structured_dir"
  echo "screening_event_paths=$screening_dir"
  echo "hypothesis_state_paths=$hypothesis_dir"
  echo "evidence_bundle_paths=$evidence_dir"
  echo "structured_files=$structured_files"
  echo "screening_files=$screening_files"
  echo "hypothesis_files=$hypothesis_files"
  echo "evidence_files=$evidence_files"
  echo "source gap input export completed"
} >&2
