use anyhow::{Context, Result};

pub struct Config {
    pub database_url : String,
    pub rpc_url      : String,
    pub ws_url       : String,
    pub program_id   : String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL not set")?,
            rpc_url: std::env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.devnet.solana.com".into()),
            ws_url: std::env::var("SOLANA_WS_URL")
                .unwrap_or_else(|_| "wss://api.devnet.solana.com".into()),
            program_id: std::env::var("PROGRAM_ID")
                .unwrap_or_else(|_| "HazZUxenwxgxDumK5rt89mhXfffnVpA7Nyvx87kMts18".into()),
        })
    }
}
