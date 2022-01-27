use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum MultisigError {
    #[error("Pending transaction limit exceeded")]
    PendingTransactionLimit,
    #[error("Multisig owners limit exceeded")]
    CustodianLimit,
    #[error("Multisig transaction doesn't belong to multisig account")]
    UndefinedTransaction,
    #[error("Pending transaction limit exceeded")]
    TransactionAlreadyExecuted,
    #[error("Signer is not custodian of multisig account")]
    InvalidCustodian,
    #[error("Insufficient multisig balance")]
    InsufficientBalance,
    #[error("Amount Overflow")]
    AmountOverflow,
}
impl From<MultisigError> for ProgramError {
    fn from(e: MultisigError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
