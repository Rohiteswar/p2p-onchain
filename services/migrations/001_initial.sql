CREATE TABLE IF NOT EXISTS markets (
    address         TEXT    PRIMARY KEY,
    base_mint       TEXT    NOT NULL,
    quote_mint      TEXT    NOT NULL,
    base_vault      TEXT    NOT NULL,
    quote_vault     TEXT    NOT NULL,
    authority       TEXT    NOT NULL,
    tick_size       BIGINT  NOT NULL DEFAULT 0,
    lot_size        BIGINT  NOT NULL DEFAULT 0,
    fee_bps         INTEGER NOT NULL DEFAULT 0,
    created_at      BIGINT  NOT NULL DEFAULT 0,
    updated_at      BIGINT  NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS orders (
    address     TEXT    PRIMARY KEY,
    market      TEXT    NOT NULL REFERENCES markets(address) ON DELETE CASCADE,
    owner       TEXT    NOT NULL,
    price       BIGINT  NOT NULL,
    orig_qty    BIGINT  NOT NULL,
    filled_qty  BIGINT  NOT NULL DEFAULT 0,
    side        INTEGER NOT NULL,
    order_type  INTEGER NOT NULL,
    status      TEXT    NOT NULL DEFAULT 'open',
    expiry      BIGINT  NOT NULL DEFAULT 0,
    placed_at   BIGINT  NOT NULL DEFAULT 0,
    updated_at  BIGINT  NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS fills (
    id          BIGSERIAL PRIMARY KEY,
    signature   TEXT    NOT NULL,
    market      TEXT    NOT NULL,
    order_addr  TEXT    NOT NULL,
    maker       TEXT    NOT NULL,
    taker       TEXT    NOT NULL,
    fill_price  BIGINT  NOT NULL,
    fill_qty    BIGINT  NOT NULL,
    timestamp   BIGINT  NOT NULL
);

CREATE TABLE IF NOT EXISTS events (
    id          BIGSERIAL PRIMARY KEY,
    signature   TEXT    NOT NULL,
    market      TEXT,
    event_type  INTEGER NOT NULL,
    data        JSONB   NOT NULL,
    slot        BIGINT  NOT NULL DEFAULT 0,
    timestamp   BIGINT  NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_orders_market  ON orders(market);
CREATE INDEX IF NOT EXISTS idx_orders_owner   ON orders(owner);
CREATE INDEX IF NOT EXISTS idx_orders_status  ON orders(status);
CREATE INDEX IF NOT EXISTS idx_fills_market   ON fills(market);
CREATE INDEX IF NOT EXISTS idx_events_market  ON events(market);
CREATE INDEX IF NOT EXISTS idx_events_type    ON events(event_type);
