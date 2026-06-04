ARG TARGETARCH
ARG TARGETVARIANT

FROM alpine:latest AS builder
FROM scratch AS base

WORKDIR /app
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

FROM base AS target-amd64
COPY --chmod=555 bin-releases/simon-x86_64-linux-musl /app/simon
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
ENTRYPOINT ["/app/simon"]

FROM base AS target-arm64
COPY --chmod=555 bin-releases/simon-aarch64-linux-musl /app/simon
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
ENTRYPOINT ["/app/simon"]

FROM base AS target-arm
COPY --chmod=555 bin-releases/simon-armv7-linux-musl /app/simon
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
ENTRYPOINT ["/app/simon"]

FROM base AS target-386
COPY --chmod=555 bin-releases/simon-i686-linux-musl /app/simon
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
ENTRYPOINT ["/app/simon"]

FROM base AS target-riscv64
COPY --chmod=555 bin-releases/simon-riscv64-linux-musl /app/simon
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
ENTRYPOINT ["/app/simon"]

# Final target stage
FROM target-${TARGETARCH} AS target