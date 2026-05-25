#!/usr/bin/env bash
set -euo pipefail

SOURCE_GAP_FILE="${INTEL_CANDIDATE_SOURCE_GAP_FILE:-${1:-}}"
OUTPUT_FILE="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_PLAN_OUTPUT:-${2:-}}"

MARKET_INGEST_APP_ROOT="${INTEL_CANDIDATE_MARKET_INGEST_APP_ROOT:-/Volumes/WD/Developments/nangman-crypto/apps/market-ingest-app}"
MARKET_VENUE="${INTEL_CANDIDATE_MARKET_VENUE:-binance}"
MARKET_QUOTE_SUFFIX="${INTEL_CANDIDATE_MARKET_QUOTE_SUFFIX:-USDT}"
MARKET_SYMBOL_MAP_FILE="${INTEL_CANDIDATE_MARKET_SYMBOL_MAP_FILE:-}"
RECOVERY_MARGIN_MS="${INTEL_CANDIDATE_MARKET_L1_RECOVERY_MARGIN_MS:-3600000}"
MARKET_WINDOW_MS="${INTEL_CANDIDATE_MARKET_L1_WINDOW_MS:-1000}"
NORMALIZE_SCHEDULE_INTERVAL_MS="${INTEL_CANDIDATE_MARKET_L1_NORMALIZE_SCHEDULE_INTERVAL_MS:-900000}"

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

require_positive_integer() {
  local name="$1"
  local value="$2"
  if ! [[ "$value" =~ ^[0-9]+$ ]] || [[ "$value" == "0" ]]; then
    echo "$name must be a positive integer; got $value" >&2
    exit 1
  fi
}

require_command date
require_command awk
require_command jq
require_command mktemp
require_absolute_file "INTEL_CANDIDATE_SOURCE_GAP_FILE or first argument" "$SOURCE_GAP_FILE"
require_absolute_output_path "INTEL_CANDIDATE_MARKET_L1_RECOVERY_PLAN_OUTPUT or second argument" "$OUTPUT_FILE"
require_positive_integer "INTEL_CANDIDATE_MARKET_L1_RECOVERY_MARGIN_MS" "$RECOVERY_MARGIN_MS"
require_positive_integer "INTEL_CANDIDATE_MARKET_L1_WINDOW_MS" "$MARKET_WINDOW_MS"
require_positive_integer "INTEL_CANDIDATE_MARKET_L1_NORMALIZE_SCHEDULE_INTERVAL_MS" "$NORMALIZE_SCHEDULE_INTERVAL_MS"

if [[ "$MARKET_INGEST_APP_ROOT" != /* ]]; then
  echo "INTEL_CANDIDATE_MARKET_INGEST_APP_ROOT must be an absolute path" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

symbol_map_json="$tmp_dir/symbol-map.json"
symbol_map_source="derived_quote_suffix"
if [[ -n "$MARKET_SYMBOL_MAP_FILE" ]]; then
  require_absolute_file "INTEL_CANDIDATE_MARKET_SYMBOL_MAP_FILE" "$MARKET_SYMBOL_MAP_FILE"
  cp "$MARKET_SYMBOL_MAP_FILE" "$symbol_map_json"
  symbol_map_source="explicit_symbol_map_file"
elif [[ -f "$MARKET_INGEST_APP_ROOT/config/universe.major-50.toml" ]]; then
  awk '
    function emit() {
      if (enabled == "true" && base != "" && raw != "") {
        print base "\t" raw
      }
    }
    /^\[\[symbols\]\]/ {
      emit()
      base = ""
      raw = ""
      enabled = ""
      next
    }
    /^[[:space:]]*base[[:space:]]*=/ {
      base = $0
      sub(/^[^"]*"/, "", base)
      sub(/".*$/, "", base)
      next
    }
    /^[[:space:]]*raw[[:space:]]*=/ {
      raw = $0
      sub(/^[^"]*"/, "", raw)
      sub(/".*$/, "", raw)
      next
    }
    /^[[:space:]]*enabled[[:space:]]*=/ {
      enabled = $0
      sub(/.*=[[:space:]]*/, "", enabled)
      gsub(/[[:space:]]/, "", enabled)
      next
    }
    END {
      emit()
    }
  ' "$MARKET_INGEST_APP_ROOT/config/universe.major-50.toml" \
    | jq -Rn '
        reduce inputs as $line ({};
          ($line | split("\t")) as $parts
          | if ($parts | length) == 2 then . + {($parts[0]): $parts[1]} else . end
        )
      ' > "$symbol_map_json"
  symbol_map_source="market_ingest_major50_config"
