# shellcheck shell=bash

load_env_file() {
  if [[ ! -f "$ENV_FILE" ]]; then
    return
  fi

  local line
  local key
  local value
  while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line#"${line%%[![:space:]]*}"}"
    line="${line%"${line##*[![:space:]]}"}"
    if [[ -z "$line" || "$line" == \#* ]]; then
      continue
    fi
    if [[ "$line" == export[[:space:]]* ]]; then
      line="${line#export}"
      line="${line#"${line%%[![:space:]]*}"}"
    fi
    if [[ "$line" != *=* ]]; then
      die "invalid env file line in $ENV_FILE: expected KEY=VALUE"
    fi
    key="${line%%=*}"
    value="${line#*=}"
    key="${key%"${key##*[![:space:]]}"}"
    value="${value#"${value%%[![:space:]]*}"}"
    if [[ ! "$key" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]]; then
      die "invalid env file key in $ENV_FILE: $key"
    fi
    if [[ "$value" == \"*\" && "$value" == *\" && "${#value}" -ge 2 ]]; then
      value="${value:1:${#value}-2}"
    elif [[ "$value" == \'*\' && "$value" == *\' && "${#value}" -ge 2 ]]; then
      value="${value:1:${#value}-2}"
    fi
    export "$key=$value"
  done < "$ENV_FILE"
}

apply_runtime_alert_env_defaults() {
  ALERT_ENV="${NANGMAN_ALERT_ENV:-dev}"
  INCLUDE_SUCCESS="${INTEL_CANDIDATE_ALERT_INCLUDE_SUCCESS:-false}"
  PIPELINE_ALERT_S3_BUCKET="${NANGMAN_PIPELINE_ALERT_S3_BUCKET:-${INTEL_CANDIDATE_PIPELINE_ALERT_S3_BUCKET:-}}"
  PIPELINE_ALERT_S3_PREFIX="${NANGMAN_PIPELINE_ALERT_S3_PREFIX:-pipeline-alert-event/schema=pipeline_alert_event_v1}"
  CLUSTER="${INTEL_CANDIDATE_ECS_CLUSTER:-ecs-nangman-dev-invest-apn2}"
  SERVICE="${INTEL_CANDIDATE_ECS_SERVICE:-svc-nangman-dev-intel-candidate}"
  AWS_REGION="${AWS_REGION:-ap-northeast-2}"
  OUTPUT_BUCKET="${INTEL_CANDIDATE_OUTPUT_S3_BUCKET:-}"
  ERROR_LOOKBACK_MINUTES="${INTEL_CANDIDATE_ALERT_ERROR_LOG_LOOKBACK_MINUTES:-30}"
}
