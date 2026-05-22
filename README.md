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
  --repair-max-keys-per-prefix 500
```

Repair scans are bounded by prefix, interval, and key count. They use the same deterministic scoring, idempotent S3 writes, NATS message IDs, and stale revision checks as live processing.

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

## Deployment defaults

Use ARM64 Fargate with `FARGATE_SPOT` as the preferred capacity provider. The task should connect to on-prem NATS through VPN or private routing, for example `nats://<private-nats-host>:4222`, with security groups/firewall rules limited to the required producers and consumers.

The example task definition is in:

```text
/Volumes/WD/Developments/nangman-crypto/apps/intel-candidate-app/ecs/task-definition.example.json
```

## Quality gate

Sonar coverage focuses on deterministic scoring, contracts, parsing, and artifact shaping. The long-running NATS/S3 orchestration modules are verified by compile, lint, container build, and deployment smoke checks rather than unit coverage because they depend on live external services.

```bash
cargo fmt --all --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
docker buildx build --platform linux/arm64 -t intel-candidate-app:local /Volumes/WD/Developments/nangman-crypto/apps/intel-candidate-app
```
