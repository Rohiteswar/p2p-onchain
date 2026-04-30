use axum::{Extension, Json};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize)]
pub struct Stats {
    markets     : i64,
    open_orders : i64,
    total_fills : i64,
    total_volume: i64,
}

pub async fn handler(Extension(pool): Extension<PgPool>) -> Json<Stats> {
    let markets = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM markets")
        .fetch_one(&pool).await.unwrap_or(0);

    let open_orders = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM orders WHERE status = 'open'",
    )
    .fetch_one(&pool).await.unwrap_or(0);

    let total_fills = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM fills")
        .fetch_one(&pool).await.unwrap_or(0);

    let total_volume = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT COALESCE(SUM(fill_price * fill_qty), 0) FROM fills",
    )
    .fetch_one(&pool).await.unwrap_or(None).unwrap_or(0);

    Json(Stats { markets, open_orders, total_fills, total_volume })
}
