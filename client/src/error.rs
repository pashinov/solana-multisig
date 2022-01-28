use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read solana config file: ({0})")]
    ConfigReadError(std::io::Error),
    #[error("failed to parse solana config file: ({0})")]
    ConfigParseError(#[from] yaml_rust::ScanError),
    #[error("invalid config: ({0})")]
    InvalidConfig(String),
    #[error("invalid threshold")]
    InvalidThreshold,
    #[error("invalid owners")]
    InvalidOwners,
    #[error("owners cannot be greater than the threshold")]
    InvalidOwnersNumber,
    #[error("invalid recipient")]
    InvalidRecipient,
    #[error("invalid amount")]
    InvalidAmount,

    #[error("solana client error: ({0})")]
    ClientError(#[from] solana_client::client_error::ClientError),
}

pub type Result<T> = std::result::Result<T, Error>;
