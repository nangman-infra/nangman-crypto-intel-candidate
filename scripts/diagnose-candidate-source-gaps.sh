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
        elif (($symbol | length) > 4 and ($symbol | endswith("USDC"))) then $symbol[0:-4]
        elif (($symbol | length) > 3 and ($symbol | endswith("USD"))) then $symbol[0:-3]
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

    def histogram($values; $key_name):
      reduce $values[] as $value ({};
        .[$value] = (.[$value] // 0) + 1
      )
      | to_entries
      | sort_by([-.value, .key])
      | map({($key_name): .key, count: .value});

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
    | symbolized($screening; $structured_symbolized) as $screening_symbolized
    | symbolized($hypothesis; $structured_symbolized) as $hypothesis_symbolized
    | symbolized($evidence; $structured_symbolized) as $evidence_symbolized
    | [
        $missing_symbols[] as $symbol
        | records_for_symbol($structured_symbolized; $symbol) as $structured_matches
        | records_for_symbol($screening_symbolized; $symbol) as $screening_matches
        | records_for_symbol($hypothesis_symbolized; $symbol) as $hypothesis_matches
        | records_for_symbol($evidence_symbolized; $symbol) as $evidence_matches
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
        | {
            symbol:$symbol,
            status:(
              if ($evidence_matches | length) > 0 then "candidate_evidence_present_not_in_research_gap"
              elif (($screening_matches | length) > 0) or (($hypothesis_matches | length) > 0) then "screened_without_research_candidate"
              elif ($structured_matches | length) > 0 then "structured_intel_without_screening"
              else "no_structured_intel_seen"
              end
            ),
            counts:{
              structured_packets:($structured_matches | length),
              screening_events:($screening_matches | length),
              hypothesis_states:($hypothesis_matches | length),
              evidence_bundles:($evidence_matches | length)
            },
            candidate_classes:histogram($class_values; "candidate_class"),
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
    | (histogram(([
        $symbol_diagnostics[]
        | .rejection_reasons[]?
        | . as $entry
        | range(0; $entry.count)
        | $entry.reason
      ]); "reason")) as $global_reasons
    | (histogram(([
        $symbol_diagnostics[]
        | .candidate_classes[]?
        | . as $entry
        | range(0; $entry.count)
        | $entry.candidate_class
      ]); "candidate_class")) as $global_classes
    | {
        schema_version:"intel_candidate_source_gap_diagnosis_v1",
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
          global_candidate_classes:$global_classes,
          global_rejection_reasons:$global_reasons
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
      "top_rejection_reasons=\(.summary.global_rejection_reasons[0:5] | map("\(.reason):\(.count)") | join(","))"
    ' "$tmp_output"
  } >&2
else
  cat "$tmp_output"
fi

echo "candidate source gap diagnosis completed" >&2
