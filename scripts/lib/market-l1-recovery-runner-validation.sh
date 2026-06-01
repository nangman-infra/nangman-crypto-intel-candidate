# shellcheck shell=bash

validate_recovery_runner_inputs() {
  require_command date
  require_command jq
  require_command mkdir
  require_command mktemp
  require_command xargs

  require_boolean "INTEL_CANDIDATE_MARKET_L1_RECOVERY_DRY_RUN" "$DRY_RUN"
  require_boolean "INTEL_CANDIDATE_MARKET_L1_RECOVERY_RESUME" "$RESUME"
  require_absolute_file "INTEL_CANDIDATE_MARKET_L1_RECOVERY_EXECUTION_MANIFEST_FILE or first argument" "$MANIFEST_FILE"
  require_absolute_dir_path "INTEL_CANDIDATE_MARKET_L1_RECOVERY_RUN_OUTPUT_DIR or second argument" "$OUTPUT_DIR"

  jq -e '.schema_version == "candidate_market_l1_recovery_execution_manifest_v1"' "$MANIFEST_FILE" >/dev/null || {
    echo "manifest must have schema_version=candidate_market_l1_recovery_execution_manifest_v1" >&2
    exit 1
  }

  approval_phrase="$(jq -r '.approval_boundary.approval_phrase // empty' "$MANIFEST_FILE")"
  if [[ -z "$approval_phrase" ]]; then
    echo "manifest approval_boundary.approval_phrase is missing" >&2
    exit 1
  fi
  validate_recovery_manifest_steps
  validate_recovery_phase_selection

  if ! is_true "$DRY_RUN"; then
    if [[ "$APPROVAL" != "$approval_phrase" ]]; then
      echo "set INTEL_CANDIDATE_MARKET_L1_RECOVERY_APPROVAL=$approval_phrase to execute writes" >&2
      exit 1
    fi
    require_real_env AWS_PROFILE
    require_real_env AWS_REGION
    require_real_env MARKET_L0_BUCKET
    require_real_env MARKET_L1_BUCKET
    require_real_env MARKET_L0_SPOOL_ROOT
    require_real_env MARKET_L1_SPOOL_ROOT
    require_real_env MARKET_NORMALIZE_CATCHUP_TMP_ROOT
  fi
}

validate_recovery_phase_selection() {
  local phases=()
  local phase
  local rank
  local previous_rank=0
  local selected_count=0
  local seen_backfill=false
  local seen_normalize=false
  local seen_post_audit=false

  IFS=',' read -r -a phases <<< "$PHASES_CSV"
  for phase in "${phases[@]}"; do
    phase="$(printf '%s' "$phase" | xargs)"
    [[ -n "$phase" ]] || continue
    selected_count=$((selected_count + 1))
    case "$phase" in
      backfill)
        rank=1
        if is_true "$seen_backfill"; then
          echo "duplicate recovery phase: backfill" >&2
          exit 1
        fi
        seen_backfill=true
        ;;
      normalize)
        rank=2
        if is_true "$seen_normalize"; then
          echo "duplicate recovery phase: normalize" >&2
          exit 1
        fi
        seen_normalize=true
        ;;
      post_audit)
        rank=3
        if is_true "$seen_post_audit"; then
          echo "duplicate recovery phase: post_audit" >&2
          exit 1
        fi
        seen_post_audit=true
        ;;
      *)
        echo "unknown recovery phase: $phase" >&2
        exit 1
        ;;
    esac
    if [[ "$rank" -lt "$previous_rank" ]]; then
      echo "recovery phases must be ordered as backfill,normalize,post_audit" >&2
      exit 1
    fi
    previous_rank="$rank"
  done

  if [[ "$selected_count" -eq 0 ]]; then
    echo "INTEL_CANDIDATE_MARKET_L1_RECOVERY_PHASES must include at least one phase" >&2
    exit 1
  fi
}

