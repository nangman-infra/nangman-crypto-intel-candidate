# syntax=docker/dockerfile:1

FROM public.ecr.aws/docker/library/rust:1.94-bookworm AS builder

WORKDIR /opt/nangman-crypto/intel-candidate

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY . /opt/nangman-crypto/intel-candidate

RUN cargo build --release

FROM public.ecr.aws/docker/library/debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --shell /usr/sbin/nologin intel-candidate \
    && chown -R intel-candidate:intel-candidate /home/intel-candidate

COPY --from=builder \
    /opt/nangman-crypto/intel-candidate/target/release/intel-candidate-agent \
    /usr/local/bin/intel-candidate-agent
COPY --from=builder \
    /opt/nangman-crypto/intel-candidate/target/release/intel-candidate-worker \
    /usr/local/bin/intel-candidate-worker
COPY --from=builder \
    /opt/nangman-crypto/intel-candidate/target/release/intel-candidate-replay-worker \
    /usr/local/bin/intel-candidate-replay-worker
COPY --from=builder \
    /opt/nangman-crypto/intel-candidate/policies/scoring-policy.v1.json \
    /opt/nangman-crypto/intel-candidate/policies/scoring-policy.v1.json

USER intel-candidate

ENV AWS_SDK_LOAD_CONFIG=1

CMD ["/usr/local/bin/intel-candidate-agent"]
