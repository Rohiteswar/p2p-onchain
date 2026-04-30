#![no_std]

use pinocchio::{
    default_allocator, nostd_panic_handler, program_entrypoint,
    AccountView, Address, ProgramResult,
};
use solana_program_error::ProgramError;

pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

pub mod ix {
    pub const CREATE_MARKET:  u8 = 0;
    pub const PLACE_ORDER:    u8 = 1;
    pub const FILL_ORDER:     u8 = 2;
    pub const CANCEL_ORDER:   u8 = 3;
    pub const EXPIRE_ORDER:   u8 = 4;
}

program_entrypoint!(process_instruction);
default_allocator!();
nostd_panic_handler!();

pub fn process_instruction(
    program_id: &Address,
    accounts:   &mut [AccountView],
    data:       &[u8],
) -> ProgramResult {
    let (discriminator, rest) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match *discriminator {
        ix::CREATE_MARKET => instructions::create_market::process(program_id, accounts, rest),
        ix::PLACE_ORDER   => instructions::place_order::process(program_id, accounts, rest),
        ix::FILL_ORDER    => instructions::fill_order::process(program_id, accounts, rest),
        ix::CANCEL_ORDER  => instructions::cancel_order::process(program_id, accounts, rest),
        ix::EXPIRE_ORDER  => instructions::expire_order::process(program_id, accounts, rest),
        _                 => Err(ProgramError::InvalidInstructionData),
    }
}
