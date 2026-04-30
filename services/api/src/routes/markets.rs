use axum::{
    extract::Path,
    http::StatusCode,
    Extension, Json,
};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::PgPool;

#[derive(Serialize, sqlx::FromRow)]
pub struct MarketRow {
    address    : String,
    base_mint  : String,
    quote_mint : String,
    base_vault : String,
    quote_vault: String,
    authority  : String,
    tick_size  : i64,
    lot_size   : i64,
    fee_bps    : i32,
    created_at : i64,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct OrderRow {
    address    : String,
    owner      : String,
    price      : i64,
    orig_qty   : i64,
    filled_qty : i64,
    side       : i32,
    order_type : i32,
    expiry     : i64,
    placed_at  : i64,
}

pub async fn list(Extension(pool): Extension<PgPool>) -> Json<Vec<MarketRow>> {
    let rows = sqlx::query_as::<_, MarketRow>(
        "SELECT address, base_mint, quote_mint, base_vault, quote_vault, authority,
                tick_size, lot_size, fee_bps, created_at
         FROM markets ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Json(rows)
}

pub async fn get(
    Path(address): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>, StatusCode> {
    let market = sqlx::query_as::<_, MarketRow>(
        "SELECT address, base_mint, quote_mint, base_vault, quote_vault, authority,
                tick_size, lot_size, fee_bps, created_at
         FROM markets WHERE address = $1",
    )
    .bind(&address)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let asks = sqlx::query_as::<_, OrderRow>(
        "SELECT address, owner, price, orig_qty, filled_qty, side, order_type, expiry, placed_at
         FROM orders WHERE market = $1 AND side = 1 AND status = 'open'
         ORDER BY price ASC LIMIT 50",
    )
    .bind(&address)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let bids = sqlx::query_as::<_, OrderRow>(
        "SELECT address, owner, price, orig_qty, filled_qty, side, order_type, expiry, placed_at
         FROM orders WHERE market = $1 AND side = 0 AND status = 'open'
         ORDER BY price DESC LIMIT 50",
    )
    .bind(&address)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Ok(Json(json!({
        "market": market,
        "asks":   asks,
        "bids":   bids,
    })))
}
