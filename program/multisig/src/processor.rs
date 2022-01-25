use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack};
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{msg, system_instruction};

use crate::instruction::MultisigInstruction;
use crate::state::Account;
use crate::{Transaction, MAX_OWNERS};

pub struct Processor<'a, 'b> {
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'b>],
    data: &'a [u8],
}

impl<'a, 'b> Processor<'a, 'b> {
    pub fn new(program_id: &'a Pubkey, accounts: &'a [AccountInfo<'b>], data: &'a [u8]) -> Self {
        Self {
            program_id,
            accounts,
            data,
        }
    }

    pub fn process(&self) -> ProgramResult {
        let instruction = MultisigInstruction::unpack(self.data)?;

        match instruction {
            MultisigInstruction::InitializeAccount {
                seed,
                threshold,
                owners,
            } => {
                msg!("Instruction: InitializeAccount");
                Self::process_initialize_account(self.accounts, seed, threshold, owners)?;
            }
            MultisigInstruction::CreateTransaction { amount } => {
                msg!("Instruction: CreateTransaction");
                Self::process_create_transaction(self.program_id, self.accounts, amount)?;
            }
            MultisigInstruction::ApproveTransaction => {
                msg!("Instruction: ApproveTransaction");
                Self::process_approve_transaction(self.accounts)?;
            }
        };

        Ok(())
    }

    fn process_initialize_account(
        accounts: &[AccountInfo],
        seed: u8,
        threshold: u64,
        owners: Vec<Pubkey>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let new_account_info = next_account_info(account_info_iter)?;
        let wallet_account_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        let mut multisig_info = Account::unpack_unchecked(&new_account_info.data.borrow())?;

        if multisig_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let new_account_info_data_len = new_account_info.data_len();
        if !rent.is_exempt(new_account_info.lamports(), new_account_info_data_len) {
            return Err(ProgramError::AccountNotRentExempt);
        }

        if owners.len() > MAX_OWNERS {
            return Err(ProgramError::InvalidInstructionData);
        }

        multisig_info.is_initialized = true;
        multisig_info.seed = seed;
        multisig_info.threshold = threshold;
        multisig_info.wallet = wallet_account_info.key.clone();
        multisig_info.owners.extend(owners);

        Account::pack(multisig_info, &mut new_account_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_create_transaction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let wallet_account_info = next_account_info(account_info_iter)?;
        let transaction_account_info = next_account_info(account_info_iter)?;
        let multisig_account_info = next_account_info(account_info_iter)?;
        let recipient_account_info = next_account_info(account_info_iter)?;
        let system_program_account = next_account_info(account_info_iter)?;

        if !wallet_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let multisig_info = Account::unpack_unchecked(&multisig_account_info.data.borrow())?;
        if !multisig_info.is_initialized {
            return Err(ProgramError::UninitializedAccount);
        }

        if multisig_info.wallet != *wallet_account_info.key {
            return Err(ProgramError::IllegalOwner);
        }

        let transaction = Transaction {
            multisig: multisig_account_info.key.clone(),
            recipient: recipient_account_info.key.clone(),
            amount,
            signers: multisig_info
                .owners
                .into_iter()
                .map(|owner| (owner, false))
                .collect(),
        };

        invoke(
            &system_instruction::create_account(
                wallet_account_info.key,
                transaction_account_info.key,
                Rent::get()?.minimum_balance(Transaction::LEN),
                Transaction::LEN as u64,
                program_id,
            ),
            &[
                wallet_account_info.clone(),
                transaction_account_info.clone(),
                system_program_account.clone(),
            ],
        )?;

        invoke(
            &system_instruction::assign(transaction_account_info.key, program_id),
            &[
                transaction_account_info.clone(),
                system_program_account.clone(),
            ],
        )?;

        Transaction::pack(transaction, &mut transaction_account_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_approve_transaction(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let wallet_account_info = next_account_info(account_info_iter)?;
        let multisig_account_info = next_account_info(account_info_iter)?;
        let transaction_account_info = next_account_info(account_info_iter)?;
        let recipient_account_info = next_account_info(account_info_iter)?;

        if !wallet_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut transaction_info =
            Transaction::unpack_unchecked(&transaction_account_info.data.borrow())?;

        transaction_info
            .signers
            .iter_mut()
            .position(|(key, is_signed)| {
                if key == wallet_account_info.key {
                    *is_signed = true;
                    true
                } else {
                    false
                }
            })
            .ok_or(ProgramError::IllegalOwner)?;

        let multisig_info = Account::unpack_unchecked(&multisig_account_info.data.borrow())?;

        let signers_count = transaction_info
            .signers
            .iter()
            .filter(|(_, is_signed)| *is_signed)
            .count() as u64;

        if multisig_info.threshold > signers_count {
            **multisig_account_info.try_borrow_mut_lamports()? -= transaction_info.amount;
            **recipient_account_info.try_borrow_mut_lamports()? += transaction_info.amount;
        }

        Ok(())
    }
}
