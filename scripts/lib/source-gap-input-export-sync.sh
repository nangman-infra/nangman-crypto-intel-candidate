#!/usr/bin/env bash

sync_source_gap_inputs() {
  sync_prefix \
    "$input_bucket" \
    "structured-intel-packet/schema=structured_intel_packet_v1/dt=$SOURCE_DT/" \
    "$structured_dir"
  sync_prefix \
    "$output_bucket" \
    "candidate-screening/schema=intel_candidate_screening_event_v1/dt=$SOURCE_DT/" \
    "$screening_dir"
  sync_prefix \
    "$output_bucket" \
    "hypothesis-state/schema=intel_candidate_hypothesis_state_v1/dt=$SOURCE_DT/" \
    "$hypothesis_dir"

  printf '%s\n' "$PRIORITIES" | tr ',' '\n' | while IFS= read -r priority; do
    [[ -z "$priority" ]] && continue
    require_priority_segment "$priority"
    sync_prefix \
      "$output_bucket" \
      "candidate-evidence-bundle/priority=$priority/schema=intel_candidate_evidence_bundle_v1/dt=$SOURCE_DT/" \
      "$evidence_dir/$priority"
  done
}

count_source_gap_inputs() {
  structured_files="$(count_files "$structured_dir")"
  screening_files="$(count_files "$screening_dir")"
  hypothesis_files="$(count_files "$hypothesis_dir")"
  evidence_files="$(count_files "$evidence_dir")"
}
