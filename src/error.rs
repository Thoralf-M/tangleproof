pub type Result<T, E = Error> = anyhow::Result<T, E>;

/// Error type of the tangleproof lib
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("serde_json Error: {0}")]
    DecodeJSON(#[from] serde_json::Error),
    #[error("IO Error")]
    IOError(#[from] std::io::Error),
    #[error("Iota_client Error")]
    IotaClientError(#[from] iota::client::Error),
    #[error("Hex Error")]
    HexError(#[from] hex::FromHexError),
    #[error("Parameter is invalid:{0}")]
    InvalidParameter(String),
    #[error("Proof has no message")]
    NoMessage,
    #[error("Proof has no UTXO")]
    NoUtxo,
    #[error("Messages don't reference each other")]
    InvalidMessageChain,
    #[error("Latest message doesn't have the latest UTXO")]
    InvalidLatestUTXO,
    #[error("Message has no transaction Payload")]
    NoTransactionPayload,
    #[error("File has no valid proof")]
    InvalidProofFile,
}
