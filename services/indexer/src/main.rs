mod config;
mod db;
mod decoder;
mod listener;
mod sync;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/../.env")).ok();
    dotenvy::dotenv().ok();

    let config = config::Config::from_env()?;
    let pool   = db::connect(&config.database_url).await?;

    info!("Running migrations...");
    db::migrate(&pool).await?;

    info!("Initial account sync...");
    sync::sync_accounts(&config, &pool).await?;

    info!("Starting real-time listener...");
    listener::listen(&config, &pool).await?;

    Ok(())
}
