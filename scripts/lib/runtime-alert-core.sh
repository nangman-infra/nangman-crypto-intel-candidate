# shellcheck shell=bash

log() {
  printf '%s\n' "$*"
}

die() {
  printf 'intel candidate runtime alert failed: %s\n' "$*" >&2
  exit 1
}

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    die "missing required command: $1"
  fi
}

is_true() {
  case "$1" in
    1 | true | TRUE | True | yes | YES | Yes) return 0 ;;
    *) return 1 ;;
  esac
}

is_boolean() {
  case "$1" in
    1 | 0 | true | false | TRUE | FALSE | True | False | yes | no | YES | NO | Yes | No) return 0 ;;
    *) return 1 ;;
  esac
}

require_boolean() {
  local name="$1"
  local value="$2"

  if ! is_boolean "$value"; then
    die "$name must be a boolean value (true/false, yes/no, 1/0)"
  fi
}

redact() {
  sed -E 's/[0-9]{12}/<aws-account-id>/g; s/[[:space:]]+$//'
}

append_check() {
  local file="$1"
  local title="$2"
  shift 2
  {
    printf '\n## %s\n' "$title"
    "$@" 2>&1 | redact
  } >> "$file"
}

remove_temp_file() {
  local file="${1:-}"
  if [[ -n "$file" ]]; then
    rm -f "$file"
  fi
}

require_non_negative_integer() {
  local name="$1"
  local value="$2"
  if [[ -z "$value" || ! "$value" =~ ^[0-9]+$ ]]; then
    die "$name must be a non-negative integer"
  fi
}

require_s3_bucket_name() {
  local name="$1"
  local value="$2"
  if [[ -z "$value" || "$value" == *"<"* || "$value" == *">"* ]]; then
    die "$name must be a real S3 bucket name"
  fi
  if (( ${#value} < 3 || ${#value} > 63 )); then
    die "$name must be 3 to 63 characters long"
  fi
  if [[ ! "$value" =~ ^[a-z0-9][a-z0-9.-]*[a-z0-9]$ ]]; then
    die "$name must start and end with a lowercase letter or number and contain only lowercase letters, numbers, periods, or hyphens"
  fi
  if [[ "$value" == *..* || "$value" == *.-* || "$value" == *-.* ]]; then
    die "$name must not contain adjacent periods or dashes next to periods"
  fi
  if [[ "$value" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    die "$name must not be formatted as an IP address"
  fi
  if [[ "$value" == xn--* || "$value" == sthree-* || "$value" == amzn-s3-demo-* ]]; then
    die "$name must not use an S3 reserved prefix"
  fi
  case "$value" in
    *-s3alias | *--ol-s3 | *.mrap | *--x-s3 | *--table-s3)
      die "$name must not use an S3 reserved suffix"
      ;;
  esac
}

require_s3_object_prefix() {
  local name="$1"
  local value="$2"
  local normalized
  if [[ -z "$value" ]]; then
    die "$name must not be empty"
  fi
  if [[ "$value" != "${value#"${value%%[![:space:]]*}"}" || "$value" != "${value%"${value##*[![:space:]]}"}" ]]; then
    die "$name must not include leading or trailing whitespace"
  fi
  case "$value" in
    /* | [sS]3://*)
      die "$name must be an object prefix, not a URI or absolute path"
      ;;
  esac
  if [[ "$value" == *"?"* || "$value" == *"#"* || "$value" == *"\\"* ]]; then
    die "$name must not include query markers, fragment markers, or backslashes"
  fi
  if [[ "$value" =~ [[:space:]] ]]; then
    die "$name must not contain whitespace"
  fi
  normalized="${value%/}"
  if [[ -z "$normalized" || "$normalized" == *"//"* || "$normalized" == "." || "$normalized" == ".." || "$normalized" == */./* || "$normalized" == */../* || "$normalized" == ./* || "$normalized" == ../* || "$normalized" == */. || "$normalized" == */.. ]]; then
    die "$name must not contain empty or period-only path segments"
  fi
}

require_alert_priority() {
  local name="$1"
  local value="$2"
  if [[ ! "$value" =~ ^P[0-9]+$ ]]; then
    die "$name must use P<number> format"
  fi
}

validate_runtime_alert_config() {
  require_non_negative_integer \
    "INTEL_CANDIDATE_ALERT_ERROR_LOG_LOOKBACK_MINUTES" \
    "$ERROR_LOOKBACK_MINUTES"
  require_s3_object_prefix \
    "NANGMAN_PIPELINE_ALERT_S3_PREFIX or INTEL_CANDIDATE_PIPELINE_ALERT_S3_PREFIX" \
    "$PIPELINE_ALERT_S3_PREFIX"
  if [[ -n "$PIPELINE_ALERT_S3_BUCKET" ]]; then
    require_s3_bucket_name \
      "NANGMAN_PIPELINE_ALERT_S3_BUCKET or INTEL_CANDIDATE_PIPELINE_ALERT_S3_BUCKET" \
      "$PIPELINE_ALERT_S3_BUCKET"
  fi
  if [[ -n "$OUTPUT_BUCKET" ]]; then
    require_s3_bucket_name \
      "INTEL_CANDIDATE_OUTPUT_S3_BUCKET" \
      "$OUTPUT_BUCKET"
  fi
}
