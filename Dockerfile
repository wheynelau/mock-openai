# Stage 1: Build
FROM alpine:3.20 AS builder

ARG VERSION
ARG REPO=wheynelau/mock-openai
ARG TARGETARCH

RUN apk add --no-cache curl xz ca-certificates jq

# Rather than building again, use the pre-built binaries from GitHub Releases
RUN set -eux; \
    case "${TARGETARCH}" in \
        amd64) RUST_TARGET="x86_64-unknown-linux-musl" ;; \
        arm64) RUST_TARGET="aarch64-unknown-linux-musl" ;; \
        *) echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
    esac; \
    if [ "${VERSION}" = "latest" ]; then \
        VERSION=$(curl -s https://api.github.com/repos/${REPO}/releases/latest | jq -r '.tag_name'); \
    fi; \
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/mock-openai-${RUST_TARGET}.tar.xz"; \
    CHECKSUM_URL="https://github.com/${REPO}/releases/download/${VERSION}/mock-openai-${RUST_TARGET}.tar.xz.sha256"; \
    echo "Downloading from: ${DOWNLOAD_URL}"; \
    curl -fsSL "${DOWNLOAD_URL}" -o /tmp/mock-openai.tar.xz; \
    curl -fsSL "${CHECKSUM_URL}" -o /tmp/mock-openai.tar.xz.sha256; \
    cd /tmp && sha256sum -c mock-openai.tar.xz.sha256; \
    tar -xJf /tmp/mock-openai.tar.xz -C /tmp; \
    chmod +x /tmp/mock-openai

# Runtime image
FROM gcr.io/distroless/static-debian13:nonroot

COPY --from=builder /tmp/mock-openai /mock-openai

EXPOSE 8079

USER nonroot:nonroot

ENTRYPOINT ["/mock-openai"]
