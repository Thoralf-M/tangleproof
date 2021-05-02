use crate::error::Result;
use crate::io;
use crate::tangle::send_transaction;
use crate::validation::is_valid;
use iota::{
    bee_rest_api::types::dtos::MessageDto,
    prelude::{Message, OutputId},
    MessageId, Payload, TransactionId,
};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, str::FromStr};

/// InclusionProof struct which holds the messages and the latest outputid
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InclusionProof {
    pub latest_output_id: OutputId,
    pub messages: Vec<Message>,
}

/// InclusionProofJson struct for human readability
#[derive(Serialize, Deserialize, Debug)]
pub struct InclusionProofJson {
    pub latest_output_id: String,
    pub messages: Vec<MessageDto>,
}

impl InclusionProof {
    /// Create a new InclusionProof from an existing OutputId and Message
    pub fn new(output: OutputId, message: Message) -> Self {
        InclusionProof {
            latest_output_id: output,
            messages: vec![message],
        }
    }
    /// Send data and add update an exisiting InclusionProof .json file or create a new one
    pub async fn send_data(
        indexation_tag: &str,
        data: &str,
        amount: u64,
        node_url: &str,
        local_pow: bool,
        seed: &str,
        proof_name: &str,
    ) -> Result<(MessageId, TransactionId, Self)> {
        let mut input: Option<OutputId> = None;
        if let Ok(proof) = InclusionProof::from_file(&proof_name).await {
            input = Some(proof.latest_output_id);
        }
        let (messageid, msg) = send_transaction(
            indexation_tag,
            data,
            amount,
            input,
            node_url,
            local_pow,
            seed,
        )
        .await?;

        let tx = match msg.payload().as_ref().expect("No payload").clone() {
            Payload::Transaction(tx) => tx,
            _ => return Err(crate::error::Error::NoTransactionPayload),
        };

        let proof = match io::read_from_file(proof_name)? {
            Some(mut proof) => {
                // Update existing proof
                proof.latest_output_id = OutputId::new(tx.id(), 0).expect("Can't get output id");
                proof.messages.push(msg);
                proof
            }
            _ => {
                // Create a new proof
                InclusionProof::new(OutputId::new(tx.id(), 0).expect("Can't get output id"), msg)
            }
        };
        io::write_to_file(proof_name, proof.clone())?;
        Ok((messageid, tx.id(), proof))
    }
    /// Convert proof type for better readability
    pub fn to_json(&self) -> Result<InclusionProofJson> {
        let output_json = self.latest_output_id.to_string();
        let json_messages = self
            .messages
            .iter()
            .map(|m| MessageDto::try_from(m).map_err(|_| crate::error::Error::ConvertMessage))
            .collect::<Result<Vec<MessageDto>>>()?;
        Ok(InclusionProofJson {
            latest_output_id: output_json,
            messages: json_messages,
        })
    }
    /// Convert Proof back from InclusionProofJson
    pub fn from_json(json_proof: InclusionProofJson) -> Result<Self> {
        Ok(Self {
            latest_output_id: OutputId::from_str(&json_proof.latest_output_id)
                .expect("Invalid output id in proof object"),
            messages: json_proof
                .messages
                .into_iter()
                .map(|m| Message::try_from(&m).expect("Invalid message in proof object"))
                .collect(),
        })
    }
    /// Read from file
    pub async fn from_file(filename: &str) -> Result<Self> {
        match io::read_from_file(filename)? {
            Some(proof) => Ok(proof),
            _ => return Err(crate::error::Error::InvalidProofFile),
        }
    }
    /// Verify transaction chain and check if latest output is unspent
    pub async fn is_valid(&self, node_url: &str) -> Result<bool> {
        is_valid(&self, node_url).await
    }
}
