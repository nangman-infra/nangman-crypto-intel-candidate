# intel-candidate-app

`intel-candidate-app`은 구조화된 intel packet을 연구 후보로 승격할지 판단하는 deterministic gate다.

## Live

```bash
intel-candidate-worker \
  --nats-url nats://REPLACE_WITH_S2S_NATS_HOST:4222 \
  --input-s3-bucket nangman-crypto-dev-intel-structuring-l1-962214 \
  --output-s3-bucket nangman-crypto-dev-intel-candidate-962214 \
  --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-962214 \
  --policy-file /opt/nangman-crypto/intel-candidate/policies/scoring-policy.v1.json
```

## Replay

정책, schema, app version, market reference 정책이 바뀌면 NATS에 남아 있는 pointer만 다시 읽으면 부족하다.

S3 durable structured packet prefix를 다시 스캔해야 한다.

```bash
intel-candidate-replay-worker \
  --nats-url nats://REPLACE_WITH_S2S_NATS_HOST:4222 \
  --input-s3-bucket nangman-crypto-dev-intel-structuring-l1-962214 \
  --output-s3-bucket nangman-crypto-dev-intel-candidate-962214 \
  --market-l1-s3-bucket nangman-crypto-dev-market-ingest-l1-962214 \
  --policy-file /opt/nangman-crypto/intel-candidate/policies/scoring-policy.v1.json \
  --replay-input-prefix structured-intel-packet/schema=structured_intel_packet_v1/ \
  --replay-report-prefix candidate-replay-report
```

Replay worker는 모든 입력 key를 `processed`, `skipped`, `failed` 중 하나로 report에 남기고, report를 candidate S3 bucket에 저장한다.

## Contract

이 앱의 입력 source of truth는 `structured_pointer_v1` NATS pointer와 S3의 `structured_intel_packet_v1` 객체다. NATS는 pointer bus이고, canonical payload는 S3에 있다.

```text
schemas/*.schema.json
asyncapi/nats.asyncapi.json
```

Worker는 입력 pointer schema, pointer가 가리키는 payload schema, S3 객체 sha256을 검증한 뒤 scoring을 시작한다. 출력은 S3에 screening/event bundle/hypothesis state를 먼저 쓰고, `intel_candidate_pointer_v1` pointer를 NATS에 publish ack 받은 뒤 input message를 ack한다.

## Quality gate

```bash
cargo fmt --all --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
docker buildx build --platform linux/arm64 -t intel-candidate-app:local /Volumes/WD/Developments/nangman-crypto/apps/intel-candidate-app
```
