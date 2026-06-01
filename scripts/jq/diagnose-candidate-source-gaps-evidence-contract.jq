include "diagnose-candidate-source-gaps-records";

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
