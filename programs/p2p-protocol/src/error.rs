use solana_program_error::ProgramError;

#[repr(u32)]
pub enum P2PError {
    InvalidDiscriminator     = 1,
    OrderFullyFilled         = 2,
    OrderExpired             = 3,
    OrderNotExpired          = 4,
    PriceMismatch            = 5,
    FillExceedsRemaining     = 6,
    ZeroFillQty              = 7,
    ZeroOrderQty             = 8,
    PostOnlyCrossed          = 9,
    UnauthorizedCanceller    = 10,
    MarketMismatch           = 11,
    VaultMismatch            = 12,
    InvalidAlignment         = 13,
    Overflow                 = 14,
}

impl From<P2PError> for ProgramError {
    fn from(e: P2PError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