else
  printf '{}\n' > "$symbol_map_json"
fi

jq -e '.schema_version == "intel_candidate_source_gap_diagnosis_v2"' "$SOURCE_GAP_FILE" >/dev/null || {
  echo "source gap diagnosis must have schema_version=intel_candidate_source_gap_diagnosis_v2" >&2
  exit 1
}

tmp_output="$tmp_dir/market-l1-recovery-plan.json"

jq -n \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg source_gap_file "$SOURCE_GAP_FILE" \
  --arg market_ingest_app_root "$MARKET_INGEST_APP_ROOT" \
  --arg market_venue "$MARKET_VENUE" \
  --arg market_quote_suffix "$MARKET_QUOTE_SUFFIX" \
  --arg market_symbol_map_file "$MARKET_SYMBOL_MAP_FILE" \
  --arg symbol_map_source "$symbol_map_source" \
  --argjson recovery_margin_ms "$RECOVERY_MARGIN_MS" \
  --argjson market_window_ms "$MARKET_WINDOW_MS" \
  --argjson normalize_schedule_interval_ms "$NORMALIZE_SCHEDULE_INTERVAL_MS" \
  --slurpfile source "$SOURCE_GAP_FILE" \
  --slurpfile symbol_map "$symbol_map_json" \
  '
    def iso_ms:
      if . == null then null
      else ((. / 1000) | strftime("%Y-%m-%dT%H:%M:%SZ"))
      end;

    def symbol_map_object:
      ($symbol_map[0].symbols? // $symbol_map[0] // {});

    def explicit_market_symbol($symbol):
      symbol_map_object[$symbol]?;

    def derived_market_symbol($symbol):
      if ($market_quote_suffix | length) == 0 then $symbol
      else "\($symbol)\($market_quote_suffix)"
      end;

    def market_symbol_for($symbol):
      explicit_market_symbol($symbol) // derived_market_symbol($symbol);

    def mapping_status_for($symbol):
      if explicit_market_symbol($symbol) != null then $symbol_map_source
      else "derived_quote_suffix_unverified"
      end;

    def clamp_positive:
      if . < 1 then 1 else . end;

    def align_floor($interval):
      if $interval <= 0 then .
      else (. - (. % $interval))
      end;

    def align_ceil($interval):
      if $interval <= 0 then .
      else
        (. % $interval) as $remainder
        | if $remainder == 0 then . else (. + ($interval - $remainder)) end
      end;

    def market_backfill_args($symbol; $start_ms; $end_ms):
      [
        "cargo",
        "run",
        "--manifest-path",
        "\($market_ingest_app_root)/Cargo.toml",
        "--bin",
        "market-backfill",
        "--",
        "--venue",
        $market_venue,
        "--config",
        "\($market_ingest_app_root)/config",
        "--input-start-ms",
        ($start_ms | tostring),
        "--input-end-ms",
        ($end_ms | tostring),
        "--symbols",
        market_symbol_for($symbol),
        "--l0-s3-bucket",
        "<market-l0-bucket>",
        "--aws-region",
        "<aws-region>"
      ];

    def market_normalize_args($start_ms; $end_ms):
      [
        "cargo",
        "run",
        "--manifest-path",
        "\($market_ingest_app_root)/Cargo.toml",
        "--bin",
        "market-normalize",
        "--",
        "--l0-s3-bucket",
        "<market-l0-bucket>",
        "--l0-local-root",
        "/opt/nangman-crypto/data/spool/market-ingest/l0",
        "--l1-s3-bucket",
        "<market-l1-bucket>",
        "--catchup-tmp-root",
        "/opt/nangman-crypto/data/spool/market-normalize/catchup",
        "--input-start-ms",
        ($start_ms | tostring),
        "--input-end-ms",
        ($end_ms | tostring),
        "--window-ms",
        ($market_window_ms | tostring),
        "--schedule-interval-ms",
        ($normalize_schedule_interval_ms | tostring),
        "--aws-region",
        "<aws-region>"
      ];

    def market_l1_index_audit_args($start_ms; $end_ms):
      [
        "cargo",
        "run",
        "--manifest-path",
        "\($market_ingest_app_root)/Cargo.toml",
        "--bin",
        "market-normalize",
        "--",
        "--l0-s3-bucket",
        "<market-l0-bucket>",
        "--l1-s3-bucket",
        "<market-l1-bucket>",
        "--audit-l1-index-start-ms",
        ($start_ms | tostring),
        "--audit-l1-index-end-ms",
        ($end_ms | tostring),
        "--window-ms",
        ($market_window_ms | tostring),
        "--aws-region",
        "<aws-region>"
      ];

    ($source[0]) as $gap
    | [
        $gap.symbols[]?
        | ([
            ((.market_context_gap.historical_terminal_missing_context_records // [])[]
              | . + {recovery_reason:"terminal_missing_market_context"}),
            ((.market_context_gap.current_or_unknown_terminal_missing_context_records // [])[]
              | . + {recovery_reason:"terminal_missing_market_context"}),
            ((.market_context_gap.historical_pending_context_records // [])[]
              | . + {recovery_reason:"pending_market_context_materialization"}),
            ((.market_context_gap.current_or_unknown_pending_context_records // [])[]
              | . + {recovery_reason:"pending_market_context_materialization"})
          ] | map(select(.event_basis_ms != null))) as $context_records
        | select(($context_records | length) > 0)
        | (.symbol) as $symbol
        | ($context_records | map(.event_basis_ms) | min) as $min_ms
        | ($context_records | map(.event_basis_ms) | max) as $max_ms
        | (($min_ms - $recovery_margin_ms) | clamp_positive) as $recovery_start_ms
        | (($max_ms + $recovery_margin_ms) | clamp_positive) as $recovery_end_ms
        | ([
            $context_records[]
            | (.event_basis_ms // null) as $event_ms
            | (($event_ms - $recovery_margin_ms) | clamp_positive | align_floor($normalize_schedule_interval_ms)) as $window_start_ms
            | (($event_ms + $recovery_margin_ms) | clamp_positive | align_ceil($normalize_schedule_interval_ms)) as $window_end_ms
            | {
                packet_id:.packet_id,
                artifact_family:(.artifact_family // null),
                recovery_reason:(.recovery_reason // null),
                market_context_status:(.market_context_status // null),
                symbols:(.symbols // []),
                event_basis_ms:$event_ms,
                event_basis_at:($event_ms | iso_ms),
                recovery_input_start_ms:$window_start_ms,
                recovery_input_start_at:($window_start_ms | iso_ms),
                recovery_input_end_ms:$window_end_ms,
                recovery_input_end_at:($window_end_ms | iso_ms),
                market_backfill_args:market_backfill_args($symbol; $window_start_ms; $window_end_ms),
                market_normalize_args:market_normalize_args($window_start_ms; $window_end_ms),
                market_l1_index_audit_args:market_l1_index_audit_args($window_start_ms; $window_end_ms)
              }
          ]) as $recovery_windows
        | ($recovery_windows | map(.recovery_input_start_ms) | min) as $recovery_start_ms
        | ($recovery_windows | map(.recovery_input_end_ms) | max) as $recovery_end_ms
        | {
            symbol:.symbol,
            recovery_class:(
              if (.market_context_gap.pending_context_packets // 0) > 0
              then "pending_market_context_materialization"
              elif .market_context_gap.historical_backfill_required // false
              then "full_historical_backfill_required"
              elif (.market_context_gap.historical_terminal_missing_context_present // false)
                and (.market_context_gap.current_or_unknown_terminal_missing_context_present // false)
              then "mixed_terminal_context_recovery"
              elif .market_context_gap.historical_terminal_missing_context_present // false
              then "historical_terminal_context_recovery"
              else "current_or_unknown_terminal_context_recovery"
              end
            ),
            primary_blocker:.primary_blocker,
            status:.status,
            terminal_missing_context_packets:(.market_context_gap.terminal_missing_context_packets // 0),
            terminal_missing_before_observed_context_floor:(.market_context_gap.terminal_missing_before_observed_context_floor // 0),
            terminal_missing_at_or_after_observed_context_floor:(.market_context_gap.terminal_missing_at_or_after_observed_context_floor // 0),
            terminal_missing_unknown_event_basis:(.market_context_gap.terminal_missing_unknown_event_basis // 0),
            pending_context_packets:(.market_context_gap.pending_context_packets // 0),
            pending_before_observed_context_floor:(.market_context_gap.pending_before_observed_context_floor // 0),
            pending_at_or_after_observed_context_floor:(.market_context_gap.pending_at_or_after_observed_context_floor // 0),
            pending_unknown_event_basis:(.market_context_gap.pending_unknown_event_basis // 0),
            planned_terminal_record_count:(
              [
                $context_records[]
                | select(.recovery_reason == "terminal_missing_market_context")
              ]
              | length
            ),
            planned_pending_record_count:(
              [
                $context_records[]
                | select(.recovery_reason == "pending_market_context_materialization")
              ]
              | length
            ),
            planned_context_record_count:($context_records | length),
            historical_terminal_missing_present:(.market_context_gap.historical_terminal_missing_context_present // false),
            current_or_unknown_terminal_missing_present:(.market_context_gap.current_or_unknown_terminal_missing_context_present // false),
            historical_pending_context_present:(.market_context_gap.historical_pending_context_present // false),
            current_or_unknown_pending_context_present:(.market_context_gap.current_or_unknown_pending_context_present // false),
            terminal_missing_event_basis_min_ms:$min_ms,
            terminal_missing_event_basis_min_at:($min_ms | iso_ms),
            terminal_missing_event_basis_max_ms:$max_ms,
            terminal_missing_event_basis_max_at:($max_ms | iso_ms),
            recovery_input_start_ms:$recovery_start_ms,
            recovery_input_start_at:($recovery_start_ms | iso_ms),
            recovery_input_end_ms:$recovery_end_ms,
            recovery_input_end_at:($recovery_end_ms | iso_ms),
            recovery_margin_ms:$recovery_margin_ms,
            venue:$market_venue,
            market_symbol:market_symbol_for(.symbol),
            symbol_mapping_status:mapping_status_for(.symbol),
            requires_symbol_mapping_review:(mapping_status_for(.symbol) == "derived_quote_suffix_unverified"),
            recovery_window_count:($recovery_windows | length),
            recovery_windows:$recovery_windows,
            post_repair_checks:[
              "rerun_candidate_source_gap_diagnosis_v2",
              "rebuild_current_approved_research_batch_manifest",
              "keep_dispatcher_shadow_paper_live_closed_until_research_gate_passes"
            ],
            sample_terminal_missing_context:(.market_context_gap.sample_terminal_missing_context // []),
            sample_pending_context:(.market_context_gap.sample_pending_context // [])
          }
      ] as $symbols
    | ($symbols | map(.recovery_input_start_ms) | min) as $recovery_start_ms
    | ($symbols | map(.recovery_input_end_ms) | max) as $recovery_end_ms
    | {
        schema_version:"candidate_market_l1_recovery_plan_v1",
        generated_at:$generated_at,
        input:{
          source_gap_file:$source_gap_file,
          source_gap_schema:($gap.schema_version // null),
          observed_context_floor_ms:($gap.summary.global_market_context_gap.observed_context_floor_ms // null),
          observed_context_floor_at:($gap.summary.global_market_context_gap.observed_context_floor_at // null),
          market_ingest_app_root:$market_ingest_app_root,
          market_venue:$market_venue,
          market_quote_suffix:$market_quote_suffix,
          market_symbol_map_file:(
            if ($market_symbol_map_file | length) == 0 then null else $market_symbol_map_file end
          ),
          symbol_map_source:$symbol_map_source,
          recovery_margin_ms:$recovery_margin_ms,
          market_window_ms:$market_window_ms,
          normalize_schedule_interval_ms:$normalize_schedule_interval_ms
        },
        safety:{
          local_planning_only:true,
          s3_read:false,
          s3_write:false,
          ecs_task_started:false,
          dispatcher_mode_changed:false,
          research_run_task_started:false,
          shadow_paper_live_enabled:false,
          execution_ready:false,
          execution_requires_explicit_approval:true
        },
        summary:{
          approved_symbols_without_candidate:($gap.summary.approved_symbols_without_candidate // null),
          source_gap_primary_blocker_counts:($gap.summary.primary_blocker_counts // []),
          global_market_context_gap:($gap.summary.global_market_context_gap // {}),
          context_recovery_symbol_count:($symbols | length),
          terminal_missing_symbol_count:(
            [$symbols[] | select((.terminal_missing_context_packets // 0) > 0)]
            | length
          ),
          pending_context_symbol_count:(
            [$symbols[] | select((.pending_context_packets // 0) > 0)]
            | length
          ),
          historical_symbol_count:(
            [$symbols[] | select(.historical_terminal_missing_present)]
            | length
          ),
          historical_pending_context_symbol_count:(
            [$symbols[] | select(.historical_pending_context_present)]
            | length
          ),
          current_or_unknown_terminal_missing_symbol_count:(
            [$symbols[] | select(.current_or_unknown_terminal_missing_present)]
            | length
          ),
          current_or_unknown_pending_context_symbol_count:(
            [$symbols[] | select(.current_or_unknown_pending_context_present)]
            | length
          ),
          full_historical_backfill_symbol_count:(
            [$symbols[] | select(.recovery_class == "full_historical_backfill_required")]
            | length
          ),
          mixed_terminal_context_symbol_count:(
            [$symbols[] | select(.recovery_class == "mixed_terminal_context_recovery")]
            | length
          ),
          historical_terminal_context_symbol_count:(
            [$symbols[] | select(.recovery_class == "historical_terminal_context_recovery")]
            | length
          ),
          current_or_unknown_terminal_context_symbol_count:(
            [$symbols[] | select(.recovery_class == "current_or_unknown_terminal_context_recovery")]
            | length
          ),
          recovery_window_count:(
            reduce $symbols[] as $symbol (0;
              . + ($symbol.recovery_window_count // 0)
            )
          ),
          symbols_requiring_symbol_mapping_review:(
            [$symbols[] | select(.requires_symbol_mapping_review)]
            | length
          ),
          recovery_input_start_ms:$recovery_start_ms,
          recovery_input_start_at:($recovery_start_ms | iso_ms),
          recovery_input_end_ms:$recovery_end_ms,
          recovery_input_end_at:($recovery_end_ms | iso_ms)
        },
        execution_preconditions:[
          "review market_symbol values against /Volumes/WD/Developments/nangman-crypto/apps/market-ingest-app/config/universe.major-50.toml",
          "replace placeholder buckets and region in generated argument arrays",
          "run market-backfill and market-normalize only after explicit operator approval",
          "do not enable dispatcher run_task, shadow, paper, or live from this plan"
        ],
        market_ingest_contract:{
          historical_l0_worker:"market-backfill",
          historical_l1_worker:"market-normalize",
          l0_worker_writes_s3:true,
          l1_worker_writes_s3:true,
          plan_executes_workers:false,
          read_only_audit_worker:"market-normalize --audit-l1-index-*",
          expected_artifacts_after_approved_execution:[
            "raw_market_event L0",
            "normalized_market_slice_v1 L1",
            "l1_index success pointer",
            "normalization report",
            "symbol_universe_snapshot where applicable"
          ]
        },
        symbols:$symbols,
        recommended_next_actions:[
          "review_symbol_mapping_before_backfill",
          "choose_market_l1_recovery_or_stale_public_intel_marking",
          "rerun_source_gap_diagnosis_after_market_l1_repair",
          "keep_research_dispatcher_dry_run_until_candidate_source_gap_closes"
        ]
      }
  ' > "$tmp_output"

if [[ -n "$OUTPUT_FILE" ]]; then
  cp "$tmp_output" "$OUTPUT_FILE"
  {
    echo "market_l1_recovery_plan_output=$OUTPUT_FILE"
    jq -r '
      "terminal_missing_symbol_count=\(.summary.terminal_missing_symbol_count)",
      "pending_context_symbol_count=\(.summary.pending_context_symbol_count)",
      "historical_symbol_count=\(.summary.historical_symbol_count)",
      "historical_pending_context_symbol_count=\(.summary.historical_pending_context_symbol_count)",
      "current_or_unknown_terminal_missing_symbol_count=\(.summary.current_or_unknown_terminal_missing_symbol_count)",
      "current_or_unknown_pending_context_symbol_count=\(.summary.current_or_unknown_pending_context_symbol_count)",
      "full_historical_backfill_symbol_count=\(.summary.full_historical_backfill_symbol_count)",
      "mixed_terminal_context_symbol_count=\(.summary.mixed_terminal_context_symbol_count)",
      "historical_terminal_context_symbol_count=\(.summary.historical_terminal_context_symbol_count)",
      "current_or_unknown_terminal_context_symbol_count=\(.summary.current_or_unknown_terminal_context_symbol_count)",
      "recovery_window_count=\(.summary.recovery_window_count)",
      "symbols_requiring_symbol_mapping_review=\(.summary.symbols_requiring_symbol_mapping_review)"
    ' "$OUTPUT_FILE"
  } >&2
else
  cat "$tmp_output"
fi
