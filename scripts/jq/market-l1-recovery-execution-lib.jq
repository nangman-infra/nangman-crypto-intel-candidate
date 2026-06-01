def iso_ms:
  if . == null then null
  else ((. / 1000) | strftime("%Y-%m-%dT%H:%M:%SZ"))
  end;

def merge_ranges:
  sort_by(.start_ms, .end_ms)
  | reduce .[] as $range ([];
      if length == 0 then
        [$range]
      else
        .[-1] as $last
        | if $range.start_ms <= $last.end_ms then
            .[:-1] + [
              $last + {
                end_ms:([$last.end_ms, $range.end_ms] | max),
                source_window_count:(($last.source_window_count // 1) + ($range.source_window_count // 1)),
                packet_ids:((($last.packet_ids // []) + ($range.packet_ids // [])) | unique),
                symbols:((($last.symbols // []) + ($range.symbols // [])) | unique),
                market_symbols:((($last.market_symbols // []) + ($range.market_symbols // [])) | unique)
              }
            ]
          else
            . + [$range]
          end
      end
    );

def backfill_args($root; $venue; $symbol; $start_ms; $end_ms):
  [
    "cargo",
    "run",
    "--manifest-path",
    "\($root)/Cargo.toml",
    "--bin",
    "market-backfill",
    "--",
    "--venue",
    $venue,
    "--config",
    "\($root)/config",
    "--input-start-ms",
    ($start_ms | tostring),
    "--input-end-ms",
    ($end_ms | tostring),
    "--symbols",
    $symbol,
    "--l0-s3-bucket",
    "${MARKET_L0_BUCKET}",
    "--l0-spool-root",
    "${MARKET_L0_SPOOL_ROOT}",
    "--disable-s3-retention",
    "--aws-region",
    "${AWS_REGION}"
  ];

def normalize_args($root; $start_ms; $end_ms; $window_ms; $schedule_interval_ms):
  [
    "cargo",
    "run",
    "--manifest-path",
    "\($root)/Cargo.toml",
    "--bin",
    "market-normalize",
    "--",
    "--l0-s3-bucket",
    "${MARKET_L0_BUCKET}",
    "--l0-local-root",
    "${MARKET_L0_SPOOL_ROOT}",
    "--l1-s3-bucket",
    "${MARKET_L1_BUCKET}",
    "--spool-root",
    "${MARKET_L1_SPOOL_ROOT}",
    "--catchup-tmp-root",
    "${MARKET_NORMALIZE_CATCHUP_TMP_ROOT}",
    "--input-start-ms",
    ($start_ms | tostring),
    "--input-end-ms",
    ($end_ms | tostring),
    "--window-ms",
    ($window_ms | tostring),
    "--schedule-interval-ms",
    ($schedule_interval_ms | tostring),
    "--disable-s3-retention",
    "--l1-index-upload-concurrency",
    "32",
    "--aws-region",
    "${AWS_REGION}"
  ];

def audit_args($root; $start_ms; $end_ms; $window_ms):
  [
    "cargo",
    "run",
    "--manifest-path",
    "\($root)/Cargo.toml",
    "--bin",
    "market-normalize",
    "--",
    "--l0-s3-bucket",
    "${MARKET_L0_BUCKET}",
    "--l1-s3-bucket",
    "${MARKET_L1_BUCKET}",
    "--audit-l1-index-start-ms",
    ($start_ms | tostring),
    "--audit-l1-index-end-ms",
    ($end_ms | tostring),
    "--window-ms",
    ($window_ms | tostring),
    "--disable-s3-retention",
    "--aws-region",
    "${AWS_REGION}"
  ];
