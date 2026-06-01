# shellcheck shell=bash

derive_log_group() {
  local task_definition="$1"
  aws ecs describe-task-definition \
    --region "$AWS_REGION" \
    --task-definition "$task_definition" \
    --query 'taskDefinition.containerDefinitions[0].logConfiguration.options."awslogs-group"' \
    --output text
}

check_service() {
  local output_file="$1"
  local failures=0
  local service_json
  service_json="$(aws ecs describe-services \
    --region "$AWS_REGION" \
    --cluster "$CLUSTER" \
    --services "$SERVICE" \
    --output json)"

  jq '{services:.services[] | {serviceName,desiredCount,runningCount,pendingCount,status,taskDefinition,rolloutState:(.deployments[0].rolloutState // "unknown")}}' \
    <<< "$service_json" | redact >> "$output_file"

  local desired running task_definition
  desired="$(jq -r '.services[0].desiredCount // 0' <<< "$service_json")"
  running="$(jq -r '.services[0].runningCount // 0' <<< "$service_json")"
  task_definition="$(jq -r '.services[0].taskDefinition // empty' <<< "$service_json")"
  if [[ "$desired" == "0" || "$running" != "$desired" ]]; then
    printf 'service_not_fully_running desired=%s running=%s\n' "$desired" "$running" >> "$output_file"
    failures=$((failures + 1))
  fi

  if [[ -n "$task_definition" && "$task_definition" != "null" ]]; then
    local log_group
    if ! log_group="$(derive_log_group "$task_definition" 2>&1)"; then
      printf 'log_group_lookup_failed=true\n' >> "$output_file"
      printf '%s\n' "$log_group" | redact >> "$output_file"
      failures=$((failures + 1))
    else
      printf 'derived_log_group=%s\n' "$log_group" | redact >> "$output_file"
      if [[ -n "$log_group" && "$log_group" != "None" ]]; then
        if ! check_recent_service_errors "$output_file" "$log_group"; then
          failures=$((failures + 1))
        fi
      fi
    fi
  fi

  if ! check_latest_candidate_evidence "$output_file"; then
    failures=$((failures + 1))
  fi
  return "$failures"
}

check_recent_service_errors() {
  local output_file="$1"
  local log_group="$2"
  local start_time error_events error_count

  start_time="$((($(date -u +%s) - (ERROR_LOOKBACK_MINUTES * 60)) * 1000))"
  if ! error_events="$(aws logs filter-log-events \
    --region "$AWS_REGION" \
    --log-group-name "$log_group" \
    --start-time "$start_time" \
    --filter-pattern 'panic ?ERROR ?error ?AccessDenied ?OutOfMemory ?SIGKILL ?Killed' \
    --limit 10 \
    --query 'events[].{timestamp:timestamp,message:message}' \
    --output json 2>&1)"; then
    printf 'recent_error_log_check_failed=true\n' >> "$output_file"
    printf '%s\n' "$error_events" | redact >> "$output_file"
    return 1
  fi
  if ! error_count="$(jq 'length' <<< "$error_events")"; then
    printf 'recent_error_log_parse_failed=true\n' >> "$output_file"
    printf '%s\n' "$error_events" | redact >> "$output_file"
    return 1
  fi
  printf 'recent_error_log_count=%s\n' "$error_count" >> "$output_file"
  if [[ "$error_count" != "0" ]]; then
    jq '.' <<< "$error_events" | redact >> "$output_file"
    return 1
  fi
  return 0
}

check_latest_candidate_evidence() {
  local output_file="$1"
  if [[ -n "$OUTPUT_BUCKET" ]]; then
    append_check "$output_file" "latest candidate evidence bundle" \
      aws s3api list-objects-v2 \
        --region "$AWS_REGION" \
        --bucket "$OUTPUT_BUCKET" \
        --prefix candidate-evidence-bundle/ \
        --max-items 1000 \
        --query 'sort_by(Contents || `[]`, &LastModified)[-1].{key:Key,lastModified:LastModified,size:Size}' \
        --output json || return 1
  else
    printf 'candidate_output_bucket_check=skipped reason=INTEL_CANDIDATE_OUTPUT_S3_BUCKET_not_set\n' >> "$output_file"
  fi
  return 0
}
