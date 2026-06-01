#!/usr/bin/env bash

write_market_l1_recovery_symbol_map_from_market_ingest_config() {
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
}

prepare_market_l1_recovery_symbol_map() {
  symbol_map_source="derived_quote_suffix"

  if [[ -n "$MARKET_SYMBOL_MAP_FILE" ]]; then
    require_market_l1_recovery_absolute_file \
      "INTEL_CANDIDATE_MARKET_SYMBOL_MAP_FILE" \
      "$MARKET_SYMBOL_MAP_FILE"
    cp "$MARKET_SYMBOL_MAP_FILE" "$symbol_map_json"
    symbol_map_source="explicit_symbol_map_file"
  elif [[ -f "$MARKET_INGEST_APP_ROOT/config/universe.major-50.toml" ]]; then
    write_market_l1_recovery_symbol_map_from_market_ingest_config
    symbol_map_source="market_ingest_major50_config"
  else
    printf '{}\n' > "$symbol_map_json"
  fi
}
