ARG TARGETARCH
ARG TARGETVARIANT

FROM scratch AS base

WORKDIR /app

FROM base AS target-amd64
COPY --chmod=555 bin-releases/simon-x86_64-linux-musl /app/simon
ENTRYPOINT ["/app/simon"]

FROM base AS target-arm64
COPY --chmod=555 bin-releases/simon-aarch64-linux-musl /app/simon
ENTRYPOINT ["/app/simon"]

FROM base AS target-arm
COPY --chmod=555 bin-releases/simon-armv7-linux-musl /app/simon
ENTRYPOINT ["/app/simon"]

FROM base AS target-386
COPY --chmod=555 bin-releases/simon-i686-linux-musl /app/simon
ENTRYPOINT ["/app/simon"]

# Final target stage
FROM target-${TARGETARCH} AS target