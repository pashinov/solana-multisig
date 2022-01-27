pub use solana_program;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar,
};

mod entrypoint;
mod processor;

pub use self::processor::*;

solana_program::declare_id!("DADjz4TQMPbgZnY2PcjKANajNSdLyQgbUHwvj9DHRz9T");

pub fn get_associated_multisig_address_and_bump_seed(
    wallet_address: &Pubkey,
    program_id: &Pubkey,
    multisig_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&wallet_address.to_bytes(), &multisig_program_id.to_bytes()],
        program_id,
    )
}

pub fn get_associated_multisig_address(wallet_address: &Pubkey) -> Pubkey {
    get_associated_multisig_address_and_bump_seed(wallet_address, &id(), &multisig::id()).0
}

pub fn create_associated_multisig_account(
    funding_address: &Pubkey,
    wallet_address: &Pubkey,
    data: Vec<u8>,
) -> Instruction {
    let associated_account_address = get_associated_multisig_address(wallet_address);

    Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(*funding_address, true),
            AccountMeta::new(associated_account_address, false),
            AccountMeta::new_readonly(*wallet_address, false),
            AccountMeta::new_readonly(multisig::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data,
    }
}
