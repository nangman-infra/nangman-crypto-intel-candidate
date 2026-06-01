#!/usr/bin/env bash

sync_prefix() {
  local bucket="$1"
  local prefix="$2"
  local destination="$3"
  ensure_safe_export_dir "source gap input sync destination" "$destination"
  aws s3 sync "s3://$bucket/$prefix" "$destination/" --only-show-errors
}

ensure_safe_export_dir() {
  local name="$1"
  local path="$2"
  require_absolute_dir_path "$name" "$path"
  mkdir -p "$path"
  require_absolute_dir_path "$name" "$path"
}

ensure_export_output_dir() {
  require_absolute_dir_path "output directory or first argument" "$OUTPUT_DIR"
  if [[ -e "$OUTPUT_DIR" && "$ALLOW_EXISTING" != "true" ]]; then
    if [[ -n "$(find "$OUTPUT_DIR" -mindepth 1 -maxdepth 1 -print -quit)" ]]; then
      echo "output directory already contains files; set INTEL_CANDIDATE_SOURCE_GAP_ALLOW_EXISTING=true to reuse it" >&2
      exit 1
    fi
  fi
  ensure_safe_export_dir "output directory or first argument" "$OUTPUT_DIR"
}

prepare_source_gap_export_dirs() {
  ensure_safe_export_dir "structured packet export directory" "$structured_dir"
  ensure_safe_export_dir "screening event export directory" "$screening_dir"
  ensure_safe_export_dir "hypothesis state export directory" "$hypothesis_dir"
  ensure_safe_export_dir "evidence bundle export directory" "$evidence_dir"
}

resolve_task_definition() {
  if [[ -n "$TASK_DEFINITION" ]]; then
    return
  fi
  require_non_empty "INTEL_CANDIDATE_ECS_CLUSTER" "$ECS_CLUSTER"
  require_non_empty "INTEL_CANDIDATE_ECS_SERVICE" "$ECS_SERVICE"
  TASK_DEFINITION="$(aws ecs describe-services \
    --cluster "$ECS_CLUSTER" \
    --services "$ECS_SERVICE" \
    --query 'services[0].taskDefinition' \
    --output text)"
}

load_task_command_buckets() {
  command_json="$(aws ecs describe-task-definition \
    --task-definition "$TASK_DEFINITION" \
    --query 'taskDefinition.containerDefinitions[0].command' \
    --output json)"

  input_bucket="$(get_arg "--input-s3-bucket")"
  output_bucket="$(get_arg "--output-s3-bucket")"
  require_non_empty "--input-s3-bucket in task definition command" "$input_bucket"
  require_non_empty "--output-s3-bucket in task definition command" "$output_bucket"
  require_s3_bucket_name "--input-s3-bucket in task definition command" "$input_bucket"
  require_s3_bucket_name "--output-s3-bucket in task definition command" "$output_bucket"
}
