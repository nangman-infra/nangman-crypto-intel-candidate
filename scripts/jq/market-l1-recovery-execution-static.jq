def recovery_execution_safety:
  {
    local_manifest_only:true,
    s3_read_performed:false,
    s3_write_performed:false,
    ecs_task_started:false,
    dispatcher_mode_changed:false,
    research_run_task_started:false,
    shadow_paper_live_enabled:false,
    contains_real_bucket_values:false,
    contains_aws_profile_value:false,
    execution_requires_explicit_operator_approval:true
  };

def readiness_gate($readiness):
  {
    readiness_file_present:($readiness != null),
    readiness_verdict:($readiness.verdict // null),
    l1_index_audit_completed:($readiness.current_state.l1_index_audit_completed // false),
    exchange_symbol_check_completed:($readiness.current_state.exchange_symbol_check_completed // false),
    missing_index_pointer_count_total:($readiness.evidence.audit_missing_index_pointer_count_total // null),
    exchangeinfo_missing_symbol_count:($readiness.evidence.exchangeinfo_missing_symbol_count // null),
    exchangeinfo_non_trading_symbol_count:($readiness.evidence.exchangeinfo_non_trading_symbol_count // null),
    promotion_passed:($readiness.current_state.promotion_passed // false),
    shadow_created:($readiness.current_state.shadow_created // false),
    paper_created:($readiness.current_state.paper_created // false),
    live_enabled:($readiness.current_state.live_enabled // false)
  };

def required_recovery_environment:
  [
    "AWS_PROFILE",
    "AWS_REGION",
    "MARKET_L0_BUCKET",
    "MARKET_L1_BUCKET",
    "MARKET_L0_SPOOL_ROOT",
    "MARKET_L1_SPOOL_ROOT",
    "MARKET_NORMALIZE_CATCHUP_TMP_ROOT"
  ];

def recovery_approval_boundary:
  {
    required:true,
    approval_phrase:"approve_market_l1_s3_write_recovery",
    reason:"market-backfill writes Market-L0 and market-normalize writes Market-L1 artifacts",
    blocked_until_approved:[
      "do_not_run_backfill_steps",
      "do_not_run_normalize_steps",
      "do_not_switch_dispatcher_out_of_dry_run",
      "do_not_create_shadow",
      "do_not_create_paper",
      "do_not_enable_live"
    ]
  };

def recovery_execution_order:
  [
    "export required environment variables locally",
    "rerun read-only audit if recovery plan is stale",
    "run backfill_steps only after explicit operator approval",
    "run normalize_steps only after backfill completes",
    "run post_audit_steps and require zero missing L1 index pointers",
    "rerun candidate source-gap diagnosis v2",
    "rebuild current-approved research batch manifest",
    "rerun research replay",
    "keep dispatcher/shadow/paper/live closed unless research gate later passes"
  ];

def recovery_post_repair_checks:
  [
    "candidate_source_gap_diagnosis_v2",
    "current_approved_research_batch_manifest",
    "research_replay",
    "promotion_gate",
    "shadow_paper_live_boundary"
  ];
