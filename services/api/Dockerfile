# ── Stage 1: build ────────────────────────────────────────────────────────
FROM rust:1.85-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests first (layer-cache friendly)
COPY Cargo.toml Cargo.lock ./
COPY programs/            programs/
COPY services/api/        services/api/
COPY services/indexer/    services/indexer/

# Build only the API binary
RUN cargo build --release -p p2p-api

# ── Stage 2: runtime ──────────────────────────────────────────────────────
FROM debian:bookworm-slim

# ca-certificates needed for TLS connections to NeonDB
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/p2p-api /usr/local/bin/p2p-api

EXPOSE 3001

CMD ["p2p-api"]
