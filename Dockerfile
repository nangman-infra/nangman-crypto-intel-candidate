FROM public.ecr.aws/docker/library/rust:1.94-bookworm AS builder

WORKDIR /opt/nangman-crypto/intel-candidate

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY . /opt/nangman-crypto/intel-candidate

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:nonroot AS runtime

COPY --from=builder --chown=nonroot:nonroot \
    /opt/nangman-crypto/intel-candidate/target/release/intel-candidate-agent \
    /usr/local/bin/intel-candidate-agent
COPY --from=builder --chown=nonroot:nonroot \
    /opt/nangman-crypto/intel-candidate/target/release/intel-candidate-worker \
    /usr/local/bin/intel-candidate-worker
COPY --from=builder --chown=nonroot:nonroot \
    /opt/nangman-crypto/intel-candidate/target/release/intel-candidate-replay-worker \
    /usr/local/bin/intel-candidate-replay-worker
COPY --from=builder --chown=nonroot:nonroot \
    /opt/nangman-crypto/intel-candidate/policies/scoring-policy.v1.json \
    /opt/nangman-crypto/intel-candidate/policies/scoring-policy.v1.json

USER nonroot:nonroot

ENV AWS_SDK_LOAD_CONFIG=1

CMD ["/usr/local/bin/intel-candidate-agent"]
