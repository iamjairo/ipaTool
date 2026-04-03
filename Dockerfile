# =============================================================================
# ipaTool - Production Multi-Stage Dockerfile
# Targets: Vue 3 frontend + Rust (Actix) backend -> single lean runtime image
# Compatible with: Docker BuildKit, Synology Container Manager (DS220+ / amd64)
# Usage:
#   docker build -t ipa-tool:latest .
#   docker build --platform linux/arm64 -t ipa-tool:arm64 .
# =============================================================================

# Pin digest-level for reproducibility; update deliberately.
ARG NODE_VERSION=20
ARG RUST_VERSION=1.86
ARG DEBIAN_CODENAME=bookworm

# =============================================================================
# Stage 1 - frontend-builder
# Build the Vue 3 / Vite frontend into a static dist/ bundle.
# =============================================================================
FROM node:${NODE_VERSION}-alpine AS frontend-builder

# pnpm version MUST match the engines.pnpm in package.json / pnpm-lock.yaml.
ARG PNPM_VERSION=10

RUN npm install -g pnpm@${PNPM_VERSION} --no-update-notifier

WORKDIR /app

# Copy manifests first to get a cache layer for installs.
COPY package.json pnpm-lock.yaml .npmrc* ./

# Mount the pnpm store as a build cache so re-builds skip re-downloading.
RUN --mount=type=cache,target=/root/.local/share/pnpm/store,id=pnpm_store \
    pnpm install --frozen-lockfile

# Copy source and build.
COPY . .

# Remove any remaining Chinese / non-ASCII characters from JS/TS/Vue source
# files before the build (sed -E strips CJK Unicode blocks U+4E00-U+9FFF,
# U+3400-U+4DBF, and common fullwidth punctuation U+FF00-U+FFEF).
RUN find src -type f \( -name "*.vue" -o -name "*.js" -o -name "*.ts" \) \
    -exec sed -i -E \
      's/[\x{4E00}-\x{9FFF}\x{3400}-\x{4DBF}\x{FF00}-\x{FFEF}]//g' {} +

RUN pnpm run build && \
    echo "--- Frontend dist contents ---" && ls -lh dist/

# =============================================================================
# Stage 2 - backend-builder
# Compile the Rust Actix server inside the server/ workspace crate.
# =============================================================================
FROM rust:${RUST_VERSION}-slim AS backend-builder

WORKDIR /build

# System dependencies for linking (openssl, pkg-config).
RUN apt-get update && apt-get install -y --no-install-recommends \
      pkg-config \
      libssl-dev \
      perl \
      make \
      gcc \
      g++ \
    && rm -rf /var/lib/apt/lists/*

# ---- Dependency pre-warming ------------------------------------------------
# Copy only the manifest files to build a cache layer of all crate deps.
# This layer is only invalidated when Cargo.toml / Cargo.lock changes.
COPY server/Cargo.toml server/Cargo.lock ./
RUN mkdir -p src && \
    echo 'fn main(){}' > src/main.rs && \
    echo '#![allow(dead_code)] pub fn lib(){}' > src/lib.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo_registry \
    --mount=type=cache,target=/build/target,id=cargo_target,sharing=locked \
    cargo build --release 2>&1 | tail -5

# Remove placeholder sources before copying real ones.
RUN rm -rf src

# ---- Real build ------------------------------------------------------------
COPY server/src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo_registry \
    --mount=type=cache,target=/build/target,id=cargo_target,sharing=locked \
    cargo build --release && \
    # Copy the binary OUT of the cache-mounted target dir while still inside
    # the same RUN layer so it survives to the next stage.
    cp target/release/server /tmp/server-bin

# =============================================================================
# Stage 3 - runtime
# Minimal Debian image; only runtime deps + the two artefacts from stages 1-2.
# =============================================================================
FROM debian:${DEBIAN_CODENAME}-slim AS runtime

LABEL org.opencontainers.image.title="IPA Tool"
LABEL org.opencontainers.image.description="IPA download and signing tool"
LABEL org.opencontainers.image.source="https://github.com/iamjairo/ipaTool"

# ca-certificates  - HTTPS outbound from the server
# curl             - used by HEALTHCHECK
# sqlite3          - runtime DB CLI (optional but handy for NAS debugging)
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates \
      curl \
      sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Non-root user for least-privilege operation.
RUN groupadd -r ipatool && useradd -r -g ipatool -d /app ipatool

WORKDIR /app

# Artefacts from previous stages.
COPY --from=frontend-builder /app/dist          ./dist
COPY --from=backend-builder  /tmp/server-bin    ./server

# Persistent storage directories; owner=ipatool so no chmod 777 needed.
RUN mkdir -p /app/data /app/downloads && \
    chown -R ipatool:ipatool /app

USER ipatool

EXPOSE 8080

ENV PORT=8080 \
    RUST_LOG=info \
    DATABASE_PATH=/app/data/ipa-webtool.db \
    STATIC_DIR=/app/dist

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -fsS http://localhost:${PORT}/health || exit 1

CMD ["./server"]
