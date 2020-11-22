use crate::error::Result;
use crate::proof::InclusionProof;
use crate::tangle::fetch;
use iota::prelude::Output;
// use iota::prelude::UTXOInput;
use iota::Payload;

pub async fn is_valid(proof: &InclusionProof) -> Result<bool> {
    if proof.messages.is_empty() {
        return Err(crate::error::Error::NoMessage);
    }
    // validate each message and hash message chain
    // let messages: Vec<Message> = proof
    //     .messages
    //     .iter()
    //     .map(|m| serde_json::from_str::<Message>(&m).expect("Invalid message input"))
    //     .collect();
    let last_index = proof.messages.len() - 1;
    for (index, message) in proof.messages.iter().enumerate() {
        if last_index != index {
            if message.id() != *proof.messages[index + 1].parent1()
                || message.id() != *proof.messages[index + 1].parent2()
            {
                return Err(crate::error::Error::InvalidMessageChain);
            }
        } else {
            if let Payload::Transaction(tx) = message.payload().as_ref().unwrap() {
                // if !tx.essence().outputs().iter().any(|utxo| {
                //     let utxo_string = serde_json::to_string(&utxo).expect("Invalid utxo");
                //     utxo_string == proof.latest_utxo
                // })
                for outputs in tx.essence().outputs().iter() {
                    if let Output::SignatureLockedSingle(x) = &outputs {
                        println!("{:?}", x.address());
                    }
                    // if outputs.output_id == proof.latest_utxo {
                    //     println!("UTXO found");
                    // }
                }
            // if !tx.essence().outputs().contains(&proof.latest_utxo) {
            //     return Err(crate::error::Error::InvalidLatestUTXO);
            // }
            } else {
                return Err(crate::error::Error::NoTransactionPayload);
            }
        }
    }

    // get latest utxo and check from node
    let _msg_id = fetch(&proof.latest_utxo).await?;
    Ok(true)
}
