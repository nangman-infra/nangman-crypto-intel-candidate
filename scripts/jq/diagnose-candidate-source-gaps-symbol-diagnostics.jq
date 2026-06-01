include "diagnose-candidate-source-gaps-lib";
include "diagnose-candidate-source-gaps-report";

def source_gap_status($structured_matches; $screening_matches; $hypothesis_matches; $evidence_matches; $evidence_contract):
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
  end;

def source_gap_symbol_diagnostic(
  $symbol;
  $structured_symbolized;
  $screening_symbolized;
  $hypothesis_symbolized;
  $evidence_symbolized;
  $structured_superseded;
  $screening_superseded;
  $hypothesis_superseded;
  $evidence_superseded;
  $observed_market_context_floor_ms
):
  records_for_symbol($structured_symbolized; $symbol) as $structured_matches
  | records_for_symbol($screening_symbolized; $symbol) as $screening_matches
  | records_for_symbol($hypothesis_symbolized; $symbol) as $hypothesis_matches
  | records_for_symbol($evidence_symbolized; $symbol) as $evidence_matches
  | records_for_symbol($structured_superseded; $symbol) as $structured_superseded_matches
  | records_for_symbol($screening_superseded; $symbol) as $screening_superseded_matches
  | records_for_symbol($hypothesis_superseded; $symbol) as $hypothesis_superseded_matches
  | records_for_symbol($evidence_superseded; $symbol) as $evidence_superseded_matches
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
  | source_gap_status(
      $structured_matches;
      $screening_matches;
      $hypothesis_matches;
      $evidence_matches;
      $evidence_contract
    ) as $status
  | (histogram(($reason_values | map(reason_group)); "blocker_group")) as $blocker_groups
  | ([
      ($structured_matches[] | market_context_record("structured_packet")),
      ($screening_matches[] | market_context_record("screening_event")),
      ($hypothesis_matches[] | market_context_record("hypothesis_state")),
      ($evidence_matches[] | market_context_record("evidence_bundle"))
    ]
    | map(select(.market_context_status != null and (.market_context_status | length) > 0))
    ) as $market_context_matches
  | (market_context_gap_summary($market_context_matches; $observed_market_context_floor_ms)) as $market_context_gap
  | {
      symbol:$symbol,
      status:$status,
      primary_blocker:primary_blocker($status; $blocker_groups; $market_context_gap),
      counts:{
        structured_packets:($structured_matches | length),
        screening_events:($screening_matches | length),
        hypothesis_states:($hypothesis_matches | length),
        evidence_bundles:($evidence_matches | length),
        superseded_structured_packets:($structured_superseded_matches | length),
        superseded_screening_events:($screening_superseded_matches | length),
        superseded_hypothesis_states:($hypothesis_superseded_matches | length),
        superseded_evidence_bundles:($evidence_superseded_matches | length)
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
    };

def source_gap_symbol_diagnostics(
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
):
  [
    $missing_symbols[] as $symbol
    | source_gap_symbol_diagnostic(
        $symbol;
        $structured_symbolized;
        $screening_symbolized;
        $hypothesis_symbolized;
        $evidence_symbolized;
        $structured_superseded;
        $screening_superseded;
        $hypothesis_superseded;
        $evidence_superseded;
        $observed_market_context_floor_ms
      )
  ];
