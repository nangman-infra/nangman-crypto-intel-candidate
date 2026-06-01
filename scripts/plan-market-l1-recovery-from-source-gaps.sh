#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
# shellcheck source=scripts/lib/market-l1-recovery-plan-validation.sh
source "$SCRIPT_DIR/lib/market-l1-recovery-plan-validation.sh"
# shellcheck source=scripts/lib/market-l1-recovery-plan-symbol-map.sh
source "$SCRIPT_DIR/lib/market-l1-recovery-plan-symbol-map.sh"
# shellcheck source=scripts/lib/market-l1-recovery-plan-output.sh
source "$SCRIPT_DIR/lib/market-l1-recovery-plan-output.sh"

SOURCE_GAP_FILE="${INTEL_CANDIDATE_SOURCE_GAP_FILE:-${1:-}}"
OUTPUT_FILE="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_PLAN_OUTPUT:-${2:-}}"

MARKET_INGEST_APP_ROOT="${INTEL_CANDIDATE_MARKET_INGEST_APP_ROOT:-/Volumes/WD/Developments/nangman-crypto/apps/market-ingest-app}"
MARKET_VENUE="${INTEL_CANDIDATE_MARKET_VENUE:-binance}"
MARKET_QUOTE_SUFFIX="${INTEL_CANDIDATE_MARKET_QUOTE_SUFFIX:-USDT}"
MARKET_SYMBOL_MAP_FILE="${INTEL_CANDIDATE_MARKET_SYMBOL_MAP_FILE:-}"
RECOVERY_MARGIN_MS="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_MARGIN_MS:-3600000}"
MARKET_WINDOW_MS="${INTEL_CANDIDATE_MARKET_L1_WINDOW_MS:-1000}"
NORMALIZE_SCHEDULE_INTERVAL_MS="${INTEL_CANDIDATE_MARKET_L1_NORMALIZE_SCHEDULE_INTERVAL_MS:-900000}"

validate_market_l1_recovery_plan_inputs
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

symbol_map_json="$tmp_dir/symbol-map.json"
prepare_market_l1_recovery_symbol_map
assert_market_l1_recovery_source_gap_schema
tmp_output="$tmp_dir/market-l1-recovery-plan.json"

write_market_l1_recovery_plan
emit_market_l1_recovery_plan_result
