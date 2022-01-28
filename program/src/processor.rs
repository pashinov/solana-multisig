use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{msg, system_instruction};

use crate::instruction::MultisigInstruction;
use crate::state::Account;
use crate::{MultisigError, Transaction, MAX_SIGNERS, MAX_TRANSACTIONS, MIN_SIGNERS};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = MultisigInstruction::unpack(instruction_data)?;

        match instruction {
            MultisigInstruction::CreateAccount { threshold, owners } => {
                msg!("Instruction: CreateAccount");
                Self::process_create_account(program_id, accounts, threshold, owners)?;
            }
            MultisigInstruction::CreateTransaction { amount } => {
                msg!("Instruction: CreateTransaction");
                Self::process_create_transaction(program_id, accounts, amount)?;
            }
            MultisigInstruction::ApproveTransaction => {
                msg!("Instruction: ApproveTransaction");
                Self::process_approve_transaction(accounts)?;
            }
        };

        Ok(())
    }

    fn process_create_account(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        threshold: u32,
        owners: Vec<Pubkey>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let funder_account_info = next_account_info(account_info_iter)?;
        let multisig_account_info = next_account_info(account_info_iter)?;
        let wallet_account_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let rent_sysvar_info = next_account_info(account_info_iter)?;

        if !(funder_account_info.is_signer && wallet_account_info.is_signer) {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (pda, nonce) =
            Pubkey::find_program_address(&[&wallet_account_info.key.to_bytes()], program_id);

        if pda != *multisig_account_info.key {
            msg!("Error: Associated address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }

        if owners.len() > MAX_SIGNERS || owners.len() < MIN_SIGNERS {
            return Err(MultisigError::CustodianLimit.into());
        }

        let multisig_account_data = Account {
            is_initialized: true,
            threshold,
            owners,
            pending_transactions: vec![],
            frozen_amount: 0,
        };

        let rent = &Rent::from_account_info(rent_sysvar_info)?;
        let required_lamports = rent
            .minimum_balance(Account::LEN)
            .max(1)
            .saturating_sub(multisig_account_info.lamports());

        if required_lamports > 0 {
            msg!(
                "Transfer {} lamports to the associated multisig account",
                required_lamports
            );
            invoke(
                &system_instruction::transfer(
                    funder_account_info.key,
                    multisig_account_info.key,
                    required_lamports,
                ),
                &[
                    funder_account_info.clone(),
                    multisig_account_info.clone(),
                    system_program_info.clone(),
                ],
            )?;
        }

        msg!("Allocate space for the associated multisig account");
        invoke_signed(
            &system_instruction::allocate(multisig_account_info.key, Account::LEN as u64),
            &[multisig_account_info.clone(), system_program_info.clone()],
            &[&[&wallet_account_info.key.to_bytes()[..], &[nonce]]],
        )?;

        msg!("Assign the associated account to the multisig program");
        invoke_signed(
            &system_instruction::assign(multisig_account_info.key, program_id),
            &[multisig_account_info.clone(), system_program_info.clone()],
            &[&[&wallet_account_info.key.to_bytes()[..], &[nonce]]],
        )?;

        Account::pack(
            multisig_account_data,
            &mut multisig_account_info.data.borrow_mut(),
        )?;

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

        // Get the rent sysvar
        let rent = Rent::get()?;

        if !(wallet_account_info.is_signer && transaction_account_info.is_signer) {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut multisig_account_data =
            Account::unpack_unchecked(&multisig_account_info.data.borrow())?;

        if !multisig_account_data.is_initialized {
            return Err(ProgramError::UninitializedAccount);
        }

        let (pda, _nonce) =
            Pubkey::find_program_address(&[&wallet_account_info.key.to_bytes()], program_id);

        if pda != *multisig_account_info.key {
            return Err(MultisigError::UndefinedTransaction.into());
        }

        if multisig_account_data.pending_transactions.len() >= MAX_TRANSACTIONS {
            return Err(MultisigError::PendingTransactionLimit.into());
        }

        if multisig_account_data.frozen_amount + amount > multisig_account_info.lamports() {
            return Err(MultisigError::InsufficientBalance.into());
        }

        let transaction_account_data = Transaction {
            multisig: *multisig_account_info.key,
            recipient: *recipient_account_info.key,
            amount,
            is_executed: false,
            signers: multisig_account_data
                .owners
                .clone()
                .into_iter()
                .map(|owner| (owner, false))
                .collect(),
        };

        invoke(
            &system_instruction::create_account(
                wallet_account_info.key,
                transaction_account_info.key,
                rent.minimum_balance(Transaction::LEN),
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

        multisig_account_data.frozen_amount += amount;
        multisig_account_data
            .pending_transactions
            .push(*transaction_account_info.key);

        Account::pack(
            multisig_account_data,
            &mut multisig_account_info.data.borrow_mut(),
        )?;
        Transaction::pack(
            transaction_account_data,
            &mut transaction_account_info.data.borrow_mut(),
        )?;

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

        let mut multisig_info = Account::unpack_unchecked(&multisig_account_info.data.borrow())?;
        let transaction_index = multisig_info
            .pending_transactions
            .iter()
            .position(|x| x == transaction_account_info.key)
            .ok_or(MultisigError::UndefinedTransaction)?;

        let mut transaction_info =
            Transaction::unpack_unchecked(&transaction_account_info.data.borrow())?;

        if transaction_info.is_executed {
            return Err(MultisigError::TransactionAlreadyExecuted.into());
        }

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
            .ok_or(MultisigError::InvalidCustodian)?;

        let signers_count = transaction_info
            .signers
            .iter()
            .filter(|(_, is_signed)| *is_signed)
            .count() as u32;

        if multisig_info.threshold >= signers_count {
            // Make lamports transfer
            **multisig_account_info.try_borrow_mut_lamports()? -= transaction_info.amount;
            **recipient_account_info.try_borrow_mut_lamports()? += transaction_info.amount;

            // Mark as executable
            transaction_info.is_executed = true;

            // Unlock frozen lamports
            multisig_info.frozen_amount -= transaction_info.amount;

            // Remove from pending list
            multisig_info.pending_transactions.remove(transaction_index);
        }

        Account::pack(multisig_info, &mut multisig_account_info.data.borrow_mut())?;
        Transaction::pack(
            transaction_info,
            &mut transaction_account_info.data.borrow_mut(),
        )?;

        Ok(())
    }
}
