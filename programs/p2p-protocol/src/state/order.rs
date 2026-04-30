use solana_address::Address;

pub const ORDER_DISCRIMINATOR: [u8; 8] = [0xAB, 0xCD, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60];

pub const ORDER_SIZE: usize = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1 + 1 + 1 + 1; // 116

const OFF_DISCRIMINATOR : usize = 0;
const OFF_MARKET        : usize = 8;
const OFF_OWNER         : usize = 40;
const OFF_PRICE         : usize = 72;
const OFF_ORIG_QTY      : usize = 80;
const OFF_FILLED_QTY    : usize = 88;
const OFF_EXPIRY        : usize = 96;
const OFF_CREATED_AT    : usize = 104;
const OFF_SIDE          : usize = 112;
const OFF_ORDER_TYPE    : usize = 113;
const OFF_BUMP          : usize = 114;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Bid = 0,
    Ask = 1,
}

impl Side {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v { 0 => Some(Side::Bid), 1 => Some(Side::Ask), _ => None }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit    = 0,
    IOC      = 1,
    FOK      = 2,
    PostOnly = 3,
}

impl OrderType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v { 0 => Some(Self::Limit), 1 => Some(Self::IOC), 2 => Some(Self::FOK), 3 => Some(Self::PostOnly), _ => None }
    }
}

pub struct Order {
    pub discriminator : [u8; 8],
    pub market        : Address,
    pub owner         : Address,
    pub price         : u64,
    pub orig_qty      : u64,
    pub filled_qty    : u64,
    pub expiry        : i64,
    pub created_at    : i64,
    pub side          : u8,
    pub order_type    : u8,
    pub bump          : u8,
}

impl Order {
    pub fn load(data: &[u8]) -> Self {
        assert!(data.len() >= ORDER_SIZE, "Order account too small");
        Self {
            discriminator : data[OFF_DISCRIMINATOR..OFF_MARKET].try_into().unwrap(),
            market        : data[OFF_MARKET..OFF_OWNER].try_into().unwrap(),
            owner         : data[OFF_OWNER..OFF_PRICE].try_into().unwrap(),
            price         : u64::from_le_bytes(data[OFF_PRICE..OFF_ORIG_QTY].try_into().unwrap()),
            orig_qty      : u64::from_le_bytes(data[OFF_ORIG_QTY..OFF_FILLED_QTY].try_into().unwrap()),
            filled_qty    : u64::from_le_bytes(data[OFF_FILLED_QTY..OFF_EXPIRY].try_into().unwrap()),
            expiry        : i64::from_le_bytes(data[OFF_EXPIRY..OFF_CREATED_AT].try_into().unwrap()),
            created_at    : i64::from_le_bytes(data[OFF_CREATED_AT..OFF_SIDE].try_into().unwrap()),
            side          : data[OFF_SIDE],
            order_type    : data[OFF_ORDER_TYPE],
            bump          : data[OFF_BUMP],
        }
    }

    pub fn store(&self, data: &mut [u8]) -> bool {
        if data.len() < ORDER_SIZE { return false; }
        data[OFF_DISCRIMINATOR..OFF_MARKET].copy_from_slice(&self.discriminator);
        data[OFF_MARKET..OFF_OWNER].copy_from_slice(self.market.as_ref());
        data[OFF_OWNER..OFF_PRICE].copy_from_slice(self.owner.as_ref());
        data[OFF_PRICE..OFF_ORIG_QTY].copy_from_slice(&self.price.to_le_bytes());
        data[OFF_ORIG_QTY..OFF_FILLED_QTY].copy_from_slice(&self.orig_qty.to_le_bytes());
        data[OFF_FILLED_QTY..OFF_EXPIRY].copy_from_slice(&self.filled_qty.to_le_bytes());
        data[OFF_EXPIRY..OFF_CREATED_AT].copy_from_slice(&self.expiry.to_le_bytes());
        data[OFF_CREATED_AT..OFF_SIDE].copy_from_slice(&self.created_at.to_le_bytes());
        data[OFF_SIDE]       = self.side;
        data[OFF_ORDER_TYPE] = self.order_type;
        data[OFF_BUMP]       = self.bump;
        true
    }

    pub fn is_initialized(&self) -> bool {
        self.discriminator == ORDER_DISCRIMINATOR
    }

    pub fn remaining_qty(&self) -> u64 {
        self.orig_qty.saturating_sub(self.filled_qty)
    }

    pub fn is_fully_filled(&self) -> bool {
        self.filled_qty >= self.orig_qty
    }

    pub fn is_expired(&self, clock_unix_ts: i64) -> bool {
        self.expiry != 0 && clock_unix_ts > self.expiry
    }

    pub fn side(&self) -> Option<Side> {
        Side::from_u8(self.side)
    }

    pub fn order_type(&self) -> Option<OrderType> {
        OrderType::from_u8(self.order_type)
    }
}
