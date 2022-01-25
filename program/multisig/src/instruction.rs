use std::convert::TryInto;

use arrayref::array_ref;
use borsh::BorshSerialize;

use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use crate::check_program_account;

pub enum MultisigInstruction {
    InitializeAccount {
        seed: u8,
        threshold: u64,
        owners: Vec<Pubkey>,
    },
    CreateTransaction {
        amount: u64,
    },
    ApproveTransaction,
}

impl MultisigInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match tag {
            1 => {
                let (seed, rest) = rest.split_at(1);
                let seed = seed
                    .try_into()
                    .ok()
                    .map(u8::from_le_bytes)
                    .ok_or(ProgramError::InvalidInstructionData)?;

                let (threshold, rest) = rest.split_at(8);
                let threshold = threshold
                    .try_into()
                    .ok()
                    .map(u64::from_le_bytes)
                    .ok_or(ProgramError::InvalidInstructionData)?;

                let (owners_len, rest) = rest.split_at(4);
                let owners_len = owners_len
                    .try_into()
                    .ok()
                    .map(u32::from_le_bytes)
                    .ok_or(ProgramError::InvalidInstructionData)?;

                let mut owners = Vec::with_capacity((32 * owners_len) as usize);

                let mut offset = 0;
                for _ in 0..owners_len {
                    let owner = array_ref![rest, offset, 32];
                    owners.push(Pubkey::new(owner));
                    offset += 32;
                }

                Self::InitializeAccount {
                    seed,
                    threshold,
                    owners,
                }
            }
            2 => {
                let amount = rest
                    .try_into()
                    .ok()
                    .map(u64::from_le_bytes)
                    .ok_or(ProgramError::InvalidInstructionData)?;

                Self::CreateTransaction { amount }
            }
            3 => Self::ApproveTransaction,
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }

    pub fn pack(&self) -> Result<Vec<u8>, ProgramError> {
        let mut buf = Vec::new();
        match self {
            Self::InitializeAccount {
                seed,
                threshold,
                owners,
            } => {
                buf.push(1);
                buf.push(*seed);
                buf.extend_from_slice(&threshold.to_le_bytes());
                buf.extend_from_slice(
                    &owners
                        .try_to_vec()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );
            }
            Self::CreateTransaction { amount } => {
                buf.push(2);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::ApproveTransaction => {
                buf.push(3);
            }
        };
        Ok(buf)
    }
}

pub fn initialize_account(
    multisig_program_id: &Pubkey,
    associated_account_pubkey: &Pubkey,
    wallet_account_pubkey: &Pubkey,
    data: &[u8],
) -> Result<Instruction, ProgramError> {
    check_program_account(multisig_program_id)?;

    let accounts = vec![
        AccountMeta::new(*associated_account_pubkey, false),
        AccountMeta::new(*wallet_account_pubkey, false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
    ];

    Ok(Instruction {
        program_id: *multisig_program_id,
        accounts,
        data: data.to_vec(),
    })
}
