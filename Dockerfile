FROM --platform=$BUILDPLATFORM public.ecr.aws/docker/library/rust:1.94-bookworm AS builder

ARG TARGETARCH

WORKDIR /opt/nangman-crypto/intel-candidate

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates gcc-aarch64-linux-gnu libc6-dev-arm64-cross pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY . /opt/nangman-crypto/intel-candidate

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

RUN set -e; \
    case "${TARGETARCH}" in \
        arm64) \
            rustup target add aarch64-unknown-linux-gnu; \
            cargo build --release --target aarch64-unknown-linux-gnu; \
            binary_dir="target/aarch64-unknown-linux-gnu/release"; \
            ;; \
        amd64) \
            cargo build --release; \
            binary_dir="target/release"; \
            ;; \
        *) \
            echo "Unsupported Docker TARGETARCH: ${TARGETARCH}" >&2; \
            exit 1; \
            ;; \
    esac; \
    mkdir -p /opt/nangman-crypto/intel-candidate/build-output; \
    cp "${binary_dir}/intel-candidate-agent" /opt/nangman-crypto/intel-candidate/build-output/intel-candidate-agent; \
    cp "${binary_dir}/intel-candidate-worker" /opt/nangman-crypto/intel-candidate/build-output/intel-candidate-worker; \
    cp "${binary_dir}/intel-candidate-replay-worker" /opt/nangman-crypto/intel-candidate/build-output/intel-candidate-replay-worker

FROM gcr.io/distroless/cc-debian12:nonroot AS runtime

COPY --from=builder --chown=nonroot:nonroot \
    /opt/nangman-crypto/intel-candidate/build-output/intel-candidate-agent \
    /usr/local/bin/intel-candidate-agent
COPY --from=builder --chown=nonroot:nonroot \
    /opt/nangman-crypto/intel-candidate/build-output/intel-candidate-worker \
    /usr/local/bin/intel-candidate-worker
COPY --from=builder --chown=nonroot:nonroot \
    /opt/nangman-crypto/intel-candidate/build-output/intel-candidate-replay-worker \
    /usr/local/bin/intel-candidate-replay-worker
COPY --from=builder --chown=nonroot:nonroot \
    /opt/nangman-crypto/intel-candidate/policies/scoring-policy.v1.json \
    /opt/nangman-crypto/intel-candidate/policies/scoring-policy.v1.json

USER nonroot:nonroot

ENV AWS_SDK_LOAD_CONFIG=1

CMD ["/usr/local/bin/intel-candidate-agent"]
