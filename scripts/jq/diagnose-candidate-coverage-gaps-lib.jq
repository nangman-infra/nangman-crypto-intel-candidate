def unique_sorted: unique | sort;

def canonical_symbol:
  (tostring | ascii_upcase | gsub("[^A-Z0-9]"; "")) as $symbol
  | if (($symbol | length) > 4 and ($symbol | endswith("USDT"))) then $symbol[0:-4]
    elif (($symbol | length) > 6 and ($symbol | endswith("USDC"))) then $symbol[0:-4]
    else $symbol
    end;

def normalized_symbol_list:
  map(canonical_symbol)
  | map(select(length > 0))
  | unique_sorted;

def require_supported_input:
  if .schema_version == "research_horizon_status_checkpoint_v1" then .
  elif (
    ((.stage_state? | type) == "object")
    and ((.major50_universe? | type) == "object")
    and ((.recent_candidates? | type) == "object")
  ) then .
  else
    error(
      "unsupported research status schema: "
      + (.schema_version // "unknown")
      + "; expected research_horizon_status_checkpoint_v1 or check-loop-state output"
    )
  end;

def stage_count($status; $name; $fallback):
  ($status.research_factory_gap_summary.stage_counts[$name] // $fallback);

def gap_count($status; $name; $fallback):
  ($status.research_factory_gap_summary.gap_counts[$name] // $fallback);
