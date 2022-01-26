use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::Pubkey;

use crate::utils::*;

pub const MAX_OWNERS: usize = 8;

pub struct Account {
    // Init status
    pub is_initialized: bool,
    // Seed to sign transaction without private key
    pub seed: u8,
    // Required number of signers
    pub threshold: u64,
    // Wallet address for associated multisig account
    pub wallet: Pubkey,
    // Custodians of multisig account
    pub owners: Vec<Pubkey>,
}

impl Sealed for Account {}

impl IsInitialized for Account {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Account {
    const LEN: usize = 302;
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Account::LEN];

        let (
            is_initialized_dst,
            seed_dst,
            threshold_dst,
            wallet_dst,
            owners_len_dst,
            owners_data_dst,
        ) = mut_array_refs![dst, 1, 1, 8, 32, 4, 32 * MAX_OWNERS];

        let Account {
            is_initialized,
            seed,
            threshold,
            wallet,
            owners,
        } = self;

        pack_bool(*is_initialized, is_initialized_dst);

        seed_dst[0] = *seed as u8;
        *threshold_dst = threshold.to_le_bytes();
        wallet_dst.copy_from_slice(wallet.as_ref());
        *owners_len_dst = (owners.len() as u32).to_le_bytes();

        let mut offset = 0;
        for owner in owners {
            let owners_data = array_mut_ref![owners_data_dst, offset, 32];
            owners_data.copy_from_slice(owner.as_ref());
            offset += 32;
        }
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Account::LEN];
        let (is_initialized, seed, threshold, wallet, owners_len, owners_data) =
            array_refs![src, 1, 1, 8, 32, 4, 32 * MAX_OWNERS];

        let is_initialized = unpack_bool(is_initialized)?;

        let seed = u8::from_le_bytes(*seed);
        let threshold = u64::from_le_bytes(*threshold);
        let wallet = Pubkey::new(wallet);
        let owners_len = u32::from_le_bytes(*owners_len);

        let mut owners = Vec::with_capacity((owners_len * 32) as usize);

        let mut offset = 0;
        for _ in 0..owners_len {
            let owner = array_ref![owners_data, offset, 32];
            owners.push(Pubkey::new(owner));
            offset += 32;
        }

        Ok(Self {
            is_initialized,
            seed,
            threshold,
            wallet,
            owners,
        })
    }
}

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

impl Pack for Transaction {
    const LEN: usize = 341;
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Transaction::LEN];

        let (
            multisig_dst,
            recipient_dst,
            amount_dst,
            is_executed_dst,
            signers_len_dst,
            signers_dst,
        ) = mut_array_refs![dst, 32, 32, 8, 1, 4, (32 + 1) * MAX_OWNERS];

        let Transaction {
            multisig,
            recipient,
            amount,
            is_executed,
            signers,
        } = self;

        multisig_dst.copy_from_slice(multisig.as_ref());
        recipient_dst.copy_from_slice(recipient.as_ref());

        *amount_dst = amount.to_le_bytes();

        pack_bool(*is_executed, is_executed_dst);

        *signers_len_dst = (signers.len() as u32).to_le_bytes();

        let mut offset = 0;
        for (owner, is_signed) in signers {
            let owner_dst = array_mut_ref![signers_dst, offset, 32];
            owner_dst.copy_from_slice(owner.as_ref());
            offset += 32;

            let is_signed_dst = array_mut_ref![signers_dst, offset, 1];
            pack_bool(*is_signed, is_signed_dst);
            offset += 1;
        }
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Transaction::LEN];
        let (multisig, recipient, amount, is_executed, signers_len, signers_data) =
            array_refs![src, 32, 32, 8, 1, 4, (32 + 1) * MAX_OWNERS];

        let multisig = Pubkey::new(multisig);
        let recipient = Pubkey::new(recipient);

        let amount = u64::from_le_bytes(*amount);

        let is_executed = unpack_bool(is_executed)?;

        let signers_len = u32::from_le_bytes(*signers_len);

        let mut signers = Vec::with_capacity((signers_len * (32 + 1)) as usize);

        let mut offset = 0;
        for _ in 0..signers_len {
            let signer = array_ref![signers_data, offset, 32];
            offset += 32;
            let is_signed = array_ref![signers_data, offset, 1];
            offset += 1;

            signers.push((Pubkey::new(signer), unpack_bool(is_signed)?));
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
