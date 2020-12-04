use crate::error::Result;
use crate::validation::is_valid;
use iota::client::types::MessageJson;
use iota::prelude::{Message, OutputId};
use serde::{Deserialize, Serialize};
use std::convert::{From, TryFrom};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InclusionProof {
    pub latest_output_id: OutputId,
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InclusionProofJson {
    pub latest_output_id: String,
    pub messages: Vec<MessageJson>,
}

impl InclusionProof {
    pub fn new(output: OutputId, message: Message) -> Self {
        InclusionProof {
            latest_output_id: output,
            messages: vec![message],
        }
    }
    pub fn to_json(&self) -> InclusionProofJson {
        let output_json = self.latest_output_id.to_string();
        let json_messages = self.messages.iter().map(MessageJson::from).collect();
        InclusionProofJson {
            latest_output_id: output_json,
            messages: json_messages,
        }
    }
    pub fn from_json(json_proof: InclusionProofJson) -> Result<Self> {
        Ok(Self {
            latest_output_id: OutputId::from_str(&json_proof.latest_output_id)
                .expect("Invalid output id in proof object"),
            messages: json_proof
                .messages
                .into_iter()
                .map(|m| Message::try_from(m).expect("Invalid message in proof object"))
                .collect(),
        })
    }
    pub async fn is_valid(&self, node_url: &str) -> Result<bool> {
        is_valid(&self, node_url).await
    }
}
