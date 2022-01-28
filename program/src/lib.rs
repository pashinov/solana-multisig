use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar;

mod error;
mod instruction;
mod processor;
mod state;
mod utils;

pub use self::error::*;
pub use self::instruction::*;
pub use self::processor::*;
pub use self::state::*;
pub use self::utils::*;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

solana_program::declare_id!("6BQQb1TXVvYrDND6BMTcm5bNqxhqJLCo9xMRksTW1yJ3");

pub fn check_program_account(program_id: &Pubkey) -> ProgramResult {
    if program_id != &id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}

pub fn get_associated_address_and_bump_seed(
    wallet_address: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&wallet_address.to_bytes()], program_id)
}

pub fn get_associated_address(wallet_address: &Pubkey) -> Pubkey {
    get_associated_address_and_bump_seed(wallet_address, &id()).0
}

pub fn create_associated_account(
    funding_address: &Pubkey,
    wallet_address: &Pubkey,
    data: Vec<u8>,
) -> Instruction {
    let associated_account_address = get_associated_address(wallet_address);

    Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(*funding_address, true),
            AccountMeta::new(associated_account_address, false),
            AccountMeta::new_readonly(*wallet_address, true),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data,
    }
}

pub fn create_transaction(
    funding_address: &Pubkey,
    wallet_address: &Pubkey,
    transaction_address: &Pubkey,
    recipient_address: &Pubkey,
    data: Vec<u8>,
) -> Instruction {
    let associated_account_address = get_associated_address(wallet_address);

    Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(*funding_address, true),
            AccountMeta::new(*transaction_address, true),
            AccountMeta::new(associated_account_address, false),
            AccountMeta::new_readonly(*recipient_address, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data,
    }
}

pub fn approve_transaction(
    funding_address: &Pubkey,
    multisig_address: &Pubkey,
    transaction_address: &Pubkey,
    recipient_address: &Pubkey,
    data: Vec<u8>,
) -> Instruction {
    Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(*funding_address, true),
            AccountMeta::new(*multisig_address, false),
            AccountMeta::new(*transaction_address, false),
            AccountMeta::new(*recipient_address, false),
        ],
        data,
    }
}
