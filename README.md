# intel-candidate-app

`intel-candidate-app` is the deterministic candidate gate for the AI-DLC alpha discovery loop.

It does not crawl raw web sources, call LLMs, run NLP extraction, make strategy decisions, place orders, or touch private exchange/account APIs. Its job is narrower: read structured intel packets from `intel-structuring`, attach deterministic candidate scoring and lineage, then write candidate artifacts for downstream research.

## Pipeline position

```text
intel-crawl
  -> raw intel L0 in S3
  -> intel-structuring
  -> structured_pointer_v1 on NATS + structured_intel_packet_v1 in S3
  -> intel-candidate
  -> screening events, evidence bundles, hypothesis states, revision index
  -> research
```

`intel-candidate` reads `structured_pointer_v1` messages from NATS and verifies the S3 payload before scoring. It never reads `intel-crawl` raw outputs directly.

## Default execution

Production runs one process: `intel-candidate-agent`.

The agent continuously consumes live structured intel pointers and, when configured, runs bounded S3 repair scans inside the same process. This keeps AI-DLC operation autonomous without forcing an operator to choose between a live binary and a replay binary.

```bash
intel-candidate-agent \
  --nats-url nats://REPLACE_WITH_S2S_NATS_HOST:4222 \
  --input-s3-bucket nangman-crypto-dev-intel-structuring-l1-<account-suffix> \
  --output-s3-bucket nangman-crypto-dev-intel-candidate-<account-suffix> \
  --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
  --policy-file /opt/nangman-crypto/intel-candidate/policies/scoring-policy.v1.json \
  --repair-input-prefix structured-intel-packet/schema=structured_intel_packet_v1/ \
  --repair-interval-secs 3600 \
  --repair-max-keys-per-prefix 500 \
  --repair-max-pages-per-prefix 8 \
  --repair-recent-partition-days 3
```

Repair scans are bounded by prefix, interval, key count, and page count. For the canonical `structured-intel-packet/schema=structured_intel_packet_v1/` prefix, the agent also derives recent `dt=YYYY-MM-DD/` prefixes and scans them before the broad catch-up prefix. This keeps fresh major-50 packets moving through candidate screening even while older historical partitions still contain stale revisions. The agent keeps a per-prefix scan cursor, so stale revision pages do not cause every cycle to reread the same first page forever. Repair uses the same deterministic scoring, idempotent S3 writes, NATS message IDs, and stale revision checks as live processing.

## Canonical storage contract

S3 is the durable source of truth. NATS is only the pointer/event bus.

Input:

```text
NATS stream: STRUCTURED_INTEL
NATS subject: structured_intel_packet.created
S3 schema: structured_intel_packet_v1
```

Output:

```text
NATS stream: INTEL_CANDIDATE
NATS subjects:
  intel_candidate_screening_event.created
  intel_candidate_evidence_bundle.created
  intel_candidate_hypothesis_state.created

S3 artifacts:
  intel_candidate_screening_event_v1
  intel_candidate_evidence_bundle_v1
  intel_candidate_hypothesis_state_v1
  intel_candidate_revision_index_v1
```

The app validates pointer schema, payload schema, S3 checksum, scoring policy, and revision ordering before acknowledging the input message.

## Diagnostic commands

The standalone live and replay binaries remain available for operator diagnostics and one-off backfills, but ECS and Docker default to `intel-candidate-agent`.

```bash
intel-candidate-worker --help
intel-candidate-replay-worker --help
```

Use the candidate coverage gap diagnosis when `research-app` reports
`candidate_generation_coverage`. It reads a local
`research_horizon_status_checkpoint_v1` file or a read-only `research-app`
loop-state check output and separates approved major-50 symbols that still have
no candidate from candidates that have not reached research replay or
promotion. It emits a machine-readable `next_decision` for the candidate
coverage handoff. It does not write S3, start ECS, switch dispatcher mode, or
create shadow/paper/live artifacts.

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/intel-candidate-app

scripts/diagnose-candidate-coverage-gaps.sh \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/retest-horizon-status.json \
  > /tmp/nangman-crypto/research-current-approved-batch/<run-id>/candidate-coverage-gap-diagnosis.json
