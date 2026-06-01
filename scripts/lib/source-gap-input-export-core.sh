#!/usr/bin/env bash

SOURCE_GAP_INPUT_EXPORT_CORE_LIB_DIR="${SOURCE_GAP_INPUT_EXPORT_CORE_LIB_DIR:-$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)}"

# shellcheck source=scripts/lib/path-safety.sh
source "$SOURCE_GAP_INPUT_EXPORT_CORE_LIB_DIR/path-safety.sh"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
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

require_true_false() {
  local name="$1"
  local value="$2"
  case "$value" in
    true | false) return 0 ;;
    *)
      echo "$name must be true or false" >&2
      exit 1
      ;;
  esac
}

require_s3_bucket_name() {
  local name="$1"
  local value="$2"
  if [[ -z "$value" || "$value" == "None" || "$value" == *"<"* || "$value" == *">"* ]]; then
    echo "$name must be a real S3 bucket name" >&2
    exit 1
  fi
  if (( ${#value} < 3 || ${#value} > 63 )); then
    echo "$name must be 3 to 63 characters long" >&2
    exit 1
  fi
  if [[ ! "$value" =~ ^[a-z0-9][a-z0-9.-]*[a-z0-9]$ ]]; then
    echo "$name must start and end with a lowercase letter or number and contain only lowercase letters, numbers, periods, or hyphens" >&2
    exit 1
  fi
  if [[ "$value" == *..* || "$value" == *.-* || "$value" == *-.* ]]; then
    echo "$name must not contain adjacent periods or dashes next to periods" >&2
    exit 1
  fi
  if [[ "$value" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "$name must not be formatted as an IP address" >&2
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

require_source_dt() {
  if [[ ! "$SOURCE_DT" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]; then
    echo "INTEL_CANDIDATE_SOURCE_GAP_DT must use YYYY-MM-DD UTC date format" >&2
    exit 1
  fi
}

require_priority_segment() {
  local priority="$1"
  if [[ -z "$priority" || ! "$priority" =~ ^[A-Za-z0-9._-]+$ || "$priority" == "." || "$priority" == ".." ]]; then
    echo "INTEL_CANDIDATE_SOURCE_GAP_PRIORITIES contains an unsafe priority segment: $priority" >&2
    exit 1
  fi
}

validate_source_gap_export_inputs() {
  require_true_false \
    "INTEL_CANDIDATE_SOURCE_GAP_ALLOW_EXISTING" \
    "$ALLOW_EXISTING"
  require_source_dt
  printf '%s\n' "$PRIORITIES" | tr ',' '\n' | while IFS= read -r priority; do
    [[ -z "$priority" ]] && continue
    require_priority_segment "$priority"
  done
}
