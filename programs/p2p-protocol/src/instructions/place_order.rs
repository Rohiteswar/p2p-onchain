use pinocchio::{
    cpi::Signer,
    sysvars::{clock::Clock, Sysvar},
    AccountView, Address, ProgramResult,
};
use pinocchio::instruction::seeds;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Transfer;
use solana_program_error::ProgramError;

use crate::{
    error::P2PError,
    events::{emit_event, EVT_ORDER_PLACED, OrderPlacedEvent},
    state::{Market, Order, OrderType, Side, ORDER_DISCRIMINATOR, ORDER_SIZE},
};

const IX_DATA_SIZE: usize = 8 + 8 + 8 + 8 + 1 + 1 + 1;

struct PlaceOrderData {
    order_id:   u64,
    price:      u64,
    qty:        u64,
    expiry:     i64,
    side:       u8,
    order_type: u8,
    order_bump: u8,
}

impl PlaceOrderData {
    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < IX_DATA_SIZE { return None; }
        Some(Self {
            order_id:   u64::from_le_bytes(data[0..8].try_into().ok()?),
            price:      u64::from_le_bytes(data[8..16].try_into().ok()?),
            qty:        u64::from_le_bytes(data[16..24].try_into().ok()?),
            expiry:     i64::from_le_bytes(data[24..32].try_into().ok()?),
            side:       data[32],
            order_type: data[33],
            order_bump: data[34],
        })
    }
}

pub fn process(
    program_id: &Address,
    accounts:   &mut [AccountView],
    data:       &[u8],
) -> ProgramResult {
    let ix = PlaceOrderData::parse(data)
        .ok_or(ProgramError::InvalidInstructionData)?;

    let side = Side::from_u8(ix.side).ok_or(ProgramError::InvalidInstructionData)?;
    let order_type = OrderType::from_u8(ix.order_type).ok_or(ProgramError::InvalidInstructionData)?;

    if ix.qty == 0 {
        return Err(P2PError::ZeroOrderQty.into());
    }
    if ix.price == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let [owner, order_acc, market_acc, owner_token, vault, _token_program, _system_program, ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !owner.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !order_acc.is_writable() || !owner_token.is_writable() || !vault.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }
    if !order_acc.is_data_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let (tick_size, lot_size) = {
        let market_data = market_acc.try_borrow()?;
        let market = Market::load(&market_data);
        if !market.is_initialized() {
            return Err(P2PError::InvalidDiscriminator.into());
        }

        let expected_vault = match side {
            Side::Bid => market.quote_vault,
            Side::Ask => market.base_vault,
        };
        if vault.address() != &expected_vault {
            return Err(P2PError::VaultMismatch.into());
        }

        (market.tick_size, market.lot_size)
    };

    let escrow_amount = match side {
        Side::Bid => ix
            .qty
            .checked_mul(ix.price)
            .and_then(|v| v.checked_mul(tick_size))
            .ok_or(P2PError::Overflow)?,
        Side::Ask => ix
            .qty
            .checked_mul(lot_size)
            .ok_or(P2PError::Overflow)?,
    };

    let order_id_bytes = ix.order_id.to_le_bytes();
    let order_bump = ix.order_bump;
    {
        let bump_arr = [order_bump];
        let order_seeds = seeds!(
            b"order",
            market_acc.address().as_ref(),
            owner.address().as_ref(),
            order_id_bytes.as_ref(),
            bump_arr.as_ref()
        );
        CreateAccount::with_minimum_balance(
            owner,
            order_acc,
            ORDER_SIZE as u64,
            program_id,
            None,
        )?
        .invoke_signed(&[Signer::from(&order_seeds)])?;
    }

    Transfer::new(owner_token, vault, owner, escrow_amount).invoke()?;

    let clock = Clock::get()?;
    {
        let mut raw = order_acc.try_borrow_mut()?;
        let ok = Order {
            discriminator : ORDER_DISCRIMINATOR,
            market        : *market_acc.address(),
            owner         : *owner.address(),
            price         : ix.price,
            orig_qty      : ix.qty,
            filled_qty    : 0,
            expiry        : ix.expiry,
            created_at    : clock.unix_timestamp,
            side          : ix.side,
            order_type    : ix.order_type,
            bump          : order_bump,
        }
        .store(&mut raw);
        if !ok {
            return Err(ProgramError::AccountDataTooSmall);
        }
    }

    emit_event!(OrderPlacedEvent {
        discriminator: EVT_ORDER_PLACED,
        market:        *market_acc.address(),
        order:         *order_acc.address(),
        owner:         *owner.address(),
        side:          ix.side,
        order_type:    ix.order_type,
        price:         ix.price,
        qty:           ix.qty,
        expiry:        ix.expiry,
        created_at:    clock.unix_timestamp,
    });

    let _ = order_type;

    Ok(())
}
