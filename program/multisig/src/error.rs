//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the Multisig program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum MultisigError {
    #[error("Pending transaction limit exceeded")]
    PendingTransactionLimit,
    #[error("Multisig transaction doesn't belong to multisig account")]
    UndefinedMultisigTransaction,
    #[error("Pending transaction limit exceeded")]
    TransactionAlreadyExecuted,
    #[error("Signer is not custodian of multisig account")]
    InvalidCustodian,
}
impl From<MultisigError> for ProgramError {
    fn from(e: MultisigError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for MultisigError {
    fn type_of() -> &'static str {
        "MultisigError"
    }
}
