use crate::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let funder_info = next_account_info(account_info_iter)?;
    let associated_account_info = next_account_info(account_info_iter)?;
    let wallet_account_info = next_account_info(account_info_iter)?;
    let multisig_program_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let rent_sysvar_info = next_account_info(account_info_iter)?;

    let (associated_address, bump_seed) = Pubkey::find_program_address(
        &[
            &wallet_account_info.key.to_bytes(),
            &multisig_program_info.key.to_bytes(),
        ],
        program_id,
    );

    if associated_address != *associated_account_info.key {
        msg!("Error: Associated address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }

    let associated_account_signer_seeds: &[&[_]] = &[
        &wallet_account_info.key.to_bytes(),
        &multisig_program_info.key.to_bytes(),
        &[bump_seed],
    ];

    let rent = &Rent::from_account_info(rent_sysvar_info)?;
    let required_lamports = rent
        .minimum_balance(multisig::Account::LEN)
        .saturating_sub(associated_account_info.lamports());

    if required_lamports > 0 {
        msg!(
            "Transfer {} lamports to the associated token account",
            required_lamports
        );
        invoke(
            &system_instruction::transfer(
                funder_info.key,
                associated_account_info.key,
                required_lamports,
            ),
            &[
                funder_info.clone(),
                associated_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    msg!("Allocate space for the associated token account");
    invoke_signed(
        &system_instruction::allocate(associated_account_info.key, multisig::Account::LEN as u64),
        &[associated_account_info.clone(), system_program_info.clone()],
        &[associated_account_signer_seeds],
    )?;

    msg!("Assign the associated account to the multisig program");
    invoke_signed(
        &system_instruction::assign(associated_account_info.key, multisig_program_info.key),
        &[associated_account_info.clone(), system_program_info.clone()],
        &[associated_account_signer_seeds],
    )?;

    msg!("Initialize the associated multisig account");
    invoke(
        &multisig::initialize_account(
            multisig_program_info.key,
            associated_account_info.key,
            wallet_account_info.key,
            data,
        )?,
        &[associated_account_info.clone(), rent_sysvar_info.clone()],
    )
}
