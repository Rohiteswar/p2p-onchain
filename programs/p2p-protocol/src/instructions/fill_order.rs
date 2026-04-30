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
    events::{emit_event, EVT_ORDER_FILLED, OrderFilledEvent},
    state::{Market, Order, Side, ORDER_DISCRIMINATOR},
};

const IX_DATA_SIZE: usize = 8;

struct FillOrderData {
    fill_qty: u64,
}

impl FillOrderData {
    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < IX_DATA_SIZE { return None; }
        Some(Self {
            fill_qty: u64::from_le_bytes(data[0..8].try_into().ok()?),
        })
    }
}

pub fn process(
    _program_id: &Address,
    accounts:    &mut [AccountView],
    data:        &[u8],
) -> ProgramResult {
    let ix = FillOrderData::parse(data)
        .ok_or(ProgramError::InvalidInstructionData)?;
    if ix.fill_qty == 0 {
        return Err(P2PError::ZeroFillQty.into());
    }

    let [taker, order_acc, market_acc, taker_base, taker_quote, maker_base, maker_quote, base_vault, quote_vault, _token_program, ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !taker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (maker_key, order_side, fill_price, escrow_release, taker_pays) = {
        let order_data = order_acc.try_borrow()?;
        let order = Order::load(&order_data);

        if order.discriminator != ORDER_DISCRIMINATOR {
            return Err(P2PError::InvalidDiscriminator.into());
        }
        if order.market != *market_acc.address() {
            return Err(P2PError::MarketMismatch.into());
        }
        if order.is_fully_filled() {
            return Err(P2PError::OrderFullyFilled.into());
        }

        let clock = Clock::get()?;
        if order.is_expired(clock.unix_timestamp) {
            return Err(P2PError::OrderExpired.into());
        }
        if ix.fill_qty > order.remaining_qty() {
            return Err(P2PError::FillExceedsRemaining.into());
        }

        let side = order.side().ok_or(ProgramError::InvalidAccountData)?;

        let market_data = market_acc.try_borrow()?;
        let market = Market::load(&market_data);

        let (escrow_release, taker_pays) = match side {
            Side::Ask => {
                let base_out = ix.fill_qty.checked_mul(market.lot_size).ok_or(P2PError::Overflow)?;
                let quote_in = ix.fill_qty.checked_mul(order.price).and_then(|v| v.checked_mul(market.tick_size)).ok_or(P2PError::Overflow)?;
                (base_out, quote_in)
            }
            Side::Bid => {
                let quote_out = ix.fill_qty.checked_mul(order.price).and_then(|v| v.checked_mul(market.tick_size)).ok_or(P2PError::Overflow)?;
                let base_in   = ix.fill_qty.checked_mul(market.lot_size).ok_or(P2PError::Overflow)?;
                (quote_out, base_in)
            }
        };

        (order.owner, side, order.price, escrow_release, taker_pays)
    };

    let (market_bump, market_base_mint, market_quote_mint) = {
        let market_data = market_acc.try_borrow()?;
        let market = Market::load(&market_data);

        if base_vault.address() != &market.base_vault {
            return Err(P2PError::VaultMismatch.into());
        }
        if quote_vault.address() != &market.quote_vault {
            return Err(P2PError::VaultMismatch.into());
        }

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
        let mkt_signer = Signer::from(&market_seeds);

        let mkt_bump_arr2 = [market_bump];
        let market_seeds2 = seeds!(
            b"market",
            market_base_mint.as_ref(),
            market_quote_mint.as_ref(),
            mkt_bump_arr2.as_ref()
        );
        let mkt_signer2 = Signer::from(&market_seeds2);

        match order_side {
            Side::Ask => {
                Transfer::new(base_vault, taker_base, market_acc, escrow_release)
                    .invoke_signed(&[mkt_signer])?;
                Transfer::new(taker_quote, quote_vault, taker, taker_pays).invoke()?;
                Transfer::new(quote_vault, maker_quote, market_acc, taker_pays)
                    .invoke_signed(&[mkt_signer2])?;
            }
            Side::Bid => {
                Transfer::new(quote_vault, taker_quote, market_acc, escrow_release)
                    .invoke_signed(&[mkt_signer])?;
                Transfer::new(taker_base, base_vault, taker, taker_pays).invoke()?;
                Transfer::new(base_vault, maker_base, market_acc, taker_pays)
                    .invoke_signed(&[mkt_signer2])?;
            }
        }
    }

    let fully_filled = {
        let mut raw = order_acc.try_borrow_mut()?;
        let mut order = Order::load(&raw);
        order.filled_qty = order.filled_qty.saturating_add(ix.fill_qty);
        let ff = order.is_fully_filled();
        order.store(&mut raw);
        ff
    };

    if fully_filled {
        let rent = order_acc.lamports();
        taker.set_lamports(taker.lamports().checked_add(rent).ok_or(P2PError::Overflow)?);
        order_acc.set_lamports(0);
        order_acc.close()?;
    }

    let ts = Clock::get()?.unix_timestamp;
    emit_event!(OrderFilledEvent {
        discriminator: EVT_ORDER_FILLED,
        market:        *market_acc.address(),
        order:         *order_acc.address(),
        maker:         maker_key,
        taker:         *taker.address(),
        fill_price,
        fill_qty:      ix.fill_qty,
        timestamp:     ts,
    });

    Ok(())
}
