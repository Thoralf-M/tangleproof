use crate::error::Result;
use crate::proof::InclusionProof;
use crate::tangle::is_output_spent;
use iota::prelude::Input;
use iota::prelude::OutputId;
use iota::prelude::UTXOInput;
use iota::Payload;
use std::collections::HashSet;

pub async fn is_valid(proof: &InclusionProof, node_url: &str) -> Result<bool> {
    if proof.messages.is_empty() {
        return Err(crate::error::Error::NoMessage);
    }
    let last_index = proof.messages.len() - 1;
    for (index, message) in proof.messages.iter().enumerate() {
        if last_index != index {
            // Check if previous tx is used in the input
            if let Payload::Transaction(tx) = message.payload().as_ref().unwrap() {
                if let Payload::Transaction(tx1) =
                    &proof.messages[index + 1].payload().as_ref().unwrap()
                {
                    let outputs = tx.essence().outputs();
                    let mut output_ids = Vec::new();
                    let inputs = tx1.essence().inputs();
                    for i in 0..outputs.len() {
                        output_ids.push(Input::UTXO(UTXOInput::from(
                            OutputId::new(tx.id(), i as u16).expect("Can't get output id"),
                        )));
                    }
                    let a: HashSet<_> = output_ids.into_iter().collect();
                    let b: HashSet<_> = inputs.iter().cloned().collect();
                    let intersection: Vec<&Input> = a.intersection(&b).collect();
                    if intersection.is_empty() {
                        return Err(crate::error::Error::InvalidMessageChain);
                    }
                }
            } else {
                return Err(crate::error::Error::InvalidMessageChain);
            }
        } else if let Payload::Transaction(tx) = message.payload().as_ref().unwrap() {
            if OutputId::new(tx.id(), 0).expect("Can't get output id") != proof.latest_output_id {
                return Err(crate::error::Error::InvalidLatestUTXO);
            }
        } else {
            return Err(crate::error::Error::NoTransactionPayload);
        }
    }
    let spent_status = is_output_spent(&proof.latest_output_id, node_url).await?;
    Ok(!spent_status)
}
