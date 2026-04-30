mod config;
mod routes;

use anyhow::Result;
use axum::{routing::get, Extension, Router};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/../.env")).ok();
    dotenvy::dotenv().ok();

    let config = config::Config::from_env()?;
    let pool   = PgPool::connect(&config.database_url).await?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health",                      get(routes::health::handler))
        .route("/stats",                       get(routes::stats::handler))
        .route("/markets",                     get(routes::markets::list))
        .route("/markets/{address}",           get(routes::markets::get))
        .route("/markets/{address}/orders",    get(routes::orders::by_market))
        .route("/markets/{address}/events",    get(routes::events::by_market))
        .route("/orders/{address}",            get(routes::orders::get))
        .route("/events",                      get(routes::events::global))
        .layer(cors)
        .layer(Extension(pool));

    let addr = format!("0.0.0.0:{}", config.port);
    info!("API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
