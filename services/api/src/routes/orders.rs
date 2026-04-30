use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, sqlx::FromRow)]
pub struct OrderRow {
    address    : String,
    market     : String,
    owner      : String,
    price      : i64,
    orig_qty   : i64,
    filled_qty : i64,
    side       : i32,
    order_type : i32,
    status     : String,
    expiry     : i64,
    placed_at  : i64,
    updated_at : i64,
}

#[derive(Deserialize)]
pub struct ByOwnerQuery {
    owner: Option<String>,
    limit: Option<i64>,
}

pub async fn get(
    Path(address): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<OrderRow>, StatusCode> {
    sqlx::query_as::<_, OrderRow>(
        "SELECT address, market, owner, price, orig_qty, filled_qty,
                side, order_type, status, expiry, placed_at, updated_at
         FROM orders WHERE address = $1",
    )
    .bind(&address)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)
    .map(Json)
}

pub async fn by_market(
    Path(market): Path<String>,
    Query(q): Query<ByOwnerQuery>,
    Extension(pool): Extension<PgPool>,
) -> Json<Vec<OrderRow>> {
    let limit = q.limit.unwrap_or(100).min(500);
    let rows = sqlx::query_as::<_, OrderRow>(
        "SELECT address, market, owner, price, orig_qty, filled_qty,
                side, order_type, status, expiry, placed_at, updated_at
         FROM orders WHERE market = $1 AND status = 'open'
         ORDER BY price DESC LIMIT $2",
    )
    .bind(&market)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Json(rows)
}
