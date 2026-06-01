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
