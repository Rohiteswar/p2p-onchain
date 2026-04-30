use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{info, warn};

use crate::{config::Config, db};

const MARKET_SIZE: u64 = 190;
const ORDER_SIZE:  u64 = 116;

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: T,
}

#[derive(Deserialize)]
struct AccountEntry {
    pubkey:  String,
    account: AccountData,
}

#[derive(Deserialize)]
struct AccountData {
    data: (String, String), // (base64_data, encoding)
}

pub async fn sync_accounts(config: &Config, pool: &PgPool) -> Result<()> {
    info!("Syncing markets from chain...");
    let markets = fetch_program_accounts(config, MARKET_SIZE).await?;
    info!("Found {} market accounts", markets.len());
    for entry in &markets {
        let bytes = match STANDARD.decode(&entry.account.data.0) {
            Ok(b) => b,
            Err(e) => { warn!("base64 decode error for {}: {}", entry.pubkey, e); continue; }
        };
        if let Err(e) = ingest_market(pool, &entry.pubkey, &bytes).await {
            warn!("ingest market {}: {}", entry.pubkey, e);
        }
    }

    info!("Syncing orders from chain...");
    let orders = fetch_program_accounts(config, ORDER_SIZE).await?;
    info!("Found {} order accounts", orders.len());
    for entry in &orders {
        let bytes = match STANDARD.decode(&entry.account.data.0) {
            Ok(b) => b,
            Err(e) => { warn!("base64 decode error for {}: {}", entry.pubkey, e); continue; }
        };
        if let Err(e) = ingest_order(pool, &entry.pubkey, &bytes).await {
            warn!("ingest order {}: {}", entry.pubkey, e);
        }
    }

    Ok(())
}

async fn fetch_program_accounts(config: &Config, data_size: u64) -> Result<Vec<AccountEntry>> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getProgramAccounts",
        "params": [
            config.program_id,
            {
                "encoding": "base64",
                "filters": [{ "dataSize": data_size }]
            }
        ]
    });

    let resp: RpcResponse<Vec<AccountEntry>> = reqwest::Client::new()
        .post(&config.rpc_url)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    Ok(resp.result)
}

fn read_pubkey(data: &[u8], offset: usize) -> Option<String> {
    if offset + 32 > data.len() { return None; }
    Some(bs58::encode(&data[offset..offset + 32]).into_string())
}

fn read_u64_le(data: &[u8], offset: usize) -> Option<i64> {
    if offset + 8 > data.len() { return None; }
    let bytes: [u8; 8] = data[offset..offset + 8].try_into().ok()?;
    Some(u64::from_le_bytes(bytes) as i64)
}

fn read_u16_le(data: &[u8], offset: usize) -> Option<i32> {
    if offset + 2 > data.len() { return None; }
    let bytes: [u8; 2] = data[offset..offset + 2].try_into().ok()?;
    Some(u16::from_le_bytes(bytes) as i32)
}

fn read_i64_le(data: &[u8], offset: usize) -> Option<i64> {
    if offset + 8 > data.len() { return None; }
    let bytes: [u8; 8] = data[offset..offset + 8].try_into().ok()?;
    Some(i64::from_le_bytes(bytes))
}

async fn ingest_market(pool: &PgPool, address: &str, data: &[u8]) -> Result<()> {
    let base_mint   = read_pubkey(data, 8).ok_or_else(|| anyhow::anyhow!("bad base_mint"))?;
    let quote_mint  = read_pubkey(data, 40).ok_or_else(|| anyhow::anyhow!("bad quote_mint"))?;
    let base_vault  = read_pubkey(data, 72).ok_or_else(|| anyhow::anyhow!("bad base_vault"))?;
    let quote_vault = read_pubkey(data, 104).ok_or_else(|| anyhow::anyhow!("bad quote_vault"))?;
    let authority   = read_pubkey(data, 136).ok_or_else(|| anyhow::anyhow!("bad authority"))?;
    let tick_size   = read_u64_le(data, 168).unwrap_or(0);
    let lot_size    = read_u64_le(data, 176).unwrap_or(0);
    let fee_bps     = read_u16_le(data, 184).unwrap_or(0);

    db::upsert_market(pool, address, &base_mint, &quote_mint, &base_vault, &quote_vault,
        &authority, tick_size, lot_size, fee_bps, 0).await?;
    Ok(())
}

async fn ingest_order(pool: &PgPool, address: &str, data: &[u8]) -> Result<()> {
    let market     = read_pubkey(data, 8).ok_or_else(|| anyhow::anyhow!("bad market"))?;
    let owner      = read_pubkey(data, 40).ok_or_else(|| anyhow::anyhow!("bad owner"))?;
    let price      = read_u64_le(data, 72).unwrap_or(0);
    let orig_qty   = read_u64_le(data, 80).unwrap_or(0);
    let filled_qty = read_u64_le(data, 88).unwrap_or(0);
    let expiry     = read_i64_le(data, 96).unwrap_or(0);
    let placed_at  = read_i64_le(data, 104).unwrap_or(0);
    let side       = *data.get(112).unwrap_or(&0) as i32;
    let order_type = *data.get(113).unwrap_or(&0) as i32;

    let status = if filled_qty >= orig_qty { "filled" } else { "open" };

    db::upsert_order(pool, address, &market, &owner, price, orig_qty,
        filled_qty, side, order_type, status, expiry, placed_at).await?;
    Ok(())
}
