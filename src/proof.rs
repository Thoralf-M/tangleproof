use crate::error::Result;
use crate::validation::is_valid;
use iota::prelude::{Message, Output};
use iota::client::types::{OutputJson, MessageJson};
use serde::{Deserialize, Serialize};
use std::convert::{From, TryFrom};
#[derive(Serialize, Deserialize, Debug)]
pub struct InclusionProof {
    pub latest_utxo: Output,
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InclusionProofJson {
    pub latest_utxo: OutputJson,
    pub messages: Vec<MessageJson>,
}

impl InclusionProof {
    pub fn new(utxo: Output, message: Message) -> Self {
        InclusionProof {
            latest_utxo: utxo,
            //     message_id: "a",
            //     output_id: OutputId {
            //         transaction_id: "af",
            //         index: 0,
            //     },
            //     output: SignatureLockedSingleOutput::new("test", NonZeroU64::new(1).unwrap()),
            // },
            messages: vec![message],
        }
    }
    pub fn to_json(&self) -> InclusionProofJson {
        let output_json = OutputJson::from(&self.latest_utxo);
        let json_messages = self.messages.iter().map(|m| MessageJson::from(m)).collect();
        InclusionProofJson{
           latest_utxo: output_json,
           messages: json_messages,
        }
    }
    pub fn from_json(json_proof: InclusionProofJson) -> Result<Self> {
        // let json_proof:  = serde_json::from_str(&proof)?;
        println!("{:?}",json_proof);
        Ok(Self{
            latest_utxo: Output::try_from(json_proof.latest_utxo)?,
            messages: json_proof.messages.into_iter().map(|m| Message::try_from(m).expect("Invalid message in proof object")).collect(),
        })
    }
    pub async fn is_valid(&self) -> Result<bool> {
        is_valid(&self).await
    }
}
