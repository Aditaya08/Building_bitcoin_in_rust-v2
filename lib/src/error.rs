use thiserror::Error;

pub type Result<T> = std::result::Result<T, BtcError>;

#[derive(Debug, Error)]
pub enum BtcError {
    #[error("block does not point to the current chain tip")]
    InvalidPreviousHash,
    #[error("block merkle root is invalid")]
    InvalidMerkleRoot,
    #[error("block hash does not satisfy target")]
    InvalidProofOfWork,
    #[error("transaction has no outputs")]
    EmptyTransaction,
    #[error("transaction input references an unknown output")]
    UnknownInput,
    #[error("transaction input signature is invalid")]
    InvalidSignature,
    #[error("transaction spends the same output twice")]
    DuplicateSpend,
    #[error("transaction outputs exceed inputs")]
    Overspend,
    #[error("coinbase transaction is invalid")]
    InvalidCoinbase,
    #[error("serialization failed")]
    Serialization,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("crypto error: {0}")]
    Crypto(String),
}
