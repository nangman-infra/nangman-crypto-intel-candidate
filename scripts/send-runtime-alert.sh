#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
APP_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd -P)"
ENV_FILE="${INTEL_CANDIDATE_ENV_FILE:-$APP_DIR/.env}"

APP_NAME="intel-candidate-app"
ALERT_ENV="${NANGMAN_ALERT_ENV:-dev}"
INCLUDE_SUCCESS="${INTEL_CANDIDATE_ALERT_INCLUDE_SUCCESS:-false}"
PIPELINE_ALERT_S3_BUCKET="${NANGMAN_PIPELINE_ALERT_S3_BUCKET:-${INTEL_CANDIDATE_PIPELINE_ALERT_S3_BUCKET:-}}"
PIPELINE_ALERT_S3_PREFIX="${NANGMAN_PIPELINE_ALERT_S3_PREFIX:-pipeline-alert-event/schema=pipeline_alert_event_v1}"

CLUSTER="${INTEL_CANDIDATE_ECS_CLUSTER:-ecs-nangman-dev-invest-apn2}"
SERVICE="${INTEL_CANDIDATE_ECS_SERVICE:-svc-nangman-dev-intel-candidate}"
AWS_REGION="${AWS_REGION:-ap-northeast-2}"
OUTPUT_BUCKET="${INTEL_CANDIDATE_OUTPUT_S3_BUCKET:-}"
ERROR_LOOKBACK_MINUTES="${INTEL_CANDIDATE_ALERT_ERROR_LOG_LOOKBACK_MINUTES:-30}"

# shellcheck source=scripts/lib/runtime-alert-core.sh
source "$SCRIPT_DIR/lib/runtime-alert-core.sh"
# shellcheck source=scripts/lib/runtime-alert-env.sh
source "$SCRIPT_DIR/lib/runtime-alert-env.sh"
# shellcheck source=scripts/lib/runtime-alert-pipeline.sh
source "$SCRIPT_DIR/lib/runtime-alert-pipeline.sh"
# shellcheck source=scripts/lib/runtime-alert-service.sh
source "$SCRIPT_DIR/lib/runtime-alert-service.sh"
# shellcheck source=scripts/lib/runtime-alert-message.sh
source "$SCRIPT_DIR/lib/runtime-alert-message.sh"
# shellcheck source=scripts/lib/runtime-alert-self-test.sh
source "$SCRIPT_DIR/lib/runtime-alert-self-test.sh"

main() {
  load_env_file
  apply_runtime_alert_env_defaults
  require_boolean "INTEL_CANDIDATE_ALERT_SELF_TEST" "${INTEL_CANDIDATE_ALERT_SELF_TEST:-false}"
  require_boolean "INTEL_CANDIDATE_ALERT_INCLUDE_SUCCESS" "$INCLUDE_SUCCESS"

  if is_true "${INTEL_CANDIDATE_ALERT_SELF_TEST:-false}"; then
    self_test
    return
  fi

  require_command aws
  require_command jq
  require_command sed
  require_command tail
  validate_runtime_alert_config

  local output_file
  output_file="$(mktemp)"
  set +e
  check_service "$output_file"
  local status=$?
  set -e

  if [[ "$status" -ne 0 ]]; then
    local alert_status=0
    send_pipeline_alert P1 "runtime check failed" "$(message P1 "runtime check failed" "$output_file" $'- ECS desired/running count와 최근 error log를 확인\n- structured pointer 입력과 candidate evidence bundle 최신성을 확인')" || alert_status=$?
    remove_temp_file "$output_file"
    if [[ "$alert_status" -ne 0 ]]; then
      return "$alert_status"
    fi
    return "$status"
  fi

  if is_true "$INCLUDE_SUCCESS"; then
    local alert_status=0
    send_pipeline_alert P3 "runtime check summary" "$(message P3 "runtime check summary" "$output_file" "- 일반 성공 알림은 기본적으로 끄고, 필요할 때만 일시적으로 켭니다.")" || alert_status=$?
    remove_temp_file "$output_file"
    return "$alert_status"
  fi
  remove_temp_file "$output_file"
}

main "$@"
