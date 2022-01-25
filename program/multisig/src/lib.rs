use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

mod instruction;
mod processor;
mod state;
mod utils;

pub use self::instruction::*;
pub use self::state::*;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

solana_program::declare_id!("EANnrvYCDjBtQVoqYatjcvWARvvDhSXYjgynY1HMEVZD");

pub fn check_program_account(program_id: &Pubkey) -> ProgramResult {
    if program_id != &id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}