```

```bash
scripts/diagnose-candidate-coverage-gaps.sh \
  /tmp/nangman-crypto/research-loop-state.json \
  /tmp/nangman-crypto/candidate-coverage-gap-diagnosis.json
```

Then use the candidate source gap diagnosis to split the missing approved
symbols into source/structuring gaps versus candidate screening rejection gaps.
Inputs are local JSON or JSONL files, or absolute directories containing `.json`
and `.jsonl` files. This command is still local-only and keeps dispatcher,
shadow, paper, and live gates closed. The output includes per-symbol
`primary_blocker` and `blocker_groups`, so operator action can separate missing
structured intel from market-context materialization, point-in-time universe,
symbol-resolution, and evidence-quality blockers.

The source gap report also preserves `market_context_gap` details for approved
major-50 symbols that were screened but did not become research candidates. In
`intel_candidate_source_gap_diagnosis_v2`, the report separates ordinary
market-context materialization gaps from terminal missing context tied to events
older than the observed market-context floor. A symbol is marked
`historical_market_l1_backfill_required` only when all of its terminal missing
context packets fall before that floor; mixed cases keep their normal primary
blocker while `summary.global_market_context_gap` and the per-symbol
`historical_terminal_missing_context_present` field show the backlog that needs
historical Market-L1 backfill or stale-public-intel marking before research
dispatch is opened.
When approved, research-eligible candidate evidence already exists but was not
inside the research batch selection window, the report marks the symbol as
`candidate_evidence_outside_research_batch_selection` with primary blocker
`research_batch_scan_window`. That case should widen the candidate scan or build
a focused manifest from the existing evidence, not trigger market-context
backfill or source crawling. The per-symbol `evidence_contract` includes full
`evidence_refs` plus a short `sample_evidence_refs` preview so research tooling
can build a focused manifest without another candidate scan.

To materialize those local inputs from the currently deployed ECS candidate
agent, use the read-only export helper. It reads the task definition to discover
the configured input/output buckets, downloads only the selected UTC `dt`
prefixes into an absolute local directory, and writes a manifest with bucket
names redacted. It does not write S3, start ECS tasks, change dispatcher mode,
or create shadow/paper/live artifacts.

```bash
INTEL_CANDIDATE_ECS_CLUSTER=<ecs-cluster> \
INTEL_CANDIDATE_ECS_SERVICE=<ecs-service> \
INTEL_CANDIDATE_SOURCE_GAP_DT=2026-05-23 \
scripts/export-source-gap-inputs-from-ecs.sh \
  /tmp/nangman-crypto/intel-candidate/source-gap-inputs/2026-05-23
