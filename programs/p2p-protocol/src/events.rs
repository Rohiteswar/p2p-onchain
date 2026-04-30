use solana_address::Address;

pub const EVT_ORDER_PLACED:    u8 = 1;
pub const EVT_ORDER_FILLED:    u8 = 2;
pub const EVT_ORDER_CANCELLED: u8 = 3;
pub const EVT_ORDER_EXPIRED:   u8 = 4;
pub const EVT_MARKET_CREATED:  u8 = 5;

#[repr(C)]
pub struct OrderPlacedEvent {
    pub discriminator: u8,
    pub market:        Address,
    pub order:         Address,
    pub owner:         Address,
    pub side:          u8,
    pub order_type:    u8,
    pub price:         u64,
    pub qty:           u64,
    pub expiry:        i64,
    pub created_at:    i64,
}

#[repr(C)]
pub struct OrderFilledEvent {
    pub discriminator: u8,
    pub market:        Address,
    pub order:         Address,
    pub maker:         Address,
    pub taker:         Address,
    pub fill_price:    u64,
    pub fill_qty:      u64,
    pub timestamp:     i64,
}

#[repr(C)]
pub struct OrderCancelledEvent {
    pub discriminator: u8,
    pub market:        Address,
    pub order:         Address,
    pub owner:         Address,
    pub timestamp:     i64,
}

#[repr(C)]
pub struct OrderExpiredEvent {
    pub discriminator: u8,
    pub market:        Address,
    pub order:         Address,
    pub owner:         Address,
    pub timestamp:     i64,
}

#[repr(C)]
pub struct MarketCreatedEvent {
    pub discriminator: u8,
    pub market:        Address,
    pub base_mint:     Address,
    pub quote_mint:    Address,
    pub tick_size:     u64,
    pub lot_size:      u64,
    pub fee_bps:       u16,
    pub timestamp:     i64,
}

macro_rules! emit_event {
    ($event:expr) => {{
        #[cfg(target_os = "solana")]
        unsafe {
            let sol_bytes: [u64; 2] = [
                &$event as *const _ as u64,
                core::mem::size_of_val(&$event) as u64,
            ];
            pinocchio::syscalls::sol_log_data(
                sol_bytes.as_ptr() as *const u8,
                1,
            );
        }
        let _ = &$event;
    }};
}

pub(crate) use emit_event;