validate_recovery_manifest_steps() {
  local validation_error
  validation_error="$(jq -r '
    def non_empty_string:
      (type == "string") and (length > 0);
    def positive_integer:
      (type == "number") and (. > 0) and ((floor) == .);
    def absolute_path:
      non_empty_string and startswith("/");
    def string_array:
      (type == "array") and all(.[]; non_empty_string);
    def expected_backfill_command($root; $venue; $step):
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
        ($step.start_ms | tostring),
        "--input-end-ms",
        ($step.end_ms | tostring),
        "--symbols",
        $step.market_symbol,
        "--l0-s3-bucket",
        "${MARKET_L0_BUCKET}",
        "--l0-spool-root",
        "${MARKET_L0_SPOOL_ROOT}",
        "--disable-s3-retention",
        "--aws-region",
        "${AWS_REGION}"
      ];
    def expected_normalize_command($root; $window_ms; $schedule_interval_ms; $step):
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
        ($step.start_ms | tostring),
        "--input-end-ms",
        ($step.end_ms | tostring),
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
    def expected_audit_command($root; $window_ms; $step):
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
        ($step.start_ms | tostring),
        "--audit-l1-index-end-ms",
        ($step.end_ms | tostring),
        "--window-ms",
        ($window_ms | tostring),
        "--disable-s3-retention",
        "--aws-region",
        "${AWS_REGION}"
      ];
    def manifest_input_error:
      if (.input | type) != "object" then
        "input must be an object"
      elif ((.input.market_ingest_app_root | absolute_path) | not) then
        "input.market_ingest_app_root must be an absolute path"
      elif ((.input.market_venue // "") | non_empty_string | not) then
        "input.market_venue must be a non-empty string"
      elif ((.input.market_window_ms | positive_integer) | not) then
        "input.market_window_ms must be a positive integer"
      elif ((.input.normalize_schedule_interval_ms | positive_integer) | not) then
        "input.normalize_schedule_interval_ms must be a positive integer"
      else
        empty
      end;
    def common_step_error($phase; $index; $step; $target; $writes_s3; $reads_s3):
      if ($step | type) != "object" then
        "\($phase)[\($index)] must be an object"
      elif $step.target != $target then
        "\($phase)[\($index)].target must be \($target)"
      elif (($step.start_ms | positive_integer) | not) then
        "\($phase)[\($index)].start_ms must be a positive integer"
      elif (($step.end_ms | positive_integer) | not) then
        "\($phase)[\($index)].end_ms must be a positive integer"
      elif $step.end_ms <= $step.start_ms then
        "\($phase)[\($index)] must have end_ms greater than start_ms"
      elif ($step.command_args | type) != "array" then
        "\($phase)[\($index)].command_args must be an array"
      elif (($step.command_args | string_array) | not) then
        "\($phase)[\($index)].command_args must contain only non-empty strings"
      elif (($step.writes_s3 // false) != $writes_s3) then
        "\($phase)[\($index)].writes_s3 must be \($writes_s3)"
      elif (($step.reads_s3 // false) != $reads_s3) then
        "\($phase)[\($index)].reads_s3 must be \($reads_s3)"
      else
        empty
      end;
    def step_error($phase; $index; $step; $root; $venue; $window_ms; $schedule_interval_ms):
      if $phase == "backfill_steps" then
        [
          common_step_error($phase; $index; $step; "market_l0"; true; false),
          if (($step.market_symbol // "") | non_empty_string | not) then
            "\($phase)[\($index)].market_symbol must be a non-empty string"
          elif $step.command_args != expected_backfill_command($root; $venue; $step) then
            "\($phase)[\($index)].command_args must match generated market-backfill command contract"
          else
            empty
          end
        ] | map(select(length > 0)) | .[0] // empty
      elif $phase == "normalize_steps" then
        [
          common_step_error($phase; $index; $step; "market_l1"; true; false),
          if (($step.market_symbols | string_array) | not) then
            "\($phase)[\($index)].market_symbols must contain non-empty strings"
          elif $step.command_args != expected_normalize_command($root; $window_ms; $schedule_interval_ms; $step) then
            "\($phase)[\($index)].command_args must match generated market-normalize command contract"
          else
            empty
          end
        ] | map(select(length > 0)) | .[0] // empty
      else
        [
          common_step_error($phase; $index; $step; "market_l1_index"; false; true),
          if (($step.market_symbols | string_array) | not) then
            "\($phase)[\($index)].market_symbols must contain non-empty strings"
          elif $step.command_args != expected_audit_command($root; $window_ms; $step) then
            "\($phase)[\($index)].command_args must match generated market-normalize audit command contract"
          else
            empty
          end
        ] | map(select(length > 0)) | .[0] // empty
      end;

    def phase_errors($phase; $root; $venue; $window_ms; $schedule_interval_ms):
      if (.[$phase] | type) != "array" then
        ["\($phase) must be an array"]
      else
        [
          range(0; (.[$phase] | length)) as $index
          | step_error($phase; $index; .[$phase][$index]; $root; $venue; $window_ms; $schedule_interval_ms)
        ]
      end;

    (.input.market_ingest_app_root // "") as $root
    | (.input.market_venue // "") as $venue
    | (.input.market_window_ms // 0) as $window_ms
    | (.input.normalize_schedule_interval_ms // 0) as $schedule_interval_ms
    | [
      manifest_input_error,
      phase_errors("backfill_steps"; $root; $venue; $window_ms; $schedule_interval_ms),
      phase_errors("normalize_steps"; $root; $venue; $window_ms; $schedule_interval_ms),
      phase_errors("post_audit_steps"; $root; $venue; $window_ms; $schedule_interval_ms)
    ]
    | flatten
    | map(select(length > 0))
    | .[0] // empty
  ' "$MANIFEST_FILE")"

  if [[ -n "$validation_error" ]]; then
    echo "manifest step validation failed: $validation_error" >&2
    exit 1
  fi
}
