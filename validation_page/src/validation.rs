use crate::InclusionProof;
use chrono::{DateTime, Utc};
use iota_client::{
    bee_message::prelude::{
        Essence, Input, MessageId, OutputId, Payload, TransactionPayload, UtxoInput,
    },
    Client,
};
use serde::{Deserialize, Serialize};

/// Function to get an outputid
pub async fn is_output_known(client: &Client, output_id: &OutputId) -> bool {
    matches!(client.get_output(&UtxoInput::from(*output_id)).await, Ok(_))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InclusionData {
    pub time: DateTime<Utc>,
    pub message_ids: Vec<MessageId>,
}

pub type Result<T, E = Error> = anyhow::Result<T, E>;

/// Error of the tangleproof lib
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("serde_json Error: {0}")]
    DecodeJSON(#[from] serde_json::Error),
    #[error("FromUtf8Error")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
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
}

/// Function to validate the structure of the proof and check if the latest output is known
// 1. Calculate message id and check if it's part of the indexation payload of the first transaction
// 2. Check for each transaction if one output is used as input in the next transaction
// 3. Check if latest output is known by a node
pub async fn is_valid_proof(iota_client: &Client, proof: &InclusionProof) -> Result<bool> {
    // 1. check if message id is part of the first indexation payload
    let msg_id = proof.message.id().0;
    let inclusion_data = get_inclusion_data(
        proof
            .transactions
            .first()
            .ok_or(Error::NoTransactionPayload)?,
    )?;
    if !inclusion_data.message_ids.contains(&msg_id) {
        return Err(Error::MessageIdNotInTransaction);
    }
    // 2. check for each transaction if one output is used as input in the next transaction
    validate_transaction_chain(&proof.transactions)?;

    // Check if latest_output_id is part of the latest transaction
    if proof
        .transactions
        .last()
        // Save to unwrap because we checked already if it's empty
        .ok_or(Error::NoTransactionPayload)?
        .id()
        != *proof.latest_output_id.transaction_id()
    {
        return Err(Error::InvalidLatestUTXO);
    }

    // 3. check if latest output is known by the node
    Ok(is_output_known(iota_client, &proof.latest_output_id).await)
}

fn get_inclusion_data(tx_payload: &TransactionPayload) -> Result<InclusionData> {
    let Essence::Regular(essence) = tx_payload.essence();
    match essence.payload() {
        Some(Payload::Indexation(indexation_payload)) => {
            let data = String::from_utf8(indexation_payload.data().to_vec())?;
            let transaction_data: InclusionData = serde_json::from_str(&data)?;
            Ok(transaction_data)
        }
        _ => Err(Error::NoIndexationPayload),
    }
}

// Checks for each transaction if one output is used as input in the next transaction
fn validate_transaction_chain(transactions: &[TransactionPayload]) -> Result<()> {
    // Check if output from previous tx is used as input in next tx
    for txs in transactions.windows(2) {
        let Essence::Regular(next_essence) = txs[1].essence();

        if !next_essence.inputs().iter().any(|input| match input {
            Input::Utxo(utxo) => utxo.output_id().transaction_id() == &txs[0].id(),
            _ => false,
        }) {
            return Err(Error::InvalidMessageChain);
        }
    }
    Ok(())
}
