use crate::chronist::InclusionData;
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
    // 1. check if message id is part of the first indexation payload
    let msg_id = proof.message.id().0;
    let inclusion_data = get_inclusion_data(
        proof
            .transaction_messages
            .first()
            .ok_or(crate::error::Error::NoMessage)?,
    )?;
    if !inclusion_data.message_ids.contains(&msg_id) {
        return Err(crate::error::Error::MessageIdNotInTransaction);
    }
    // 2. check for each transaction if one output is used as input in the next transaction
    validate_transaction_chain(&proof.transaction_messages)?;

    // Check if latest_output_id is part of the latest transaction
    if let Some(Payload::Transaction(tx)) = proof
        .transaction_messages
        .last()
        // Save to unwrap because we checked already if it's empty
        .ok_or(crate::error::Error::NoMessage)?
        .payload()
    {
        if tx.id() != *proof.latest_output_id.transaction_id() {
            return Err(crate::error::Error::InvalidLatestUTXO);
        }
    } else {
        return Err(crate::error::Error::NoTransactionPayload);
    }
    // 3. check if latest output is known by the node
    Ok(is_output_known(iota_client, &proof.latest_output_id).await)
}

fn get_inclusion_data(message: &Message) -> Result<InclusionData> {
    match message.payload() {
        Some(Payload::Transaction(tx_payload)) => {
            let Essence::Regular(essence) = tx_payload.essence();
            match essence.payload() {
                Some(Payload::Indexation(indexation_payload)) => {
                    let data = String::from_utf8(indexation_payload.data().to_vec())?;
                    let transaction_data: InclusionData = serde_json::from_str(&data)?;
                    Ok(transaction_data)
                }
                _ => Err(crate::error::Error::NoIndexationPayload),
            }
        }
        _ => Err(crate::error::Error::NoTransactionPayload),
    }
}

// Checks for each transaction if one output is used as input in the next transaction
fn validate_transaction_chain(transaction_messages: &[Message]) -> Result<()> {
    // Check if output from previous tx is used as input in next tx
    for messages in transaction_messages.windows(2) {
        let tx = match messages[0].payload() {
            Some(Payload::Transaction(tx)) => tx,
            _ => return Err(crate::error::Error::NoTransactionPayload),
        };
        let next_tx = match messages[1].payload() {
            Some(Payload::Transaction(tx)) => tx,
            _ => return Err(crate::error::Error::NoTransactionPayload),
        };

        let Essence::Regular(next_essence) = next_tx.essence();

        if !next_essence.inputs().iter().any(|input| match input {
            Input::Utxo(utxo) => utxo.output_id().transaction_id() == &tx.id(),
            _ => false,
        }) {
            return Err(crate::error::Error::InvalidMessageChain);
        }
    }
    Ok(())
}
