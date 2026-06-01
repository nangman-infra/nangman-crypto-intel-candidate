#!/usr/bin/env bash

write_market_l1_recovery_execution_manifest() {
  local output_path="$1"
  local readiness_json="$2"
  local jq_dir="${SCRIPT_DIR}/jq"

  jq -L "$jq_dir" -n \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg recovery_plan_file "$RECOVERY_PLAN_FILE" \
    --arg readiness_file "$READINESS_FILE" \
    --slurpfile plan "$RECOVERY_PLAN_FILE" \
    --slurpfile readiness "$readiness_json" \
    -f "$jq_dir/market-l1-recovery-execution.jq" \
    > "$output_path"
}
