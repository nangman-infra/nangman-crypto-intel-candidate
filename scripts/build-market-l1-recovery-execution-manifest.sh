#!/usr/bin/env bash
set -euo pipefail

RECOVERY_PLAN_FILE="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_PLAN_FILE:-${1:-}}"
OUTPUT_FILE="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_EXECUTION_MANIFEST_OUTPUT:-${2:-}}"
READINESS_FILE="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_READINESS_FILE:-${3:-}}"

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

require_absolute_output_path() {
  local name="$1"
  local path="$2"
  if [[ -z "$path" ]]; then
    return
  fi
  if [[ "$path" != /* ]]; then
    echo "$name must be an absolute path; got $path" >&2
    exit 1
  fi
}

require_command date
require_command jq
require_command mktemp

require_absolute_file "INTEL_CANDIDATE_MARKET_L1_RECOVERY_PLAN_FILE or first argument" "$RECOVERY_PLAN_FILE"
require_absolute_output_path "INTEL_CANDIDATE_MARKET_L1_RECOVERY_EXECUTION_MANIFEST_OUTPUT or second argument" "$OUTPUT_FILE"
if [[ -n "$READINESS_FILE" ]]; then
  require_absolute_file "INTEL_CANDIDATE_MARKET_L1_RECOVERY_READINESS_FILE or third argument" "$READINESS_FILE"
fi

jq -e '.schema_version == "candidate_market_l1_recovery_plan_v1"' "$RECOVERY_PLAN_FILE" >/dev/null || {
  echo "recovery plan must have schema_version=candidate_market_l1_recovery_plan_v1" >&2
  exit 1
}

if [[ -n "$READINESS_FILE" ]]; then
  jq -e '.schema_version == "candidate_market_l1_recovery_readiness_v1"' "$READINESS_FILE" >/dev/null || {
    echo "readiness file must have schema_version=candidate_market_l1_recovery_readiness_v1" >&2
    exit 1
  }
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

readiness_json="$tmp_dir/readiness.json"
if [[ -n "$READINESS_FILE" ]]; then
  cp "$READINESS_FILE" "$readiness_json"
else
  printf 'null\n' > "$readiness_json"
fi

tmp_output="$tmp_dir/market-l1-recovery-execution-manifest.json"

jq -n \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg recovery_plan_file "$RECOVERY_PLAN_FILE" \
  --arg readiness_file "$READINESS_FILE" \
  --slurpfile plan "$RECOVERY_PLAN_FILE" \
  --slurpfile readiness "$readiness_json" \
  '
    def iso_ms:
      if . == null then null
      else ((. / 1000) | strftime("%Y-%m-%dT%H:%M:%SZ"))
      end;

    def merge_ranges:
      sort_by(.start_ms, .end_ms)
      | reduce .[] as $range ([];
          if length == 0 then
            [$range]
          else
            .[-1] as $last
            | if $range.start_ms <= $last.end_ms then
                .[:-1] + [
                  $last + {
                    end_ms:([$last.end_ms, $range.end_ms] | max),
                    source_window_count:(($last.source_window_count // 1) + ($range.source_window_count // 1)),
                    packet_ids:((($last.packet_ids // []) + ($range.packet_ids // [])) | unique),
                    symbols:((($last.symbols // []) + ($range.symbols // [])) | unique),
                    market_symbols:((($last.market_symbols // []) + ($range.market_symbols // [])) | unique)
                  }
                ]
              else
                . + [$range]
              end
          end
        );

    def backfill_args($root; $venue; $symbol; $start_ms; $end_ms):
      [
        "cargo",
        "run",
        "--manifest-path",
        "\($root)/Cargo.toml",
        "--bin",
        "market-backfill",
        "--",
        "--venue",
        $venue,
        "--config",
        "\($root)/config",
        "--input-start-ms",
        ($start_ms | tostring),
        "--input-end-ms",
        ($end_ms | tostring),
        "--symbols",
        $symbol,
        "--l0-s3-bucket",
        "${MARKET_L0_BUCKET}",
        "--l0-spool-root",
        "${MARKET_L0_SPOOL_ROOT}",
        "--disable-s3-retention",
        "--aws-region",
        "${AWS_REGION}"
      ];

    def normalize_args($root; $start_ms; $end_ms; $window_ms; $schedule_interval_ms):
      [
        "cargo",
        "run",
        "--manifest-path",
        "\($root)/Cargo.toml",
        "--bin",
        "market-normalize",
        "--",
        "--l0-s3-bucket",
        "${MARKET_L0_BUCKET}",
        "--l0-local-root",
        "${MARKET_L0_SPOOL_ROOT}",
        "--l1-s3-bucket",
        "${MARKET_L1_BUCKET}",
        "--spool-root",
        "${MARKET_L1_SPOOL_ROOT}",
        "--catchup-tmp-root",
        "${MARKET_NORMALIZE_CATCHUP_TMP_ROOT}",
        "--input-start-ms",
        ($start_ms | tostring),
        "--input-end-ms",
        ($end_ms | tostring),
        "--window-ms",
        ($window_ms | tostring),
        "--schedule-interval-ms",
        ($schedule_interval_ms | tostring),
        "--disable-s3-retention",
        "--aws-region",
        "${AWS_REGION}"
      ];

    def audit_args($root; $start_ms; $end_ms; $window_ms):
      [
        "cargo",
        "run",
        "--manifest-path",
        "\($root)/Cargo.toml",
        "--bin",
        "market-normalize",
        "--",
        "--l0-s3-bucket",
        "${MARKET_L0_BUCKET}",
        "--l1-s3-bucket",
        "${MARKET_L1_BUCKET}",
        "--audit-l1-index-start-ms",
        ($start_ms | tostring),
        "--audit-l1-index-end-ms",
        ($end_ms | tostring),
        "--window-ms",
        ($window_ms | tostring),
        "--disable-s3-retention",
        "--aws-region",
        "${AWS_REGION}"
      ];

    ($plan[0]) as $plan
    | ($readiness[0]) as $readiness
    | ($plan.input.market_ingest_app_root) as $root
    | ($plan.input.market_venue) as $venue
    | ($plan.input.market_window_ms) as $window_ms
    | ($plan.input.normalize_schedule_interval_ms) as $schedule_interval_ms
    | [
        $plan.symbols[]?
        | .symbol as $symbol
        | .market_symbol as $market_symbol
        | .recovery_windows[]?
        | {
            symbol:$symbol,
            market_symbol:$market_symbol,
            start_ms:.recovery_input_start_ms,
            end_ms:.recovery_input_end_ms,
            packet_ids:[.packet_id],
            symbols:[$symbol],
            market_symbols:[$market_symbol],
            source_window_count:1
          }
      ] as $flat_windows
    | ($flat_windows
        | group_by(.market_symbol)
        | map({
            symbol:.[0].symbol,
            market_symbol:.[0].market_symbol,
            ranges:(map({
              start_ms,
              end_ms,
              packet_ids,
              symbols,
              market_symbols,
              source_window_count
            }) | merge_ranges)
          })
      ) as $by_market_symbol
    | ($by_market_symbol
        | map(
            . as $group
            | $group.ranges[]
            | {
                symbol:$group.symbol,
                market_symbol:$group.market_symbol,
                start_ms,
                start_at:(.start_ms | iso_ms),
                end_ms,
                end_at:(.end_ms | iso_ms),
                source_window_count,
                packet_ids,
                command_args:backfill_args($root; $venue; $group.market_symbol; .start_ms; .end_ms),
                writes_s3:true,
                target:"market_l0"
              }
          )
      ) as $backfill_steps
    | ($flat_windows
        | map({
            start_ms,
            end_ms,
            packet_ids,
            symbols,
            market_symbols,
            source_window_count
          })
        | merge_ranges
        | map({
            start_ms,
            start_at:(.start_ms | iso_ms),
            end_ms,
            end_at:(.end_ms | iso_ms),
            source_window_count,
            packet_ids,
            symbols,
            market_symbols,
            command_args:normalize_args($root; .start_ms; .end_ms; $window_ms; $schedule_interval_ms),
            writes_s3:true,
            target:"market_l1"
          })
      ) as $normalize_steps
    | ($normalize_steps
        | map({
            start_ms,
            start_at,
            end_ms,
            end_at,
            source_window_count,
            packet_ids,
            symbols,
            market_symbols,
            command_args:audit_args($root; .start_ms; .end_ms; $window_ms),
            reads_s3:true,
            writes_s3:false,
            target:"market_l1_index"
          })
      ) as $post_audit_steps
    | {
        schema_version:"candidate_market_l1_recovery_execution_manifest_v1",
        generated_at:$generated_at,
        input:{
          recovery_plan_file:$recovery_plan_file,
          readiness_file:(if ($readiness_file | length) == 0 then null else $readiness_file end),
          market_ingest_app_root:$root,
          market_venue:$venue,
          market_window_ms:$window_ms,
          normalize_schedule_interval_ms:$schedule_interval_ms
        },
        safety:{
          local_manifest_only:true,
          s3_read_performed:false,
          s3_write_performed:false,
          ecs_task_started:false,
          dispatcher_mode_changed:false,
          research_run_task_started:false,
          shadow_paper_live_enabled:false,
          contains_real_bucket_values:false,
          contains_aws_profile_value:false,
          execution_requires_explicit_operator_approval:true
        },
        readiness_gate:{
          readiness_file_present:($readiness != null),
          readiness_verdict:($readiness.verdict // null),
          l1_index_audit_completed:($readiness.current_state.l1_index_audit_completed // false),
          exchange_symbol_check_completed:($readiness.current_state.exchange_symbol_check_completed // false),
          missing_index_pointer_count_total:($readiness.evidence.audit_missing_index_pointer_count_total // null),
          exchangeinfo_missing_symbol_count:($readiness.evidence.exchangeinfo_missing_symbol_count // null),
          exchangeinfo_non_trading_symbol_count:($readiness.evidence.exchangeinfo_non_trading_symbol_count // null),
          promotion_passed:($readiness.current_state.promotion_passed // false),
          shadow_created:($readiness.current_state.shadow_created // false),
          paper_created:($readiness.current_state.paper_created // false),
          live_enabled:($readiness.current_state.live_enabled // false)
        },
        required_environment:[
          "AWS_PROFILE",
          "AWS_REGION",
          "MARKET_L0_BUCKET",
          "MARKET_L1_BUCKET",
          "MARKET_L0_SPOOL_ROOT",
          "MARKET_L1_SPOOL_ROOT",
          "MARKET_NORMALIZE_CATCHUP_TMP_ROOT"
        ],
        approval_boundary:{
          required:true,
          approval_phrase:"approve_market_l1_s3_write_recovery",
          reason:"market-backfill writes Market-L0 and market-normalize writes Market-L1 artifacts",
          blocked_until_approved:[
            "do_not_run_backfill_steps",
            "do_not_run_normalize_steps",
            "do_not_switch_dispatcher_out_of_dry_run",
            "do_not_create_shadow",
            "do_not_create_paper",
            "do_not_enable_live"
          ]
        },
        summary:{
          source_recovery_window_count:($flat_windows | length),
          backfill_step_count:($backfill_steps | length),
          normalize_step_count:($normalize_steps | length),
          post_audit_step_count:($post_audit_steps | length),
          historical_symbol_count:($plan.summary.historical_symbol_count // null),
          market_symbol_count:($by_market_symbol | length),
          market_symbols:($by_market_symbol | map(.market_symbol)),
          recovery_input_start_ms:($plan.summary.recovery_input_start_ms // null),
          recovery_input_start_at:($plan.summary.recovery_input_start_at // null),
          recovery_input_end_ms:($plan.summary.recovery_input_end_ms // null),
          recovery_input_end_at:($plan.summary.recovery_input_end_at // null)
        },
        execution_order:[
          "export required environment variables locally",
          "rerun read-only audit if recovery plan is stale",
          "run backfill_steps only after explicit operator approval",
          "run normalize_steps only after backfill completes",
          "run post_audit_steps and require zero missing L1 index pointers",
          "rerun candidate source-gap diagnosis v2",
          "rebuild current-approved research batch manifest",
          "rerun research replay",
          "keep dispatcher/shadow/paper/live closed unless research gate later passes"
        ],
        backfill_steps:$backfill_steps,
        normalize_steps:$normalize_steps,
        post_audit_steps:$post_audit_steps,
        post_repair_checks:[
          "candidate_source_gap_diagnosis_v2",
          "current_approved_research_batch_manifest",
          "research_replay",
          "promotion_gate",
          "shadow_paper_live_boundary"
        ]
      }
  ' > "$tmp_output"

if [[ -n "$OUTPUT_FILE" ]]; then
  cp "$tmp_output" "$OUTPUT_FILE"
  {
    echo "market_l1_recovery_execution_manifest_output=$OUTPUT_FILE"
    jq -r '
      "source_recovery_window_count=\(.summary.source_recovery_window_count)",
      "backfill_step_count=\(.summary.backfill_step_count)",
      "normalize_step_count=\(.summary.normalize_step_count)",
      "post_audit_step_count=\(.summary.post_audit_step_count)",
      "approval_required=\(.approval_boundary.required)"
    ' "$OUTPUT_FILE"
  } >&2
else
  cat "$tmp_output"
fi
