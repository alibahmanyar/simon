ARG TARGETARCH
ARG TARGETVARIANT

# Final stage
FROM busybox:latest AS base

WORKDIR /app

FROM base AS amd64
COPY target/x86_64-unknown-linux-musl/release/simon /app/simon
ENTRYPOINT ["/app/simon"]

FROM base AS arm64
COPY target/aarch64-unknown-linux-musl/release/simon /app/simon
ENTRYPOINT ["/app/simon"]

FROM base AS arm
COPY target/armv7-unknown-linux-musl/release/simon /app/simon
ENTRYPOINT ["/app/simon"]

FROM base AS i386
COPY target/i686-unknown-linux-musl/release/simon /app/simon
ENTRYPOINT ["/app/simon"]

FROM ${TARGETARCH} AS target