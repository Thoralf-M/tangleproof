use crate::error::Result;
use crate::validation::is_valid;
use iota::prelude::{Message, Output};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct InclusionProof {
    pub latest_utxo: Output,
    pub messages: Vec<Message>,
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

    pub async fn is_valid(&self) -> Result<bool> {
        is_valid(&self).await
    }
}
