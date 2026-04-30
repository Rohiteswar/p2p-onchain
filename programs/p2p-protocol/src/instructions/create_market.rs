use pinocchio::{
    cpi::Signer,
    sysvars::{clock::Clock, Sysvar},
    AccountView, Address, ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::InitializeAccount3;
use solana_program_error::ProgramError;

use crate::{
    events::{emit_event, MarketCreatedEvent, EVT_MARKET_CREATED},
    state::{Market, MARKET_DISCRIMINATOR, MARKET_SIZE},
};

use pinocchio::instruction::seeds;

const IX_DATA_SIZE: usize = 8 + 8 + 2 + 1 + 1 + 1;

struct CreateMarketData {
    tick_size   : u64,
    lot_size    : u64,
    fee_bps     : u16,
    market_bump : u8,
    bv_bump     : u8,
    qv_bump     : u8,
}

impl CreateMarketData {
    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < IX_DATA_SIZE { return None; }
        Some(Self {
            tick_size   : u64::from_le_bytes(data[0..8].try_into().ok()?),
            lot_size    : u64::from_le_bytes(data[8..16].try_into().ok()?),
            fee_bps     : u16::from_le_bytes(data[16..18].try_into().ok()?),
            market_bump : data[18],
            bv_bump     : data[19],
            qv_bump     : data[20],
        })
    }
}

pub fn process(
    program_id: &Address,
    accounts:   &mut [AccountView],
    data:       &[u8],
) -> ProgramResult {
    let ix = CreateMarketData::parse(data)
        .ok_or(ProgramError::InvalidInstructionData)?;
    if ix.tick_size == 0 || ix.lot_size == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let [payer, market_acc, base_mint, quote_mint, base_vault, quote_vault, token_program, _system_program, ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !payer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !market_acc.is_writable() || !base_vault.is_writable() || !quote_vault.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }
    if !market_acc.is_data_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    {
        let market_bump_arr = [ix.market_bump];
        let market_seeds = seeds!(
            b"market",
            base_mint.address().as_ref(),
            quote_mint.address().as_ref(),
            market_bump_arr.as_ref()
        );
        CreateAccount::with_minimum_balance(
            payer,
            market_acc,
            MARKET_SIZE as u64,
            program_id,
            None,
        )?
        .invoke_signed(&[Signer::from(&market_seeds)])?;
    }

    const TOKEN_ACCOUNT_SIZE: u64 = 165;
    {
        let bv_bump_arr = [ix.bv_bump];
        let bv_seeds = seeds!(
            b"vault",
            market_acc.address().as_ref(),
            b"base",
            bv_bump_arr.as_ref()
        );
        CreateAccount::with_minimum_balance(
            payer,
            base_vault,
            TOKEN_ACCOUNT_SIZE,
            token_program.address(),
            None,
        )?
        .invoke_signed(&[Signer::from(&bv_seeds)])?;
    }
    InitializeAccount3 {
        account: base_vault,
        mint:    base_mint,
        owner:   market_acc.address(),
    }
    .invoke()?;

    {
        let qv_bump_arr = [ix.qv_bump];
        let qv_seeds = seeds!(
            b"vault",
            market_acc.address().as_ref(),
            b"quote",
            qv_bump_arr.as_ref()
        );
        CreateAccount::with_minimum_balance(
            payer,
            quote_vault,
            TOKEN_ACCOUNT_SIZE,
            token_program.address(),
            None,
        )?
        .invoke_signed(&[Signer::from(&qv_seeds)])?;
    }
    InitializeAccount3 {
        account: quote_vault,
        mint:    quote_mint,
        owner:   market_acc.address(),
    }
    .invoke()?;

    {
        let mut raw = market_acc.try_borrow_mut()?;
        let ok = Market {
            discriminator    : MARKET_DISCRIMINATOR,
            base_mint        : *base_mint.address(),
            quote_mint       : *quote_mint.address(),
            base_vault       : *base_vault.address(),
            quote_vault      : *quote_vault.address(),
            authority        : *payer.address(),
            tick_size        : ix.tick_size,
            lot_size         : ix.lot_size,
            fee_bps          : ix.fee_bps,
            bump             : ix.market_bump,
            base_vault_bump  : ix.bv_bump,
            quote_vault_bump : ix.qv_bump,
        }
        .store(&mut raw);
        if !ok {
            return Err(ProgramError::AccountDataTooSmall);
        }
    }

    let ts = Clock::get()?.unix_timestamp;
    emit_event!(MarketCreatedEvent {
        discriminator: EVT_MARKET_CREATED,
        market:        *market_acc.address(),
        base_mint:     *base_mint.address(),
        quote_mint:    *quote_mint.address(),
        tick_size:     ix.tick_size,
        lot_size:      ix.lot_size,
        fee_bps:       ix.fee_bps,
        timestamp:     ts,
    });

    Ok(())
}
