use anyhow::Result;
use sqlx::PgPool;

pub async fn connect(url: &str) -> Result<PgPool> {
    Ok(PgPool::connect(url).await?)
}

pub async fn migrate(pool: &PgPool) -> Result<()> {
    sqlx::raw_sql(include_str!("../../migrations/001_initial.sql"))
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn upsert_market(
    pool       : &PgPool,
    address    : &str,
    base_mint  : &str,
    quote_mint : &str,
    base_vault : &str,
    quote_vault: &str,
    authority  : &str,
    tick_size  : i64,
    lot_size   : i64,
    fee_bps    : i32,
    ts         : i64,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO markets (address, base_mint, quote_mint, base_vault, quote_vault, authority,
                              tick_size, lot_size, fee_bps, created_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$10)
         ON CONFLICT (address) DO UPDATE SET updated_at = EXCLUDED.updated_at",
    )
    .bind(address).bind(base_mint).bind(quote_mint)
    .bind(base_vault).bind(quote_vault).bind(authority)
    .bind(tick_size).bind(lot_size).bind(fee_bps).bind(ts)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn upsert_order(
    pool      : &PgPool,
    address   : &str,
    market    : &str,
    owner     : &str,
    price     : i64,
    orig_qty  : i64,
    filled_qty: i64,
    side      : i32,
    order_type: i32,
    status    : &str,
    expiry    : i64,
    placed_at : i64,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO orders (address, market, owner, price, orig_qty, filled_qty,
                             side, order_type, status, expiry, placed_at, updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$11)
         ON CONFLICT (address) DO UPDATE
           SET filled_qty = EXCLUDED.filled_qty,
               status     = EXCLUDED.status,
               updated_at = EXCLUDED.updated_at",
    )
    .bind(address).bind(market).bind(owner)
    .bind(price).bind(orig_qty).bind(filled_qty)
    .bind(side).bind(order_type).bind(status)
    .bind(expiry).bind(placed_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn close_order(pool: &PgPool, address: &str, status: &str, ts: i64) -> Result<()> {
    sqlx::query(
        "UPDATE orders SET status = $2, updated_at = $3 WHERE address = $1",
    )
    .bind(address).bind(status).bind(ts)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fill_order(
    pool      : &PgPool,
    address   : &str,
    fill_qty  : i64,
    ts        : i64,
) -> Result<()> {
    sqlx::query(
        "UPDATE orders
         SET filled_qty = LEAST(orig_qty, filled_qty + $2),
             status     = CASE WHEN filled_qty + $2 >= orig_qty THEN 'filled' ELSE 'open' END,
             updated_at = $3
         WHERE address = $1",
    )
    .bind(address).bind(fill_qty).bind(ts)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_fill(
    pool       : &PgPool,
    signature  : &str,
    market     : &str,
    order_addr : &str,
    maker      : &str,
    taker      : &str,
    fill_price : i64,
    fill_qty   : i64,
    timestamp  : i64,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO fills (signature, market, order_addr, maker, taker, fill_price, fill_qty, timestamp)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(signature).bind(market).bind(order_addr)
    .bind(maker).bind(taker).bind(fill_price).bind(fill_qty).bind(timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_event(
    pool       : &PgPool,
    signature  : &str,
    market     : Option<&str>,
    event_type : i32,
    data       : serde_json::Value,
    slot       : i64,
    timestamp  : i64,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO events (signature, market, event_type, data, slot, timestamp)
         VALUES ($1,$2,$3,$4,$5,$6)",
    )
    .bind(signature).bind(market).bind(event_type)
    .bind(data).bind(slot).bind(timestamp)
    .execute(pool)
    .await?;
    Ok(())
}
