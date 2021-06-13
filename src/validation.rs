use crate::chronist::TransactionData;
use crate::error::Result;
use crate::inclusion_proof::InclusionProof;
use crate::iota_api::is_output_known;
use iota_client::bee_message::prelude::{Essence, Input, Message, Payload};
use iota_client::Client;

/// Function to validate the structure of the proof and check if the latest output is unspent
// 1. check if message id is part of the first indexation payload
// 2. check for each transaction if one output is used as input in the next transaction
// 3. check if latest output is known by the node
pub async fn is_valid_proof(iota_client: &Client, proof: &InclusionProof) -> Result<bool> {
    if proof.transaction_messages.is_empty() {
        return Err(crate::error::Error::NoMessage);
    }
    // 1. check if message id is part of the first indexation payload
    let msg_id = proof.message.id().0;
    let data = get_indexation_data(
        proof
            .transaction_messages
            .first()
            .ok_or(crate::error::Error::NoMessage)?,
    )?;
    let transaction_data: TransactionData = serde_json::from_str(&data)?;
    if !transaction_data.message_ids.contains(&msg_id) {
        return Err(crate::error::Error::MessageIdNotInTransaction);
    }
    // 2. check for each transaction if one output is used as input in the next transaction
    let last_index = proof.transaction_messages.len() - 1;
    for (index, message) in proof.transaction_messages.iter().enumerate() {
        if last_index != index {
            // Check if output from previous tx is used as input in next tx
            if let Some(Payload::Transaction(tx)) = message.payload() {
                if let Some(Payload::Transaction(index_plus_one_tx)) =
                    &proof.transaction_messages[index + 1].payload()
                {
                    let tx_id_first = tx.id();
                    let Essence::Regular(essence1) = index_plus_one_tx.essence();

                    if !essence1.inputs().iter().any(|input| match input {
                        Input::Utxo(utxo) => utxo.output_id().transaction_id() == &tx_id_first,
                        _ => false,
                    }) {
                        return Err(crate::error::Error::InvalidMessageChain);
                    }
                }
            } else {
                return Err(crate::error::Error::InvalidMessageChain);
            }
        } else if let Some(Payload::Transaction(tx)) = message.payload() {
            if tx.id() != *proof.latest_output_id.transaction_id() {
                return Err(crate::error::Error::InvalidLatestUTXO);
            }
        } else {
            return Err(crate::error::Error::NoTransactionPayload);
        }
    }
    // 3. check if latest output is known by the node
    Ok(is_output_known(iota_client, &proof.latest_output_id).await)
}

fn get_indexation_data(message: &Message) -> Result<String> {
    match message.payload() {
        Some(Payload::Transaction(tx_payload)) => {
            let Essence::Regular(essence) = tx_payload.essence();
            match essence.payload() {
                Some(Payload::Indexation(indexation_payload)) => {
                    Ok(String::from_utf8(indexation_payload.data().to_vec())?)
                }
                _ => Err(crate::error::Error::NoIndexationPayload),
            }
        }
        _ => Err(crate::error::Error::NoTransactionPayload),
    }
}
