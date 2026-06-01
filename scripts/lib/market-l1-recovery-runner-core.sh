# shellcheck shell=bash

RECOVERY_RUNNER_CORE_LIB_DIR="${RECOVERY_RUNNER_CORE_LIB_DIR:-$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)}"

# shellcheck source=scripts/lib/path-safety.sh
source "$RECOVERY_RUNNER_CORE_LIB_DIR/path-safety.sh"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

require_absolute_file() {
  local name="$1"
  local path="$2"
  if [[ -z "$path" || "$path" != /* ]]; then
    echo "$name must be an absolute file path" >&2
    exit 1
  fi
  if [[ ! -f "$path" ]]; then
    echo "$name does not exist: $path" >&2
    exit 1
  fi
}

require_real_env() {
  local name="$1"
  local value="${!name:-}"
  if [[ -z "$value" || "$value" == *"<"* || "$value" == *">"* ]]; then
    echo "$name must be set to a real value before non-dry-run execution" >&2
    exit 1
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
    echo "$name must be a boolean value (true/false, yes/no, 1/0)" >&2
    exit 1
  fi
}

phase_array_name() {
  case "$1" in
    backfill) printf 'backfill_steps\n' ;;
    normalize) printf 'normalize_steps\n' ;;
    post_audit) printf 'post_audit_steps\n' ;;
    *)
      echo "unknown phase: $1" >&2
      exit 1
      ;;
  esac
}

replace_placeholder() {
  case "$1" in
    '${AWS_REGION}') printf '%s\n' "$AWS_REGION" ;;
    '${MARKET_L0_BUCKET}') printf '%s\n' "$MARKET_L0_BUCKET" ;;
    '${MARKET_L1_BUCKET}') printf '%s\n' "$MARKET_L1_BUCKET" ;;
    '${MARKET_L0_SPOOL_ROOT}') printf '%s\n' "$MARKET_L0_SPOOL_ROOT" ;;
    '${MARKET_L1_SPOOL_ROOT}') printf '%s\n' "$MARKET_L1_SPOOL_ROOT" ;;
    '${MARKET_NORMALIZE_CATCHUP_TMP_ROOT}') printf '%s\n' "$MARKET_NORMALIZE_CATCHUP_TMP_ROOT" ;;
    *) printf '%s\n' "$1" ;;
  esac
}
