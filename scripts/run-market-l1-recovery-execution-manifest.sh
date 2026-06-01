#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
# shellcheck source=scripts/lib/market-l1-recovery-runner.sh
source "$SCRIPT_DIR/lib/market-l1-recovery-runner.sh"

MANIFEST_FILE="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_EXECUTION_MANIFEST_FILE:-${1:-}}"
OUTPUT_DIR="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_RUN_OUTPUT_DIR:-${2:-}}"
APPROVAL="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_APPROVAL:-}"
DRY_RUN="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_DRY_RUN:-false}"
RESUME="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_RESUME:-true}"
PHASES_CSV="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_PHASES:-backfill,normalize,post_audit}"

validate_recovery_runner_inputs
prepare_recovery_runner_output
run_recovery_manifest_steps

emit_event "run_completed" "$(jq -nc '{status:"success"}')"
write_summary "success"
print_recovery_run_summary
