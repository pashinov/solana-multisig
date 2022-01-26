use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::Pubkey;

pub const MAX_OWNERS: usize = 8;
pub const MAX_TRANSACTIONS: usize = 10;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
    // Init status
    pub is_initialized: bool,
    // Seed to sign transaction without private key
    pub seed: u8,
    // Required number of signers
    pub threshold: u32,
    // Wallet address for associated multisig account
    pub wallet: Pubkey,
    // Custodians of multisig account
    pub owners: Vec<Pubkey>,
    // Set of new transactions
    pub transactions: Vec<Pubkey>,
}

impl Sealed for Account {}

impl IsInitialized for Account {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

const MULTISIG_ACCOUNT_LEN: usize = 622; // 1 + 1 + 4 + 32 + 4 + 32*MAX_OWNERS + 4 + 32*MAX_TRANSACTIONS

impl Pack for Account {
    const LEN: usize = MULTISIG_ACCOUNT_LEN;
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let unpacked = Self::try_from_slice(src)?;
        Ok(unpacked)
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
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

const TRANSACTION_LEN: usize = 622; // 32 + 32 + 8 + 1 + 4 + (32 + 1)*MAX_OWNERS

impl Pack for Transaction {
    const LEN: usize = TRANSACTION_LEN;
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let unpacked = Self::try_from_slice(src)?;
        Ok(unpacked)
    }
}
