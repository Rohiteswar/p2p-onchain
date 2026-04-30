use pinocchio::{
    cpi::Signer,
    sysvars::{clock::Clock, Sysvar},
    AccountView, Address, ProgramResult,
};
use pinocchio::instruction::seeds;
use pinocchio_token::instructions::Transfer;
use solana_program_error::ProgramError;

use crate::{
    error::P2PError,
    events::{emit_event, EVT_ORDER_CANCELLED, OrderCancelledEvent},
    state::{Market, Order, Side, ORDER_DISCRIMINATOR},
};

pub fn process(
    _program_id: &Address,
    accounts:    &mut [AccountView],
    _data:       &[u8],
) -> ProgramResult {
    let [owner, order_acc, market_acc, owner_token, vault, _token_program, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !owner.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !order_acc.is_writable() || !owner_token.is_writable() || !vault.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }

    let (remaining_qty, refund_amount) = {
        let order_data = order_acc.try_borrow()?;
        let order = Order::load(&order_data);

        if order.discriminator != ORDER_DISCRIMINATOR {
            return Err(P2PError::InvalidDiscriminator.into());
        }
        if order.market != *market_acc.address() {
            return Err(P2PError::MarketMismatch.into());
        }
        if &order.owner != owner.address() {
            return Err(P2PError::UnauthorizedCanceller.into());
        }

        let side = order.side().ok_or(ProgramError::InvalidAccountData)?;

        let market_data = market_acc.try_borrow()?;
        let market = Market::load(&market_data);

        let refund = match side {
            Side::Bid => order
                .remaining_qty()
                .checked_mul(order.price)
                .and_then(|v| v.checked_mul(market.tick_size))
                .ok_or(P2PError::Overflow)?,
            Side::Ask => order
                .remaining_qty()
                .checked_mul(market.lot_size)
                .ok_or(P2PError::Overflow)?,
        };

        let expected_vault = match side {
            Side::Bid => market.quote_vault,
            Side::Ask => market.base_vault,
        };
        if vault.address() != &expected_vault {
            return Err(P2PError::VaultMismatch.into());
        }

        (order.remaining_qty(), refund)
    };

    if remaining_qty == 0 {
        return Err(P2PError::OrderFullyFilled.into());
    }

    let (market_bump, market_base_mint, market_quote_mint) = {
        let market_data = market_acc.try_borrow()?;
        let market = Market::load(&market_data);
        (market.bump, market.base_mint, market.quote_mint)
    };

    {
        let mkt_bump_arr = [market_bump];
        let market_seeds = seeds!(
            b"market",
            market_base_mint.as_ref(),
            market_quote_mint.as_ref(),
            mkt_bump_arr.as_ref()
        );
        Transfer::new(vault, owner_token, market_acc, refund_amount)
            .invoke_signed(&[Signer::from(&market_seeds)])?;
    }

    let rent = order_acc.lamports();
    owner.set_lamports(owner.lamports().checked_add(rent).ok_or(P2PError::Overflow)?);
    order_acc.set_lamports(0);
    order_acc.close()?;

    let ts = Clock::get()?.unix_timestamp;
    emit_event!(OrderCancelledEvent {
        discriminator: EVT_ORDER_CANCELLED,
        market:        *market_acc.address(),
        order:         *order_acc.address(),
        owner:         *owner.address(),
        timestamp:     ts,
    });

    Ok(())
}
