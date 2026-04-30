use axum::{Extension, Json};
use serde_json::{json, Value};
use sqlx::PgPool;

pub async fn handler(Extension(pool): Extension<PgPool>) -> Json<Value> {
    let ok = sqlx::query("SELECT 1").execute(&pool).await.is_ok();
    Json(json!({
        "status": if ok { "ok" } else { "degraded" },
        "db": ok,
        "version": "0.1.0",
    }))
}
