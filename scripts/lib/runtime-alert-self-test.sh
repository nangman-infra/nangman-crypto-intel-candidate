# shellcheck shell=bash

self_test() {
  require_command jq
  local tmp
  tmp="$(mktemp)"
  cat > "$tmp" <<'EOF'
service_not_fully_running desired=1 running=0
recent_error_log_count=1
candidate_output_bucket_check=skipped reason=INTEL_CANDIDATE_OUTPUT_S3_BUCKET_not_set
EOF
  local rendered
  rendered="$(message P1 "runtime check failed" "$tmp" "- ECS service와 최근 error log를 먼저 확인")"
  [[ "$rendered" == *"[P1][intel-candidate-app]"* ]] || die "self-test expected P1 title"
  [[ "$rendered" == *"candidate 생성 상태"* ]] || die "self-test expected candidate context"
  [[ "$rendered" == *"안전 상태:"* ]] || die "self-test expected safety state"
  rm -f "$tmp"

  local original_env_file="${ENV_FILE:-}"
  local env_tmp marker_tmp
  env_tmp="$(mktemp)"
  marker_tmp="$(mktemp)"
  rm -f "$marker_tmp"
  cat > "$env_tmp" <<EOF
# comments and blank lines are ignored
INTEL_CANDIDATE_SELF_TEST_VALUE=plain
export INTEL_CANDIDATE_SELF_TEST_QUOTED="quoted value"
INTEL_CANDIDATE_SELF_TEST_LITERAL=\$(touch "$marker_tmp")
EOF
  ENV_FILE="$env_tmp"
  load_env_file
  [[ "${INTEL_CANDIDATE_SELF_TEST_VALUE:-}" == "plain" ]] || die "self-test expected env value"
  [[ "${INTEL_CANDIDATE_SELF_TEST_QUOTED:-}" == "quoted value" ]] || die "self-test expected quoted env value"
  [[ "${INTEL_CANDIDATE_SELF_TEST_LITERAL:-}" == "\$(touch \"$marker_tmp\")" ]] || die "self-test expected literal env value"
  [[ ! -e "$marker_tmp" ]] || die "self-test must not execute env file commands"
  local original_nangman_pipeline_alert_s3_bucket="${NANGMAN_PIPELINE_ALERT_S3_BUCKET:-}"
  local original_nangman_alert_env="${NANGMAN_ALERT_ENV:-}"
  local original_pipeline_alert_s3_bucket="${PIPELINE_ALERT_S3_BUCKET:-}"
  local original_alert_env="${ALERT_ENV:-}"
  NANGMAN_PIPELINE_ALERT_S3_BUCKET="env-file-alert-bucket"
  NANGMAN_ALERT_ENV="env-file-env"
  apply_runtime_alert_env_defaults
  [[ "$PIPELINE_ALERT_S3_BUCKET" == "env-file-alert-bucket" ]] || die "self-test expected env file alert bucket"
  [[ "$ALERT_ENV" == "env-file-env" ]] || die "self-test expected env file alert env"
  NANGMAN_PIPELINE_ALERT_S3_BUCKET="$original_nangman_pipeline_alert_s3_bucket"
  NANGMAN_ALERT_ENV="$original_nangman_alert_env"
  PIPELINE_ALERT_S3_BUCKET="$original_pipeline_alert_s3_bucket"
  ALERT_ENV="$original_alert_env"
  ENV_FILE="$original_env_file"
  rm -f "$env_tmp" "$marker_tmp"

  local original_error_lookback_minutes="${ERROR_LOOKBACK_MINUTES:-}"
  if (ERROR_LOOKBACK_MINUTES="not-a-number"; validate_runtime_alert_config) >/dev/null 2>&1; then
    die "self-test expected invalid error lookback validation failure"
  fi
  if (PIPELINE_ALERT_S3_BUCKET="Bad_Bucket"; require_s3_bucket_name "test bucket" "$PIPELINE_ALERT_S3_BUCKET") >/dev/null 2>&1; then
    die "self-test expected invalid alert bucket validation failure"
  fi
  if (PIPELINE_ALERT_S3_BUCKET="192.168.5.4"; require_s3_bucket_name "test bucket" "$PIPELINE_ALERT_S3_BUCKET") >/dev/null 2>&1; then
    die "self-test expected IP-shaped alert bucket validation failure"
  fi
  if (OUTPUT_BUCKET="Bad_Bucket"; validate_runtime_alert_config) >/dev/null 2>&1; then
    die "self-test expected invalid output bucket validation failure"
  fi
  if (PIPELINE_ALERT_S3_PREFIX="s3://other-bucket/pipeline-alert"; require_s3_object_prefix "test prefix" "$PIPELINE_ALERT_S3_PREFIX") >/dev/null 2>&1; then
    die "self-test expected invalid alert prefix validation failure"
  fi
  if (require_alert_priority "test priority" "critical") >/dev/null 2>&1; then
    die "self-test expected invalid alert priority validation failure"
  fi
  ERROR_LOOKBACK_MINUTES="$original_error_lookback_minutes"

  local original_aws_region="${AWS_REGION:-}"
  local runtime_alert_self_test_counter=0
  local temp_root temp_count upload_status
  temp_root="$(mktemp -d)"
  runtime_alert_self_test_counter=0
  mktemp() {
    runtime_alert_self_test_counter=$((runtime_alert_self_test_counter + 1))
    local path="$temp_root/payload-$runtime_alert_self_test_counter.tmp"
    : > "$path"
    printf '%s\n' "$path"
  }
  aws() {
    if [[ "$1 $2" == "s3api put-object" ]]; then
      return 42
    fi
    return 9
  }
  PIPELINE_ALERT_S3_BUCKET="runtime-alert-self-test-bucket"
  AWS_REGION="ap-northeast-2"
  set +e
  send_pipeline_alert P1 "self-test upload failure" "upload failure body"
  upload_status=$?
  set -e
  [[ "$upload_status" -eq 42 ]] || die "self-test expected failed upload status"
  temp_count="$(find "$temp_root" -type f | wc -l | tr -d ' ')"
  [[ "$temp_count" == "0" ]] || die "self-test expected failed alert payload temp cleanup"
  unset -f aws
  unset -f mktemp
  PIPELINE_ALERT_S3_BUCKET="$original_pipeline_alert_s3_bucket"
  AWS_REGION="$original_aws_region"
  rm -rf "$temp_root"

  tmp="$(mktemp)"
  OUTPUT_BUCKET="candidate-output-bucket"
  aws() {
    if [[ "$1 $2" == "s3api list-objects-v2" ]]; then
      printf 'AccessDenied: denied\n' >&2
      return 42
    fi
    return 9
  }
  if check_latest_candidate_evidence "$tmp" >/dev/null 2>&1; then
    die "self-test expected candidate evidence check failure"
  fi
  grep -q "AccessDenied" "$tmp" || die "self-test expected evidence failure details"
  unset -f aws
  OUTPUT_BUCKET=""
  remove_temp_file "$tmp"

  tmp="$(mktemp)"
  aws() {
    if [[ "$1 $2" == "logs filter-log-events" ]]; then
      printf 'AccessDenied: denied logs\n' >&2
      return 42
    fi
    return 9
  }
  if check_recent_service_errors "$tmp" "/aws/ecs/intel-candidate" >/dev/null 2>&1; then
    die "self-test expected recent log check failure"
  fi
  grep -q "AccessDenied" "$tmp" || die "self-test expected recent log failure details"
  unset -f aws
  remove_temp_file "$tmp"

  log "send-runtime-alert self-test passed"
}