```

```bash
INTEL_CANDIDATE_STRUCTURED_PACKET_PATHS=/tmp/nangman-crypto/intel-candidate/source-gap-inputs/2026-05-23/structured \
INTEL_CANDIDATE_SCREENING_EVENT_PATHS=/tmp/nangman-crypto/intel-candidate/source-gap-inputs/2026-05-23/screening \
INTEL_CANDIDATE_HYPOTHESIS_STATE_PATHS=/tmp/nangman-crypto/intel-candidate/source-gap-inputs/2026-05-23/hypothesis \
INTEL_CANDIDATE_EVIDENCE_BUNDLE_PATHS=/tmp/nangman-crypto/intel-candidate/source-gap-inputs/2026-05-23/evidence \
scripts/diagnose-candidate-source-gaps.sh \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/candidate-coverage-gap-diagnosis.json \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/candidate-source-gap-diagnosis.json
```

If the source gap diagnosis reports terminal missing market context, turn it
into a local Market-L1 recovery plan before running any backfill. The planner
handles both historical gaps before the observed Market-L1 context floor and
current or unknown terminal gaps at or after that floor. It emits packet-sized
recovery windows with placeholder market-ingest arguments. Those windows are
aligned to the `market-normalize` schedule interval and include read-only L1
index audit arguments so operators can prove which windows already exist before
writing new L0/L1 data. It does not read or write S3, start ECS, change
dispatcher mode, or open research/shadow/paper/live gates.
Symbol mappings derived from a quote suffix are marked for review; by default
the planner uses
`/Volumes/WD/Developments/nangman-crypto/apps/market-ingest-app/config/universe.major-50.toml`
when it is available, and `INTEL_CANDIDATE_MARKET_SYMBOL_MAP_FILE` can override
that with an operator-approved JSON mapping.

```bash
scripts/plan-market-l1-recovery-from-source-gaps.sh \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/candidate-source-gap-diagnosis.json \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/candidate-market-l1-recovery-plan.local.json
```

After read-only L1 index audit and symbol checks, turn the recovery plan into a
redacted approval manifest. This manifest merges duplicate or overlapping
windows and keeps bucket/profile values as environment placeholders. It still
does not execute backfill, write S3, start ECS, change dispatcher mode, or open
shadow/paper/live gates.

```bash
scripts/build-market-l1-recovery-execution-manifest.sh \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/candidate-market-l1-recovery-plan.local.json \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/candidate-market-l1-recovery-execution-manifest.redacted.json \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/market-l1-recovery-readiness.redacted.json
```

The approval runner checkpoints each step and can resume already completed
steps. Without the explicit approval phrase it can only dry-run. A non-dry run
executes `market-backfill` and `market-normalize`, so it writes Market-L0/L1 S3
objects. Recovery commands disable S3 retention cleanup so bulk recovery and
bucket cleanup remain separate operator actions. Normalize recovery also raises
L1 index pointer upload concurrency for one-shot repair speed; regular worker
defaults remain unchanged.

```bash
INTEL_CANDIDATE_MARKET_L1_RECOVERY_DRY_RUN=true \
scripts/run-market-l1-recovery-execution-manifest.sh \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/candidate-market-l1-recovery-execution-manifest.redacted.json \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/market-l1-recovery-run
```

```bash
AWS_PROFILE=<sso-profile> \
AWS_REGION=ap-northeast-2 \
MARKET_L0_BUCKET=<market-l0-bucket> \
MARKET_L1_BUCKET=<market-l1-bucket> \
MARKET_L0_SPOOL_ROOT=/tmp/nangman-crypto/market-ingest-spool/l0 \
MARKET_L1_SPOOL_ROOT=/tmp/nangman-crypto/market-ingest-spool/l1 \
MARKET_NORMALIZE_CATCHUP_TMP_ROOT=/tmp/nangman-crypto/market-normalize/catchup \
INTEL_CANDIDATE_MARKET_L1_RECOVERY_APPROVAL=approve_market_l1_s3_write_recovery \
scripts/run-market-l1-recovery-execution-manifest.sh \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/candidate-market-l1-recovery-execution-manifest.redacted.json \
  /tmp/nangman-crypto/research-current-approved-batch/<run-id>/market-l1-recovery-run
```

Research-bound evidence bundles only emit horizons that downstream `research-app`
can admit under its intraday holding contract: `15m`, `1h`, `4h`, `24h`, or
`72h`. Longer horizons such as `7d` stay outside candidate output until the
research holding policy is explicitly widened.

## Deployment defaults

Use ARM64 Fargate with `FARGATE_SPOT` as the preferred capacity provider. The task should connect to on-prem NATS through VPN or private routing, for example `nats://<private-nats-host>:4222`, with security groups/firewall rules limited to the required producers and consumers.

The example task definition is in:

```text
/Volumes/WD/Developments/nangman-crypto/apps/intel-candidate-app/ecs/task-definition.example.json
```

## Quality gate

Sonar coverage focuses on deterministic scoring, contracts, parsing, and artifact shaping. The long-running NATS/S3 orchestration modules, including `agent`, `live`, `storage`, and `worker`, are verified by compile, lint, container build, and deployment smoke checks rather than unit coverage because they depend on live external services.

```bash
cargo fmt --all --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
docker buildx build --platform linux/arm64 -t intel-candidate-app:local /Volumes/WD/Developments/nangman-crypto/apps/intel-candidate-app
```
