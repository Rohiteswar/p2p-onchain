use solana_address::Address;

pub const MARKET_DISCRIMINATOR: [u8; 8] = [0xAB, 0xCD, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06];

pub const MARKET_SIZE: usize = 8 + 32 + 32 + 32 + 32 + 32 + 8 + 8 + 2 + 1 + 1 + 1 + 1; // 190

const OFF_DISCRIMINATOR    : usize = 0;
const OFF_BASE_MINT        : usize = 8;
const OFF_QUOTE_MINT       : usize = 40;
const OFF_BASE_VAULT       : usize = 72;
const OFF_QUOTE_VAULT      : usize = 104;
const OFF_AUTHORITY        : usize = 136;
const OFF_TICK_SIZE        : usize = 168;
const OFF_LOT_SIZE         : usize = 176;
const OFF_FEE_BPS          : usize = 184;
const OFF_BUMP             : usize = 186;
const OFF_BASE_VAULT_BUMP  : usize = 187;
const OFF_QUOTE_VAULT_BUMP : usize = 188;

pub struct Market {
    pub discriminator    : [u8; 8],
    pub base_mint        : Address,
    pub quote_mint       : Address,
    pub base_vault       : Address,
    pub quote_vault      : Address,
    pub authority        : Address,
    pub tick_size        : u64,
    pub lot_size         : u64,
    pub fee_bps          : u16,
    pub bump             : u8,
    pub base_vault_bump  : u8,
    pub quote_vault_bump : u8,
}

impl Market {
    pub fn load(data: &[u8]) -> Self {
        assert!(data.len() >= MARKET_SIZE, "Market account too small");
        Self {
            discriminator    : data[OFF_DISCRIMINATOR..OFF_BASE_MINT].try_into().unwrap(),
            base_mint        : data[OFF_BASE_MINT..OFF_QUOTE_MINT].try_into().unwrap(),
            quote_mint       : data[OFF_QUOTE_MINT..OFF_BASE_VAULT].try_into().unwrap(),
            base_vault       : data[OFF_BASE_VAULT..OFF_QUOTE_VAULT].try_into().unwrap(),
            quote_vault      : data[OFF_QUOTE_VAULT..OFF_AUTHORITY].try_into().unwrap(),
            authority        : data[OFF_AUTHORITY..OFF_TICK_SIZE].try_into().unwrap(),
            tick_size        : u64::from_le_bytes(data[OFF_TICK_SIZE..OFF_LOT_SIZE].try_into().unwrap()),
            lot_size         : u64::from_le_bytes(data[OFF_LOT_SIZE..OFF_FEE_BPS].try_into().unwrap()),
            fee_bps          : u16::from_le_bytes(data[OFF_FEE_BPS..OFF_BUMP].try_into().unwrap()),
            bump             : data[OFF_BUMP],
            base_vault_bump  : data[OFF_BASE_VAULT_BUMP],
            quote_vault_bump : data[OFF_QUOTE_VAULT_BUMP],
        }
    }

    pub fn store(&self, data: &mut [u8]) -> bool {
        if data.len() < MARKET_SIZE { return false; }
        data[OFF_DISCRIMINATOR..OFF_BASE_MINT].copy_from_slice(&self.discriminator);
        data[OFF_BASE_MINT..OFF_QUOTE_MINT].copy_from_slice(self.base_mint.as_ref());
        data[OFF_QUOTE_MINT..OFF_BASE_VAULT].copy_from_slice(self.quote_mint.as_ref());
        data[OFF_BASE_VAULT..OFF_QUOTE_VAULT].copy_from_slice(self.base_vault.as_ref());
        data[OFF_QUOTE_VAULT..OFF_AUTHORITY].copy_from_slice(self.quote_vault.as_ref());
        data[OFF_AUTHORITY..OFF_TICK_SIZE].copy_from_slice(self.authority.as_ref());
        data[OFF_TICK_SIZE..OFF_LOT_SIZE].copy_from_slice(&self.tick_size.to_le_bytes());
        data[OFF_LOT_SIZE..OFF_FEE_BPS].copy_from_slice(&self.lot_size.to_le_bytes());
        data[OFF_FEE_BPS..OFF_BUMP].copy_from_slice(&self.fee_bps.to_le_bytes());
        data[OFF_BUMP]             = self.bump;
        data[OFF_BASE_VAULT_BUMP]  = self.base_vault_bump;
        data[OFF_QUOTE_VAULT_BUMP] = self.quote_vault_bump;
        true
    }

    pub fn is_initialized(&self) -> bool {
        self.discriminator == MARKET_DISCRIMINATOR
    }
}
