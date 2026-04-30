use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, sqlx::FromRow)]
pub struct EventRow {
    id         : i64,
    signature  : String,
    market     : Option<String>,
    event_type : i32,
    data       : serde_json::Value,
    slot       : i64,
    timestamp  : i64,
}

#[derive(Deserialize)]
pub struct EventQuery {
    limit: Option<i64>,
}

pub async fn global(
    Query(q): Query<EventQuery>,
    Extension(pool): Extension<PgPool>,
) -> Json<Vec<EventRow>> {
    let limit = q.limit.unwrap_or(50).min(200);
    let rows = sqlx::query_as::<_, EventRow>(
        "SELECT id, signature, market, event_type, data, slot, timestamp
         FROM events ORDER BY id DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Json(rows)
}

pub async fn by_market(
    Path(market): Path<String>,
    Query(q): Query<EventQuery>,
    Extension(pool): Extension<PgPool>,
) -> Json<Vec<EventRow>> {
    let limit = q.limit.unwrap_or(50).min(200);
    let rows = sqlx::query_as::<_, EventRow>(
        "SELECT id, signature, market, event_type, data, slot, timestamp
         FROM events WHERE market = $1 ORDER BY id DESC LIMIT $2",
    )
    .bind(&market)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Json(rows)
}
