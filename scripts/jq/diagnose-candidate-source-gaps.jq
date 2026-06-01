include "diagnose-candidate-source-gaps-lib";
include "diagnose-candidate-source-gaps-report";
import "diagnose-candidate-source-gaps-symbol-diagnostics" as symbol_diagnostics;

($coverage[0]) as $gap
| ($gap.gaps.approved_symbols_without_candidate // [] | map(canonical_symbol) | unique | sort) as $missing_symbols
| ($structured | map(. + {__packet_id:packet_id, __supersedes_packet_id:supersedes_packet_id, __symbols:record_symbols})) as $structured_all_symbolized
| symbolized($screening; $structured_all_symbolized) as $screening_all_symbolized
| symbolized($hypothesis; $structured_all_symbolized) as $hypothesis_all_symbolized
| symbolized($evidence; $structured_all_symbolized) as $evidence_all_symbolized
| ([
    ($structured_all_symbolized[] | .__supersedes_packet_id),
    ($screening_all_symbolized[] | .__supersedes_packet_id),
    ($hypothesis_all_symbolized[] | .__supersedes_packet_id),
    ($evidence_all_symbolized[] | .__supersedes_packet_id)
  ] | map(select(length > 0)) | unique | sort) as $superseded_packet_ids
| current_records($structured_all_symbolized; $superseded_packet_ids) as $structured_symbolized
| current_records($screening_all_symbolized; $superseded_packet_ids) as $screening_symbolized
| current_records($hypothesis_all_symbolized; $superseded_packet_ids) as $hypothesis_symbolized
| current_records($evidence_all_symbolized; $superseded_packet_ids) as $evidence_symbolized
| superseded_records($structured_all_symbolized; $superseded_packet_ids) as $structured_superseded
| superseded_records($screening_all_symbolized; $superseded_packet_ids) as $screening_superseded
| superseded_records($hypothesis_all_symbolized; $superseded_packet_ids) as $hypothesis_superseded
| superseded_records($evidence_all_symbolized; $superseded_packet_ids) as $evidence_superseded
| ([
    $structured_symbolized[]
    | select(is_available_market_context)
    | context_basis_ms
    | select(. != null)
  ] | min) as $observed_market_context_floor_ms
| symbol_diagnostics::source_gap_symbol_diagnostics(
    $missing_symbols;
    $structured_symbolized;
    $screening_symbolized;
    $hypothesis_symbolized;
    $evidence_symbolized;
    $structured_superseded;
    $screening_superseded;
    $hypothesis_superseded;
    $evidence_superseded;
    $observed_market_context_floor_ms
  ) as $symbol_diagnostics
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
| (global_market_context_gap_summary($symbol_diagnostics; $observed_market_context_floor_ms)) as $global_market_context_gap
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
      global_market_context_gap:$global_market_context_gap,
      revision_filter:{
        superseded_packet_ids:($superseded_packet_ids | length),
        current_structured_packets:($structured_symbolized | length),
        current_screening_events:($screening_symbolized | length),
        current_hypothesis_states:($hypothesis_symbolized | length),
        current_evidence_bundles:($evidence_symbolized | length),
        superseded_structured_packets:($structured_superseded | length),
        superseded_screening_events:($screening_superseded | length),
        superseded_hypothesis_states:($hypothesis_superseded | length),
        superseded_evidence_bundles:($evidence_superseded | length),
        sample_superseded_packet_ids:($superseded_packet_ids[0:20])
      }
    },
    symbols:$symbol_diagnostics,
    recommended_actions:source_gap_recommended_actions($symbol_diagnostics)
  }
