use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::{Pubkey, PUBKEY_BYTES};

/// Minimum number of multisignature signers
pub const MIN_SIGNERS: usize = 1;
/// Maximum number of multisignature signers
pub const MAX_SIGNERS: usize = 8;
/// Maximum number of simultaneous pending transactions
pub const MAX_TRANSACTIONS: usize = 10;

use crate::utils::*;

#[derive(Debug)]
pub struct Account {
    // Init status
    pub is_initialized: bool,
    // Required number of signers
    pub threshold: u32,
    // Custodians of multisig account
    pub owners: Vec<Pubkey>,
    // Set of pending transactions
    pub pending_transactions: Vec<Pubkey>,
    // Frozen lamports by pending transactions
    pub frozen_amount: u64,
}

impl Sealed for Account {}

impl IsInitialized for Account {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

const ACCOUNT_LEN: usize = 597;

impl Pack for Account {
    const LEN: usize = ACCOUNT_LEN;
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, ACCOUNT_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            is_initialized,
            threshold,
            frozen_amount,
            owners_len,
            pending_transactions_len,
            data_flat,
        ) = mut_array_refs![
            dst,
            1,
            4,
            8,
            4,
            4,
            PUBKEY_BYTES * MAX_SIGNERS + PUBKEY_BYTES * MAX_TRANSACTIONS
        ];

        pack_bool(self.is_initialized, is_initialized);
        *threshold = self.threshold.to_le_bytes();
        *frozen_amount = self.frozen_amount.to_le_bytes();
        *owners_len = (self.owners.len() as u32).to_le_bytes();
        *pending_transactions_len = (self.pending_transactions.len() as u32).to_le_bytes();

        let mut offset = 0;
        for owner in &self.owners {
            let owners_flat = array_mut_ref![data_flat, offset, PUBKEY_BYTES];
            owners_flat.copy_from_slice(owner.as_ref());
            offset += PUBKEY_BYTES;
        }
        for pending_transaction in &self.pending_transactions {
            let pending_transactions_flat = array_mut_ref![data_flat, offset, PUBKEY_BYTES];
            pending_transactions_flat.copy_from_slice(pending_transaction.as_ref());
            offset += PUBKEY_BYTES;
        }
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![src, 0, ACCOUNT_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            is_initialized,
            threshold,
            frozen_amount,
            owners_len,
            pending_transactions_len,
            data_flat,
        ) = array_refs![
            input,
            1,
            4,
            8,
            4,
            4,
            PUBKEY_BYTES * MAX_SIGNERS + PUBKEY_BYTES * MAX_TRANSACTIONS
        ];

        let is_initialized = unpack_bool(is_initialized)?;
        let threshold = u32::from_le_bytes(*threshold);
        let frozen_amount = u64::from_le_bytes(*frozen_amount);
        let owners_len = u32::from_le_bytes(*owners_len);
        let pending_transactions_len = u32::from_le_bytes(*pending_transactions_len);

        let mut owners = Vec::with_capacity(owners_len as usize);
        let mut pending_transactions = Vec::with_capacity(pending_transactions_len as usize);

        let mut offset = 0;
        for _ in 0..owners_len {
            let owners_flat = array_ref![data_flat, offset, PUBKEY_BYTES];
            owners.push(Pubkey::new(owners_flat));
            offset += PUBKEY_BYTES;
        }
        for _ in 0..pending_transactions_len {
            let pending_transactions_flat = array_ref![data_flat, offset, PUBKEY_BYTES];
            pending_transactions.push(Pubkey::new(pending_transactions_flat));
            offset += PUBKEY_BYTES;
        }

        Ok(Self {
            is_initialized,
            threshold,
            owners,
            pending_transactions,
            frozen_amount,
        })
    }
}

#[derive(Debug)]
pub struct Transaction {
    // The multisig account this transaction belongs to
    pub multisig: Pubkey,
    // Recipient address
    pub recipient: Pubkey,
    // Amount of lamports to send to recipient
    pub amount: u64,
    // Boolean ensuring one time execution.
    pub is_executed: bool,
    // Owners with status of transaction signature
    pub signers: Vec<(Pubkey, bool)>,
}

impl Sealed for Transaction {}

const TRANSACTION_LEN: usize = 341; // 32 + 32 + 8 + 1 + 4 + (32 + 1)*MAX_OWNERS

impl Pack for Transaction {
    const LEN: usize = TRANSACTION_LEN;
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, TRANSACTION_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (multisig, recipient, amount, is_executed, signers_len, signers_flat) = mut_array_refs![
            dst,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            8,
            1,
            4,
            (32 + 1) * MAX_SIGNERS
        ];

        *amount = self.amount.to_le_bytes();
        multisig.copy_from_slice(self.multisig.as_ref());
        recipient.copy_from_slice(self.recipient.as_ref());
        pack_bool(self.is_executed, is_executed);

        *signers_len = (self.signers.len() as u32).to_le_bytes();

        let mut offset = 0;
        for (signer, is_signed) in &self.signers {
            let signer_flat = array_mut_ref![signers_flat, offset, PUBKEY_BYTES];
            signer_flat.copy_from_slice(signer.as_ref());
            offset += PUBKEY_BYTES;

            let is_signed_flat = array_mut_ref![signers_flat, offset, 1];
            pack_bool(*is_signed, is_signed_flat);
            offset += 1;
        }
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![src, 0, TRANSACTION_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (multisig, recipient, amount, is_executed, signers_len, signers_flat) = array_refs![
            input,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            8,
            1,
            4,
            (32 + 1) * MAX_SIGNERS
        ];

        let is_executed = unpack_bool(is_executed)?;
        let multisig = Pubkey::new(multisig);
        let recipient = Pubkey::new(recipient);
        let amount = u64::from_le_bytes(*amount);

        let signers_len = u32::from_le_bytes(*signers_len);

        let mut signers = Vec::with_capacity(signers_len as usize);

        let mut offset = 0;
        for _ in 0..signers_len {
            let signer_flat = array_ref![signers_flat, offset, PUBKEY_BYTES];
            offset += PUBKEY_BYTES;
            let is_signed = array_ref![signers_flat, offset, 1];
            offset += 1;

            signers.push((Pubkey::new(signer_flat), unpack_bool(is_signed)?));
        }

        Ok(Self {
            multisig,
            recipient,
            amount,
            is_executed,
            signers,
        })
    }
}
