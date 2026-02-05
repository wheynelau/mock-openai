# Stage 1: Prepare binary
FROM alpine:3.20 AS builder

ARG VERSION
ARG REPO=wheynelau/mock-openai
ARG TARGETARCH

RUN apk add --no-cache curl xz

# Download pre-built binaries from GitHub Releases
RUN set -eux; \
    case "${TARGETARCH}" in \
        amd64) RUST_TARGET="x86_64-unknown-linux-musl" ;; \
        arm64) RUST_TARGET="aarch64-unknown-linux-musl" ;; \
        *) echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
    esac; \
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/mock-openai-${RUST_TARGET}.tar.xz"; \
    echo "Downloading from: ${DOWNLOAD_URL}"; \
    curl -fsSL "${DOWNLOAD_URL}" -o /tmp/mock-openai.tar.xz; \
    tar -xJf /tmp/mock-openai.tar.xz -C /tmp; \
    BINARY=$(find /tmp -name "mock-openai" -type f | head -1); \
    mv "$BINARY" /mock-openai; \
    chmod +x /mock-openai

# Runtime image
FROM gcr.io/distroless/static-debian13:nonroot

COPY --from=builder /mock-openai /mock-openai

EXPOSE 8079

USER nonroot:nonroot

ENTRYPOINT ["/mock-openai"]
