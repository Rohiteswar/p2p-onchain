# ── Stage 1: build ────────────────────────────────────────────────────────
FROM rust:1.85-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests first (layer-cache friendly)
COPY Cargo.toml Cargo.lock ./
COPY programs/            programs/
COPY services/indexer/    services/indexer/
COPY services/api/        services/api/

# Build only the indexer binary
RUN cargo build --release -p p2p-indexer

# ── Stage 2: runtime ──────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/p2p-indexer /usr/local/bin/p2p-indexer

CMD ["p2p-indexer"]
