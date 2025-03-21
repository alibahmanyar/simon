ARG TARGETARCH
ARG TARGETVARIANT

FROM scratch AS base

WORKDIR /app

FROM base AS target-amd64
COPY target/x86_64-unknown-linux-musl/release/simon /app/simon
ENTRYPOINT ["/app/simon"]

FROM base AS target-arm64
COPY target/aarch64-unknown-linux-musl/release/simon /app/simon
ENTRYPOINT ["/app/simon"]

FROM base AS target-arm
COPY target/armv7-unknown-linux-musleabihf/release/simon /app/simon
ENTRYPOINT ["/app/simon"]

FROM base AS target-386
COPY target/i686-unknown-linux-musl/release/simon /app/simon
ENTRYPOINT ["/app/simon"]

# Final target stage
FROM target-${TARGETARCH} AS target