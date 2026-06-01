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

def supersedes_packet_id:
  (.supersedes_packet_id? // .superseded_packet_id? // .lineage.supersedes_packet_id? // "");

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

def iso_ms:
  if . == null then null
  else ((. / 1000) | strftime("%Y-%m-%dT%H:%M:%SZ"))
  end;

def histogram($values; $key_name):
  reduce $values[] as $value ({};
    .[$value] = (.[$value] // 0) + 1
  )
  | to_entries
  | sort_by([-.value, .key])
  | map({($key_name): .key, count: .value});
