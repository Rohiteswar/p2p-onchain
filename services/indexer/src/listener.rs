use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use sqlx::PgPool;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

use crate::{config::Config, db, decoder::{parse_events_from_logs, P2PEvent}};

#[derive(Deserialize)]
struct LogsNotification {
    method: Option<String>,
    params: Option<LogsParams>,
}

#[derive(Deserialize)]
struct LogsParams {
    result: LogsResult,
}

#[derive(Deserialize)]
struct LogsResult {
    context: Context,
    value:   LogsValue,
}

#[derive(Deserialize)]
struct Context {
    slot: i64,
}

#[derive(Deserialize)]
struct LogsValue {
    signature: String,
    err:       Option<serde_json::Value>,
    logs:      Vec<String>,
}

pub async fn listen(config: &Config, pool: &PgPool) -> Result<()> {
    loop {
        info!("Connecting to Solana WebSocket: {}", config.ws_url);
        match run_listener(config, pool).await {
            Ok(()) => info!("WebSocket closed cleanly, reconnecting..."),
            Err(e) => {
                error!("WebSocket error: {e}, reconnecting in 5s...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn run_listener(config: &Config, pool: &PgPool) -> Result<()> {
    let (ws, _) = connect_async(&config.ws_url).await?;
    let (mut write, mut read) = ws.split();

    let subscribe = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "logsSubscribe",
        "params": [
            { "mentions": [config.program_id] },
            { "commitment": "confirmed" }
        ]
    });
    write.send(Message::Text(subscribe.to_string())).await?;
    info!("Subscribed to program logs: {}", config.program_id);

    // Send a ping every 30s — Solana devnet drops idle connections after ~60s
    let mut ping_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    ping_interval.tick().await; // skip the immediate first tick

    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_message(&text, pool).await {
                            warn!("handle_message error: {e}");
                        }
                    }
                    // Respond to server-initiated pings
                    Some(Ok(Message::Ping(p))) => { write.send(Message::Pong(p)).await?; }
                    // Ignore pong replies to our own pings
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => return Err(e.into()),
                    _ => {}
                }
            }
            // Client-initiated keepalive ping
            _ = ping_interval.tick() => {
                write.send(Message::Ping(vec![])).await?;
            }
        }
    }
    Ok(())
}

async fn handle_message(text: &str, pool: &PgPool) -> Result<()> {
    let notification: LogsNotification = serde_json::from_str(text)?;

    let Some(method) = &notification.method else { return Ok(()); };
    if method != "logsNotification" { return Ok(()); }

    let Some(params) = notification.params else { return Ok(()); };
    let value = &params.result.value;
    let slot  = params.result.context.slot;

    if value.err.is_some() { return Ok(()); }

    let events = parse_events_from_logs(&value.logs);
    for event in events {
        if let Err(e) = process_event(&event, &value.signature, slot, pool).await {
            warn!("process_event error: {e}");
        }
    }
    Ok(())
}

async fn process_event(event: &P2PEvent, sig: &str, slot: i64, pool: &PgPool) -> Result<()> {
    let ts   = event.timestamp();
    let disc = event.discriminator();
    let mkt  = event.market();
    let data = serde_json::to_value(event)?;

    db::insert_event(pool, sig, Some(mkt), disc, data, slot, ts).await?;

    match event {
        P2PEvent::MarketCreated(e) => {
            db::upsert_market(
                pool, &e.market, &e.base_mint, &e.quote_mint,
                "", "", "", e.tick_size, e.lot_size, e.fee_bps as i32, e.timestamp,
            ).await?;
        }
        P2PEvent::OrderPlaced(e) => {
            db::upsert_order(
                pool, &e.order, &e.market, &e.owner,
                e.price, e.qty, 0,
                e.side as i32, e.order_type as i32,
                "open", e.expiry, e.created_at,
            ).await?;
        }
        P2PEvent::OrderFilled(e) => {
            db::fill_order(pool, &e.order, e.fill_qty, e.timestamp).await?;
            db::insert_fill(
                pool, sig, &e.market, &e.order,
                &e.maker, &e.taker, e.fill_price, e.fill_qty, e.timestamp,
            ).await?;
        }
        P2PEvent::OrderCancelled(e) => {
            db::close_order(pool, &e.order, "cancelled", e.timestamp).await?;
        }
        P2PEvent::OrderExpired(e) => {
            db::close_order(pool, &e.order, "expired", e.timestamp).await?;
        }
    }
    Ok(())
}
