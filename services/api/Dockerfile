# ── Stage 1: build ────────────────────────────────────────────────────────
FROM rust:1.88-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests and services only
# programs/ is excluded — Pinocchio targets BPF and cannot compile for x86_64
COPY Cargo.toml Cargo.lock ./
COPY services/api/        services/api/
COPY services/indexer/    services/indexer/

# Remove the on-chain program from workspace members before building
RUN sed -i '/"programs\/p2p-protocol"/d' Cargo.toml

# Build only the API binary
RUN cargo build --release -p p2p-api

# ── Stage 2: runtime ──────────────────────────────────────────────────────
FROM debian:bookworm-slim

# ca-certificates needed for TLS connections to NeonDB
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/p2p-api /usr/local/bin/p2p-api

EXPOSE 3001

CMD ["p2p-api"]
