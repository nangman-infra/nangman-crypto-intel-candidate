#!/usr/bin/env bash
set -euo pipefail

COVERAGE_GAP_FILE="${INTEL_CANDIDATE_COVERAGE_GAP_FILE:-${1:-}}"
OUTPUT_FILE="${INTEL_CANDIDATE_SOURCE_GAP_OUTPUT:-${2:-}}"

STRUCTURED_PACKET_PATHS="${INTEL_CANDIDATE_STRUCTURED_PACKET_PATHS:-}"
SCREENING_EVENT_PATHS="${INTEL_CANDIDATE_SCREENING_EVENT_PATHS:-}"
HYPOTHESIS_STATE_PATHS="${INTEL_CANDIDATE_HYPOTHESIS_STATE_PATHS:-}"
EVIDENCE_BUNDLE_PATHS="${INTEL_CANDIDATE_EVIDENCE_BUNDLE_PATHS:-}"

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
  case "$path" in
    /*) ;;
    *)
      echo "$name must be an absolute path; got $path" >&2
      exit 1
      ;;
  esac
}

collect_json_paths() {
  local name="$1"
  local raw="$2"
  if [[ -z "$raw" ]]; then
    return
  fi

  printf '%s\n' "$raw" | tr ',' '\n' | while IFS= read -r path; do
    [[ -z "$path" ]] && continue
    if [[ "$path" != /* ]]; then
      echo "$name must contain only absolute file or directory paths; got $path" >&2
      exit 1
    fi
    if [[ -f "$path" ]]; then
      printf '%s\n' "$path"
    elif [[ -d "$path" ]]; then
      find "$path" -type f \( -name '*.json' -o -name '*.jsonl' \) | sort
    else
      echo "$name path does not exist: $path" >&2
      exit 1
    fi
  done
}

append_json_records() {
  local destination="$1"
  shift

  : > "$destination"
  for path in "$@"; do
    jq -c 'if type == "array" then .[] else . end' "$path" >> "$destination"
  done
}

require_command date
require_command find
require_command jq
require_command mktemp
require_command sort
require_absolute_file "INTEL_CANDIDATE_COVERAGE_GAP_FILE or first argument" "$COVERAGE_GAP_FILE"
require_absolute_output_path "INTEL_CANDIDATE_SOURCE_GAP_OUTPUT or second argument" "$OUTPUT_FILE"

structured_paths=()
while IFS= read -r path; do
  structured_paths+=("$path")
done < <(collect_json_paths "INTEL_CANDIDATE_STRUCTURED_PACKET_PATHS" "$STRUCTURED_PACKET_PATHS")

screening_paths=()
while IFS= read -r path; do
  screening_paths+=("$path")
done < <(collect_json_paths "INTEL_CANDIDATE_SCREENING_EVENT_PATHS" "$SCREENING_EVENT_PATHS")

hypothesis_paths=()
while IFS= read -r path; do
  hypothesis_paths+=("$path")
done < <(collect_json_paths "INTEL_CANDIDATE_HYPOTHESIS_STATE_PATHS" "$HYPOTHESIS_STATE_PATHS")

evidence_paths=()
while IFS= read -r path; do
  evidence_paths+=("$path")
done < <(collect_json_paths "INTEL_CANDIDATE_EVIDENCE_BUNDLE_PATHS" "$EVIDENCE_BUNDLE_PATHS")

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

structured_records="$tmp_dir/structured.jsonl"
screening_records="$tmp_dir/screening.jsonl"
hypothesis_records="$tmp_dir/hypothesis.jsonl"
evidence_records="$tmp_dir/evidence.jsonl"
tmp_output="$tmp_dir/output.json"

if [[ "${#structured_paths[@]}" -gt 0 ]]; then
  append_json_records "$structured_records" "${structured_paths[@]}"
else
  : > "$structured_records"
fi

if [[ "${#screening_paths[@]}" -gt 0 ]]; then
  append_json_records "$screening_records" "${screening_paths[@]}"
else
  : > "$screening_records"
fi

if [[ "${#hypothesis_paths[@]}" -gt 0 ]]; then
  append_json_records "$hypothesis_records" "${hypothesis_paths[@]}"
else
  : > "$hypothesis_records"
fi

if [[ "${#evidence_paths[@]}" -gt 0 ]]; then
  append_json_records "$evidence_records" "${evidence_paths[@]}"
else
  : > "$evidence_records"
fi

jq -n \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg coverage_gap_file "$COVERAGE_GAP_FILE" \
  --argjson structured_path_count "${#structured_paths[@]}" \
  --argjson screening_path_count "${#screening_paths[@]}" \
  --argjson hypothesis_path_count "${#hypothesis_paths[@]}" \
  --argjson evidence_path_count "${#evidence_paths[@]}" \
  --slurpfile coverage "$COVERAGE_GAP_FILE" \
  --slurpfile structured "$structured_records" \
  --slurpfile screening "$screening_records" \
  --slurpfile hypothesis "$hypothesis_records" \
  --slurpfile evidence "$evidence_records" \
  '
    def canonical_symbol:
      (tostring | ascii_upcase | gsub("[^A-Z0-9]"; "")) as $symbol
      | if (($symbol | length) > 4 and ($symbol | endswith("USDT"))) then $symbol[0:-4]
        elif (($symbol | length) > 6 and ($symbol | endswith("USDC"))) then $symbol[0:-4]
        else $symbol
        end;

    def as_symbol_list:
      if . == null then []
      elif type == "array" then
        [
          .[]
          | if type == "object" then (.symbol? // .base_symbol? // .asset? // .ticker? // empty)
            else .
            end
        ]
      elif type == "object" then
        [(.symbol? // .base_symbol? // .asset? // .ticker? // empty)]
      else [.]
      end;

    def record_symbols:
      [
        (.normalized_symbols? | as_symbol_list[]?),
        (.symbols? | as_symbol_list[]?),
        (.symbol? | as_symbol_list[]?),
        (.base_symbol? | as_symbol_list[]?),
        (.asset? | as_symbol_list[]?),
        (.ticker? | as_symbol_list[]?)
      ]
      | map(canonical_symbol)
      | map(select(length > 0))
      | unique
      | sort;

    def packet_id:
      (.packet_id? // .input_packet_id? // .structured_packet_id? // .id? // "");

    def reasons:
      [
        (.reasons? // []),
        (.retryable_reasons? // []),
        (.terminal_reasons? // []),
        (.observe_or_reject_reasons? // []),
        (.missing_reasons? // [])
      ]
      | flatten
      | map(tostring)
      | map(select(length > 0));

    def candidate_class:
      (.candidate_class? // .current_state? // .candidate_state? // "unknown" | tostring);

    def context_status:
      (.market_context_status? // .market_context_ref.status? // .market_context.status? // "" | tostring);

    def context_basis_ms:
      (.market_context_ref.basis_timestamp_ms? // .market_context.basis_timestamp_ms? // null);

    def event_basis_ms:
      (.published_at_ms? // .fetched_at_ms? // .event_timestamp_ms? // .decision_available_at_ms? // null);

    def terminal_context_reason:
      (.market_context_terminal_reason? // .market_context.terminal_reason? // "");

    def is_available_market_context:
      (context_status) as $status
      | [
          "available_symbol_context",
          "available_general_context",
          "nearest_available",
          "stale_but_usable",
          "symbol_context_only"
        ]
      | index($status) != null;

    def is_terminal_missing_market_context:
      context_status == "unavailable"
      and terminal_context_reason == "terminal_missing_market_context";

    def iso_ms:
      if . == null then null
      else ((. / 1000) | strftime("%Y-%m-%dT%H:%M:%SZ"))
      end;

    def reason_group:
      if . == "missing_market_feature_delta"
        or . == "derivatives_metric_delta_missing"
      then "market_context_materialization"
      elif . == "missing_market_regime_context"
        or . == "missing_data_quality_summary"
        or . == "market_context_not_research_admissible"
      then "market_context_materialization"
      elif . == "missing_point_in_time_universe"
        or . == "not_admitted_universe"
      then "point_in_time_universe_admission"
      elif . == "weak_symbol_resolution"
        or . == "missing_symbol_resolution_trace"
      then "symbol_resolution"
      elif . == "missing_evidence"
      then "evidence_quality"
      elif . == "missing_source_independence"
        or . == "insufficient_source_independence"
        or . == "single_source_only"
      then "source_independence"
      else "other"
      end;

    def horizon_ms($h):
      if $h == "15m" then 900000
      elif $h == "1h" then 3600000
      elif $h == "4h" then 14400000
      elif $h == "24h" then 86400000
      elif $h == "72h" then 259200000
      elif $h == "7d" then 604800000
      else null
      end;

    def evidence_horizon_contract_valid:
      (.allowed_horizons // []) as $horizons
      | ($horizons | length) > 0
        and all($horizons[]; (horizon_ms(.) != null and horizon_ms(.) <= 259200000));

    def evidence_ref_values($matches):
      [
        $matches[]
        | (.bundle_key? // .key? // .storage_uri? // empty)
      ]
      | unique
      | sort;

    def evidence_contract_summary($matches):
      {
        evidence_ref_count:(evidence_ref_values($matches) | length),
        evidence_refs:evidence_ref_values($matches),
        research_eligible_count:([$matches[] | select(.research_eligible == true)] | length),
        approved_universe_count:([$matches[] | select(.approved_universe_symbol == true)] | length),
        horizon_contract_valid_count:([$matches[] | select(evidence_horizon_contract_valid)] | length),
        latest_created_at_ms:(
          [
            $matches[]
            | (.candidate_created_at_ms? // .created_at_ms? // null)
            | select(. != null)
          ]
          | max
        ),
        latest_created_at:(
          [
            $matches[]
            | (.candidate_created_at_ms? // .created_at_ms? // null)
            | select(. != null)
          ]
          | max
          | iso_ms
        ),
        sample_evidence_refs:(
          evidence_ref_values($matches) | .[0:10]
        )
      };

    def histogram($values; $key_name):
      reduce $values[] as $value ({};
        .[$value] = (.[$value] // 0) + 1
      )
      | to_entries
      | sort_by([-.value, .key])
      | map({($key_name): .key, count: .value});

    def primary_blocker($status; $groups; $market_context_gap):
      if $status == "no_structured_intel_seen"
      then "structured_intel_absent"
      elif $status == "structured_intel_without_screening"
      then "candidate_worker_input_gap"
      elif $status == "candidate_evidence_outside_research_batch_selection"
      then "research_batch_scan_window"
      elif $status == "candidate_evidence_present_not_in_research_gap"
      then "research_manifest_reconciliation"
      elif (($market_context_gap.historical_backfill_required // false)
        and any($groups[]?; .blocker_group == "market_context_materialization"))
      then "historical_market_l1_backfill_required"
      elif any($groups[]?; .blocker_group == "market_context_materialization")
      then "market_context_materialization"
      elif any($groups[]?; .blocker_group == "point_in_time_universe_admission")
      then "point_in_time_universe_admission"
      elif any($groups[]?; .blocker_group == "symbol_resolution")
      then "symbol_resolution"
      elif any($groups[]?; .blocker_group == "evidence_quality")
      then "evidence_quality"
      elif any($groups[]?; .blocker_group == "source_independence")
      then "source_independence"
      elif $status == "screened_without_research_candidate"
      then "unclassified_screening_gap"
      else "no_candidate_gap_detected"
      end;

    def symbolized($records; $structured_records):
      [
        $records[] as $record
        | (($record | record_symbols) as $direct_symbols
          | if ($direct_symbols | length) > 0 then $direct_symbols
            else [
              $structured_records[]
              | select(packet_id == ($record | packet_id))
              | record_symbols[]
            ] | unique | sort
            end) as $symbols
        | $record + {
            __packet_id:($record | packet_id),
            __symbols:$symbols
          }
      ];

    def records_for_symbol($records; $symbol):
      [
        $records[]
        | select((.__symbols // record_symbols) | index($symbol))
      ];

    ($coverage[0]) as $gap
    | ($gap.gaps.approved_symbols_without_candidate // [] | map(canonical_symbol) | unique | sort) as $missing_symbols
    | ($structured | map(. + {__packet_id:packet_id, __symbols:record_symbols})) as $structured_symbolized
    | ([
        $structured_symbolized[]
        | select(is_available_market_context)
        | context_basis_ms
        | select(. != null)
      ] | min) as $observed_market_context_floor_ms
    | symbolized($screening; $structured_symbolized) as $screening_symbolized
    | symbolized($hypothesis; $structured_symbolized) as $hypothesis_symbolized
    | symbolized($evidence; $structured_symbolized) as $evidence_symbolized
    | [
        $missing_symbols[] as $symbol
        | records_for_symbol($structured_symbolized; $symbol) as $structured_matches
        | records_for_symbol($screening_symbolized; $symbol) as $screening_matches
        | records_for_symbol($hypothesis_symbolized; $symbol) as $hypothesis_matches
        | records_for_symbol($evidence_symbolized; $symbol) as $evidence_matches
        | (evidence_contract_summary($evidence_matches)) as $evidence_contract
        | ([
            ($screening_matches[] | reasons[]),
            ($hypothesis_matches[] | reasons[]),
            ($evidence_matches[] | reasons[])
          ]) as $reason_values
        | ([
            ($screening_matches[] | candidate_class),
            ($hypothesis_matches[] | candidate_class),
            ($evidence_matches[] | candidate_class)
          ] | map(select(length > 0))) as $class_values
        | (
            if (
              ($evidence_matches | length) > 0
              and ($evidence_contract.research_eligible_count // 0) > 0
              and ($evidence_contract.approved_universe_count // 0) > 0
              and ($evidence_contract.horizon_contract_valid_count // 0) > 0
            ) then "candidate_evidence_outside_research_batch_selection"
            elif ($evidence_matches | length) > 0 then "candidate_evidence_present_not_in_research_gap"
            elif (($screening_matches | length) > 0) or (($hypothesis_matches | length) > 0) then "screened_without_research_candidate"
            elif ($structured_matches | length) > 0 then "structured_intel_without_screening"
            else "no_structured_intel_seen"
            end
          ) as $status
        | (histogram(($reason_values | map(reason_group)); "blocker_group")) as $blocker_groups
        | ([
            $structured_matches[]
            | select(is_terminal_missing_market_context)
            | {
                packet_id:.__packet_id,
                symbols:.__symbols,
                published_at_ms:(.published_at_ms? // null),
                fetched_at_ms:(.fetched_at_ms? // null),
                event_basis_ms:event_basis_ms,
                event_basis_at:(event_basis_ms | iso_ms)
              }
          ]) as $terminal_missing_context_matches
        | ([
            $terminal_missing_context_matches[]
            | select(
                $observed_market_context_floor_ms != null
                and .event_basis_ms != null
                and .event_basis_ms < $observed_market_context_floor_ms
              )
          ]) as $historical_terminal_missing_context_matches
        | ([
            $terminal_missing_context_matches[]
            | select(
                ($observed_market_context_floor_ms == null)
                or (.event_basis_ms == null)
                or (.event_basis_ms >= $observed_market_context_floor_ms)
              )
          ]) as $current_or_unknown_terminal_missing_context_matches
        | ([
            $structured_matches[]
            | select(is_available_market_context)
            | .__packet_id
          ] | unique | sort) as $available_context_packet_ids
        | {
            observed_context_floor_ms:$observed_market_context_floor_ms,
            observed_context_floor_at:($observed_market_context_floor_ms | iso_ms),
            terminal_missing_context_packets:($terminal_missing_context_matches | length),
            terminal_missing_before_observed_context_floor:($historical_terminal_missing_context_matches | length),
            terminal_missing_at_or_after_observed_context_floor:($current_or_unknown_terminal_missing_context_matches | length),
            terminal_missing_unknown_event_basis:(
              [
                $terminal_missing_context_matches[]
                | select(.event_basis_ms == null)
              ]
              | length
            ),
            available_context_packets:($available_context_packet_ids | length),
            historical_terminal_missing_context_present:(($historical_terminal_missing_context_matches | length) > 0),
            current_or_unknown_terminal_missing_context_present:(($current_or_unknown_terminal_missing_context_matches | length) > 0),
            historical_terminal_missing_event_basis_min_ms:(
              [
                $historical_terminal_missing_context_matches[]
                | .event_basis_ms
                | select(. != null)
              ]
              | min
            ),
            historical_terminal_missing_event_basis_min_at:(
              [
                $historical_terminal_missing_context_matches[]
                | .event_basis_ms
                | select(. != null)
              ]
              | min
              | iso_ms
            ),
            historical_terminal_missing_event_basis_max_ms:(
              [
                $historical_terminal_missing_context_matches[]
                | .event_basis_ms
                | select(. != null)
              ]
              | max
            ),
            historical_terminal_missing_event_basis_max_at:(
              [
                $historical_terminal_missing_context_matches[]
                | .event_basis_ms
                | select(. != null)
              ]
              | max
              | iso_ms
            ),
            historical_backfill_required:(
              ($terminal_missing_context_matches | length) > 0
              and $observed_market_context_floor_ms != null
              and ($historical_terminal_missing_context_matches | length) == ($terminal_missing_context_matches | length)
            ),
            sample_terminal_missing_context:(
              $terminal_missing_context_matches
              | sort_by(.event_basis_ms // 0)
              | .[0:10]
            ),
            historical_terminal_missing_context_records:(
              $historical_terminal_missing_context_matches
              | sort_by(.event_basis_ms // 0)
            ),
            current_or_unknown_terminal_missing_context_records:(
              $current_or_unknown_terminal_missing_context_matches
              | sort_by(.event_basis_ms // 0)
            )
          } as $market_context_gap
        | {
            symbol:$symbol,
            status:$status,
            primary_blocker:primary_blocker($status; $blocker_groups; $market_context_gap),
            counts:{
              structured_packets:($structured_matches | length),
              screening_events:($screening_matches | length),
              hypothesis_states:($hypothesis_matches | length),
              evidence_bundles:($evidence_matches | length)
            },
            evidence_contract:$evidence_contract,
            market_context_gap:$market_context_gap,
            candidate_classes:histogram($class_values; "candidate_class"),
            blocker_groups:$blocker_groups,
            rejection_reasons:histogram($reason_values; "reason"),
            sample_packet_ids:(
              [
                ($structured_matches[] | .__packet_id),
                ($screening_matches[] | .__packet_id),
                ($hypothesis_matches[] | .__packet_id),
                ($evidence_matches[] | .__packet_id)
              ]
              | map(select(length > 0))
              | unique
              | sort
              | .[0:10]
            )
          }
      ] as $symbol_diagnostics
    | (histogram(($symbol_diagnostics | map(.status)); "status")) as $status_counts
    | (histogram(($symbol_diagnostics | map(.primary_blocker)); "primary_blocker")) as $primary_blocker_counts
    | (histogram(([
        $symbol_diagnostics[]
        | .rejection_reasons[]?
        | . as $entry
        | range(0; $entry.count)
        | $entry.reason
      ]); "reason")) as $global_reasons
    | (histogram(([
        $symbol_diagnostics[]
        | .blocker_groups[]?
        | . as $entry
        | range(0; $entry.count)
        | $entry.blocker_group
      ]); "blocker_group")) as $global_blocker_groups
    | (histogram(([
        $symbol_diagnostics[]
        | .candidate_classes[]?
        | . as $entry
        | range(0; $entry.count)
        | $entry.candidate_class
      ]); "candidate_class")) as $global_classes
    | ({
        observed_context_floor_ms:$observed_market_context_floor_ms,
        observed_context_floor_at:($observed_market_context_floor_ms | iso_ms),
        symbols_with_terminal_missing_context:(
          [$symbol_diagnostics[] | select((.market_context_gap.terminal_missing_context_packets // 0) > 0)]
          | length
        ),
        symbols_with_historical_terminal_missing_context:(
          [$symbol_diagnostics[] | select(.market_context_gap.historical_terminal_missing_context_present // false)]
          | length
        ),
        symbols_with_current_or_unknown_terminal_missing_context:(
          [$symbol_diagnostics[] | select(.market_context_gap.current_or_unknown_terminal_missing_context_present // false)]
          | length
        ),
        symbols_requiring_full_historical_backfill:(
          [$symbol_diagnostics[] | select(.market_context_gap.historical_backfill_required // false)]
          | length
        ),
        terminal_missing_context_packets:(
          reduce $symbol_diagnostics[] as $symbol (0;
            . + ($symbol.market_context_gap.terminal_missing_context_packets // 0)
          )
        ),
        terminal_missing_before_observed_context_floor:(
          reduce $symbol_diagnostics[] as $symbol (0;
            . + ($symbol.market_context_gap.terminal_missing_before_observed_context_floor // 0)
          )
        ),
        terminal_missing_at_or_after_observed_context_floor:(
          reduce $symbol_diagnostics[] as $symbol (0;
            . + ($symbol.market_context_gap.terminal_missing_at_or_after_observed_context_floor // 0)
          )
        ),
        available_context_packets:(
          reduce $symbol_diagnostics[] as $symbol (0;
            . + ($symbol.market_context_gap.available_context_packets // 0)
          )
        )
      }) as $global_market_context_gap
    | {
        schema_version:"intel_candidate_source_gap_diagnosis_v2",
        generated_at:$generated_at,
        input:{
          coverage_gap_file:$coverage_gap_file,
          coverage_gap_schema:($gap.schema_version // null),
          coverage_blocking_stage:($gap.coverage.blocking_stage // null),
          structured_packet_path_count:$structured_path_count,
          screening_event_path_count:$screening_path_count,
          hypothesis_state_path_count:$hypothesis_path_count,
          evidence_bundle_path_count:$evidence_path_count
        },
        safety:{
          s3_read:false,
          s3_write:false,
          ecs_task_started:false,
          dispatcher_mode_changed:false,
          local_diagnosis_only:true,
          shadow_paper_live_enabled:false
        },
        summary:{
          approved_symbols_without_candidate:($missing_symbols | length),
          status_counts:$status_counts,
          primary_blocker_counts:$primary_blocker_counts,
          global_candidate_classes:$global_classes,
          global_blocker_groups:$global_blocker_groups,
          global_rejection_reasons:$global_reasons,
          global_market_context_gap:$global_market_context_gap
        },
        symbols:$symbol_diagnostics,
        recommended_actions:(
          [
            if any($symbol_diagnostics[]?; .status == "no_structured_intel_seen")
              then "increase_structured_intel_source_coverage_for_missing_major50_symbols"
              else empty
            end,
            if any($symbol_diagnostics[]?; .status == "structured_intel_without_screening")
              then "inspect_candidate_worker_input_scan_and_structured_pointer_delivery"
              else empty
            end,
            if any($symbol_diagnostics[]?; .status == "screened_without_research_candidate")
              then "inspect_scoring_rejection_reasons_before_enabling_dispatcher_run_task"
              else empty
            end,
            if any($symbol_diagnostics[]?; .status == "candidate_evidence_outside_research_batch_selection")
              then "widen_research_candidate_scan_or_build_focused_manifest_for_existing_evidence"
              else empty
            end,
            if any($symbol_diagnostics[]?; .primary_blocker == "market_context_materialization")
              then "repair_or_rehydrate_market_context_before_forcing_candidate_generation"
              else empty
            end,
            if any($symbol_diagnostics[]?; .market_context_gap.historical_terminal_missing_context_present // false)
              then "backfill_historical_market_l1_or_mark_stale_public_intel_before_research_dispatch"
              else empty
            end,
            if any($symbol_diagnostics[]?; .primary_blocker == "point_in_time_universe_admission")
              then "inspect_point_in_time_universe_snapshot_before_widening_candidate_policy"
              else empty
            end,
            if any($symbol_diagnostics[]?; .status == "candidate_evidence_present_not_in_research_gap")
              then "reconcile_candidate_evidence_artifacts_with_research_gap_status"
              else empty
            end,
            "keep_dispatcher_dry_run_until_batch_research_loop_is_closed",
            "do_not_open_shadow_paper_live_from_candidate_source_gap"
          ]
          | unique
        )
      }
  ' > "$tmp_output"

if [[ -n "$OUTPUT_FILE" ]]; then
  cp "$tmp_output" "$OUTPUT_FILE"
  {
    echo "source_gap_output=$OUTPUT_FILE"
    jq -r '
      "approved_symbols_without_candidate=\(.summary.approved_symbols_without_candidate)",
      "status_counts=\(.summary.status_counts | map("\(.status):\(.count)") | join(","))",
      "primary_blockers=\(.summary.primary_blocker_counts | map("\(.primary_blocker):\(.count)") | join(","))",
      "top_rejection_reasons=\(.summary.global_rejection_reasons[0:5] | map("\(.reason):\(.count)") | join(","))"
    ' "$tmp_output"
  } >&2
else
  cat "$tmp_output"
fi

echo "candidate source gap diagnosis completed" >&2
