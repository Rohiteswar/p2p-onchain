# P2P Protocol — On-Chain + Off-Chain Backend

A fully on-chain orderbook DEX on Solana. Every order is a PDA, every fill is atomic, every state change emits a structured binary event. This repo contains the on-chain program, the indexer service, and the REST API service.

---

## Repository Structure

```
.
├── programs/
│   └── p2p-protocol/       On-chain Solana program (Pinocchio 0.11, no Anchor)
├── services/
│   ├── api/                Axum REST API — serves indexed data over HTTP
│   ├── indexer/            Real-time indexer — listens to Solana logs, writes to PostgreSQL
│   └── migrations/
│       └── 001_initial.sql PostgreSQL schema
├── Cargo.toml              Cargo workspace root
└── Cargo.lock
```

---

## On-Chain Program

- **Program ID:** `HazZUxenwxgxDumK5rt89mhXfffnVpA7Nyvx87kMts18`
- **Network:** Solana Devnet
- **Runtime:** SBPF via Pinocchio 0.11 — zero Anchor overhead, under 5,000 CU per instruction

### Instructions

| Instruction | Description |
|---|---|
| `create_market` | Initialise a new market with base/quote mints, tick size, lot size, fee bps |
| `place_order` | Place a Limit, IOC, FOK, or Post-Only order — stored as a PDA |
| `fill_order` | Atomically match maker and taker, release escrowed tokens |
| `cancel_order` | Cancel an open order and return escrowed tokens to owner |

### Events

Every instruction emits a structured binary event via `sol_log_data`:

| Discriminator | Event |
|---|---|
| `1` | `OrderPlaced` |
| `2` | `OrderFilled` |
| `3` | `OrderCancelled` |
| `4` | `OrderExpired` |
| `5` | `MarketCreated` |

---

## Services

Both services are part of the same Cargo workspace and share the root `Cargo.toml`.

### Indexer (`services/indexer`)

Connects to Solana via WebSocket (`logsSubscribe`), decodes binary events from program log data, and writes them to PostgreSQL in real time. On startup it also runs `getProgramAccounts` to back-fill any markets and orders that exist on-chain before the indexer started.

**Flow:**
```
Solana Devnet WS → logsSubscribe → decode binary event → upsert to PostgreSQL
                                                        ↑
                              getProgramAccounts (on startup, back-fill)
```

**Environment variables:**

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | required | PostgreSQL connection string |
| `SOLANA_RPC_URL` | `https://api.devnet.solana.com` | Solana HTTP RPC endpoint |
| `SOLANA_WS_URL` | `wss://api.devnet.solana.com` | Solana WebSocket endpoint |
| `PROGRAM_ID` | `HazZUxen...` | Program ID to subscribe to |

---

### API (`services/api`)

Stateless Axum HTTP server that reads from PostgreSQL and exposes indexed data as JSON. Fully CORS-enabled.

**Environment variables:**

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | required | PostgreSQL connection string |
| `API_PORT` | `3001` | Port to listen on |

**Endpoints:**

| Method | Path | Description |
|---|---|---|
| `GET` | `/health` | Health check |
| `GET` | `/stats` | Protocol-wide stats (markets, open orders, total fills) |
| `GET` | `/markets` | List all indexed markets |
| `GET` | `/markets/:address` | Market detail + full order book (bids + asks) |
| `GET` | `/markets/:address/orders` | All orders for a market |
| `GET` | `/markets/:address/events` | Events for a market |
| `GET` | `/orders/:address` | Single order by address |
| `GET` | `/events` | Global event feed (supports `?limit=`) |

---

## Database Schema

PostgreSQL — four tables, auto-migrated by the indexer on startup.

```
markets   address (PK), base_mint, quote_mint, vaults, tick_size, lot_size, fee_bps
orders    address (PK), market (FK), owner, price, qty, filled_qty, side, status
fills     id, signature, market, order_addr, maker, taker, fill_price, fill_qty
events    id, signature, market, event_type, data (JSONB), slot, timestamp
```

---

## Running Locally

**Prerequisites:** Rust 1.75+, PostgreSQL (or a NeonDB connection string)

**1. Set environment variables**

```bash
cp services/.env.example services/.env
# Edit services/.env with your DATABASE_URL and RPC URLs
```

**2. Run the indexer**

```bash
cargo run -p p2p-indexer
```

Migrations run automatically on first start.

**3. Run the API**

```bash
cargo run -p p2p-api
# Listening on http://0.0.0.0:3001
```

---

## Deployment (Railway)

Both services have Dockerfiles that build from the workspace root. The Docker build context must be the **repo root** (not a subdirectory) because of the Cargo workspace.

**API service**
- Dockerfile: `services/api/Dockerfile`
- Root directory: `/` (repo root)
- Required env vars: `DATABASE_URL`, `API_PORT`
- Expose a public domain for the HTTP endpoint

**Indexer service**
- Dockerfile: `services/indexer/Dockerfile`
- Root directory: `/` (repo root)
- Required env vars: `DATABASE_URL`, `SOLANA_RPC_URL`, `SOLANA_WS_URL`, `PROGRAM_ID`
- No public domain needed — outbound only

Local Docker build (from repo root):

```bash
docker build -f services/api/Dockerfile     -t p2p-api     .
docker build -f services/indexer/Dockerfile -t p2p-indexer .
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| On-chain | Rust, Pinocchio 0.11, SBPF |
| Indexer | Rust, Tokio, tokio-tungstenite, SQLx |
| API | Rust, Axum 0.8, SQLx, tower-http |
| Database | PostgreSQL (NeonDB) |
| Network | Solana Devnet |
