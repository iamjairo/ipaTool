# Multi-stage Dockerfile
# ── Stage 1: Frontend ─────────────────────────────────────────────────────────
FROM node:20-alpine AS frontend-builder

# Install pnpm (version must match the lockfile)
RUN npm install -g pnpm@10

# Set working directory
WORKDIR /app

# Copy dependency manifests
COPY package.json pnpm-lock.yaml ./

# Install dependencies (with BuildKit cache mount)
RUN --mount=type=cache,target=/root/.pnpm-store,id=pnpm_cache \
    pnpm install --frozen-lockfile

# Copy all source files
COPY . .

# Build the frontend
RUN pnpm run build && \
    ls -la dist/ && \
    echo "Frontend build completed successfully"

# ── Stage 2: Rust backend ─────────────────────────────────────────────────────
FROM rust:1.84-slim AS backend-builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libssl3 \
    perl \
    make \
    gcc \
    g++ \
    && rm -rf /var/lib/apt/lists/*

# Verify toolchain
RUN rustc --version && cargo --version

# Copy Cargo manifests
COPY server/Cargo.toml server/Cargo.lock ./

# Pre-build dependencies using placeholder source files
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "#![allow(dead_code)]" > src/lib.rs

RUN --mount=type=cache,target=/app/target,id=cargo_cache,sharing=locked \
    cargo build --release && \
    echo "Dependency pre-build completed"

# Remove placeholder source files
RUN rm -rf src

# Copy actual source code
COPY server/src ./src

# Build the final binary
RUN --mount=type=cache,target=/app/target,id=cargo_cache,sharing=locked \
    cargo build --release && \
    ls -lh target/release/server && \
    echo "Backend build completed successfully"

# Copy binary out of the cache mount so it is accessible in the next stage
RUN --mount=type=cache,target=/app/target,id=cargo_cache,sharing=locked \
    cp target/release/server /app/server && \
    ls -lh /app/server && \
    echo "Binary copied successfully"

# ── Stage 3: Production image ─────────────────────────────────────────────────
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy built frontend from Stage 1
COPY --from=frontend-builder /app/dist ./dist

# Copy built backend binary from Stage 2
COPY --from=backend-builder /app/server ./server

# Create data directory
RUN mkdir -p /app/data && chmod 777 /app/data

EXPOSE 8080

ENV PORT=8080
ENV RUST_LOG=info
ENV DATABASE_PATH=/app/data/ipa-webtool.db

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./server"]

