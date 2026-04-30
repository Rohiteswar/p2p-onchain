use anyhow::{Context, Result};

pub struct Config {
    pub database_url : String,
    pub port         : u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL not set")?,
            port: std::env::var("API_PORT")
                .unwrap_or_else(|_| "3001".into())
                .parse()
                .unwrap_or(3001),
        })
    }
}
