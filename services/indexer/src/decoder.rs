use anyhow::{bail, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;

pub const DISC_ORDER_PLACED    : u8 = 1;
pub const DISC_ORDER_FILLED    : u8 = 2;
pub const DISC_ORDER_CANCELLED : u8 = 3;
pub const DISC_ORDER_EXPIRED   : u8 = 4;
pub const DISC_MARKET_CREATED  : u8 = 5;

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum P2PEvent {
    MarketCreated(MarketCreatedEvent),
    OrderPlaced(OrderPlacedEvent),
    OrderFilled(OrderFilledEvent),
    OrderCancelled(OrderCancelledEvent),
    OrderExpired(OrderExpiredEvent),
}

impl P2PEvent {
    pub fn discriminator(&self) -> i32 {
        match self {
            Self::MarketCreated(_)  => DISC_MARKET_CREATED  as i32,
            Self::OrderPlaced(_)    => DISC_ORDER_PLACED    as i32,
            Self::OrderFilled(_)    => DISC_ORDER_FILLED    as i32,
            Self::OrderCancelled(_) => DISC_ORDER_CANCELLED as i32,
            Self::OrderExpired(_)   => DISC_ORDER_EXPIRED   as i32,
        }
    }

    pub fn market(&self) -> &str {
        match self {
            Self::MarketCreated(e)  => &e.market,
            Self::OrderPlaced(e)    => &e.market,
            Self::OrderFilled(e)    => &e.market,
            Self::OrderCancelled(e) => &e.market,
            Self::OrderExpired(e)   => &e.market,
        }
    }

    pub fn timestamp(&self) -> i64 {
        match self {
            Self::MarketCreated(e)  => e.timestamp,
            Self::OrderPlaced(e)    => e.created_at,
            Self::OrderFilled(e)    => e.timestamp,
            Self::OrderCancelled(e) => e.timestamp,
            Self::OrderExpired(e)   => e.timestamp,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct MarketCreatedEvent {
    pub market    : String,
    pub base_mint : String,
    pub quote_mint: String,
    pub tick_size : i64,
    pub lot_size  : i64,
    pub fee_bps   : u16,
    pub timestamp : i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct OrderPlacedEvent {
    pub market     : String,
    pub order      : String,
    pub owner      : String,
    pub side       : u8,
    pub order_type : u8,
    pub price      : i64,
    pub qty        : i64,
    pub expiry     : i64,
    pub created_at : i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct OrderFilledEvent {
    pub market     : String,
    pub order      : String,
    pub maker      : String,
    pub taker      : String,
    pub fill_price : i64,
    pub fill_qty   : i64,
    pub timestamp  : i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct OrderCancelledEvent {
    pub market    : String,
    pub order     : String,
    pub owner     : String,
    pub timestamp : i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct OrderExpiredEvent {
    pub market    : String,
    pub order     : String,
    pub owner     : String,
    pub timestamp : i64,
}

fn pubkey(data: &[u8], offset: usize) -> Result<String> {
    if offset + 32 > data.len() {
        bail!("buffer too short: need {} got {}", offset + 32, data.len());
    }
    Ok(bs58::encode(&data[offset..offset + 32]).into_string())
}

fn u64_le(data: &[u8], offset: usize) -> Result<i64> {
    if offset + 8 > data.len() {
        bail!("buffer too short for u64 at {}", offset);
    }
    let bytes: [u8; 8] = data[offset..offset + 8].try_into()?;
    Ok(u64::from_le_bytes(bytes) as i64)
}

fn i64_le(data: &[u8], offset: usize) -> Result<i64> {
    if offset + 8 > data.len() {
        bail!("buffer too short for i64 at {}", offset);
    }
    let bytes: [u8; 8] = data[offset..offset + 8].try_into()?;
    Ok(i64::from_le_bytes(bytes))
}

fn u16_le(data: &[u8], offset: usize) -> Result<u16> {
    if offset + 2 > data.len() {
        bail!("buffer too short for u16 at {}", offset);
    }
    let bytes: [u8; 2] = data[offset..offset + 2].try_into()?;
    Ok(u16::from_le_bytes(bytes))
}

pub fn decode_event(data: &[u8]) -> Result<P2PEvent> {
    if data.is_empty() {
        bail!("empty event buffer");
    }
    match data[0] {
        DISC_MARKET_CREATED  => Ok(P2PEvent::MarketCreated(decode_market_created(data)?)),
        DISC_ORDER_PLACED    => Ok(P2PEvent::OrderPlaced(decode_order_placed(data)?)),
        DISC_ORDER_FILLED    => Ok(P2PEvent::OrderFilled(decode_order_filled(data)?)),
        DISC_ORDER_CANCELLED => Ok(P2PEvent::OrderCancelled(decode_order_cancelled(data)?)),
        DISC_ORDER_EXPIRED   => Ok(P2PEvent::OrderExpired(decode_order_expired(data)?)),
        d                    => bail!("unknown discriminator: {}", d),
    }
}

pub fn parse_events_from_logs(logs: &[String]) -> Vec<P2PEvent> {
    const PREFIX: &str = "Program data: ";
    logs.iter()
        .filter_map(|log| {
            let encoded = log.strip_prefix(PREFIX)?;
            let bytes = STANDARD.decode(encoded.trim()).ok()?;
            decode_event(&bytes).ok()
        })
        .collect()
}

fn decode_market_created(d: &[u8]) -> Result<MarketCreatedEvent> {
    Ok(MarketCreatedEvent {
        market:     pubkey(d, 1)?,
        base_mint:  pubkey(d, 33)?,
        quote_mint: pubkey(d, 65)?,
        tick_size:  u64_le(d, 97)?,
        lot_size:   u64_le(d, 105)?,
        fee_bps:    u16_le(d, 113)?,
        timestamp:  i64_le(d, 115)?,
    })
}

fn decode_order_placed(d: &[u8]) -> Result<OrderPlacedEvent> {
    Ok(OrderPlacedEvent {
        market:     pubkey(d, 1)?,
        order:      pubkey(d, 33)?,
        owner:      pubkey(d, 65)?,
        side:       d[97],
        order_type: d[98],
        price:      u64_le(d, 99)?,
        qty:        u64_le(d, 107)?,
        expiry:     i64_le(d, 115)?,
        created_at: i64_le(d, 123)?,
    })
}

fn decode_order_filled(d: &[u8]) -> Result<OrderFilledEvent> {
    Ok(OrderFilledEvent {
        market:     pubkey(d, 1)?,
        order:      pubkey(d, 33)?,
        maker:      pubkey(d, 65)?,
        taker:      pubkey(d, 97)?,
        fill_price: u64_le(d, 129)?,
        fill_qty:   u64_le(d, 137)?,
        timestamp:  i64_le(d, 145)?,
    })
}

fn decode_order_cancelled(d: &[u8]) -> Result<OrderCancelledEvent> {
    Ok(OrderCancelledEvent {
        market:    pubkey(d, 1)?,
        order:     pubkey(d, 33)?,
        owner:     pubkey(d, 65)?,
        timestamp: i64_le(d, 97)?,
    })
}

fn decode_order_expired(d: &[u8]) -> Result<OrderExpiredEvent> {
    Ok(OrderExpiredEvent {
        market:    pubkey(d, 1)?,
        order:     pubkey(d, 33)?,
        owner:     pubkey(d, 65)?,
        timestamp: i64_le(d, 97)?,
    })
}
