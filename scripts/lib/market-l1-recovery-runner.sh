# shellcheck shell=bash

RECOVERY_RUNNER_LIB_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

# shellcheck source=scripts/lib/market-l1-recovery-runner-core.sh
source "$RECOVERY_RUNNER_LIB_DIR/market-l1-recovery-runner-core.sh"
# shellcheck source=scripts/lib/market-l1-recovery-runner-events.sh
source "$RECOVERY_RUNNER_LIB_DIR/market-l1-recovery-runner-events.sh"
# shellcheck source=scripts/lib/market-l1-recovery-runner-validation.sh
source "$RECOVERY_RUNNER_LIB_DIR/market-l1-recovery-runner-validation.sh"
# shellcheck source=scripts/lib/market-l1-recovery-runner-step-execution.sh
source "$RECOVERY_RUNNER_LIB_DIR/market-l1-recovery-runner-step-execution.sh"
# shellcheck source=scripts/lib/market-l1-recovery-runner-steps.sh
source "$RECOVERY_RUNNER_LIB_DIR/market-l1-recovery-runner-steps.sh"
