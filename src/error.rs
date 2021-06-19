pub type Result<T, E = Error> = anyhow::Result<T, E>;
use warp::reject::Reject;

/// Error of the tangleproof lib
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("serde_json Error: {0}")]
    DecodeJSON(#[from] serde_json::Error),
    #[error(transparent)]
    // #[error("Iota_client Error")]
    IotaClientError(#[from] iota_client::Error),
    #[error("Bee_message Error")]
    BeeMessageError(#[from] iota_client::bee_message::Error),
    #[error("Bee_rest_api Error")]
    BeeRestApiError(#[from] iota_client::bee_rest_api::types::error::Error),
    #[error("Hex Error")]
    HexError(#[from] hex::FromHexError),
    #[error("Rocksdb Error")]
    RocksdbError(#[from] rocksdb::Error),
    #[error("ParseIntError")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("FromUtf8Error")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("Parameter is invalid:{0}")]
    InvalidParameter(String),
    #[error("Proof has no message")]
    NoMessage,
    #[error("Can't convert Message to or from MessageDto")]
    ConvertMessage,
    #[error("Messages don't reference each other")]
    InvalidMessageChain,
    #[error("Latest message doesn't have the latest UTXO")]
    InvalidLatestUTXO,
    #[error("Message has no transaction payload")]
    NoTransactionPayload,
    #[error("Message has no indexation payload")]
    NoIndexationPayload,
    #[error("Message id is not in the transaction")]
    MessageIdNotInTransaction,
    #[error("File has no valid proof")]
    InvalidProofFile,
    #[error("Record not found in db")]
    RecordNotFound,
    #[error("Utxo input not found in message")]
    UtxoInputNotFound,
    #[error("Utxo output not found in message")]
    UtxoOutputNotFound,
    #[error("Not enough funds")]
    NotEnoughFunds,
    #[error("Inclusion position not set")]
    InclusionPositionNotSet,
}

impl Reject for Error {}
