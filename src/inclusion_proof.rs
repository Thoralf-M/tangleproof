use crate::{error::Result, validation::is_valid_proof};
use iota_client::{
    bee_message::prelude::{Message, OutputId, TransactionPayload},
    bee_rest_api::types::dtos::{MessageDto, TransactionPayloadDto},
    Client,
};
use serde::{de::Error, Deserialize, Serialize, Serializer};
use std::{convert::TryFrom, str::FromStr};

/// InclusionProof struct which holds the messages and the latest outputid
#[derive(Debug, Clone)]
pub struct InclusionProof {
    pub latest_output_id: OutputId,
    pub message: Message,
    pub transactions: Vec<TransactionPayload>,
}

#[derive(Serialize, Deserialize)]
struct InclusionProofDto {
    #[serde(rename = "latestOutputId")]
    latest_output_id: String,
    message: MessageDto,
    #[serde(rename = "transactions")]
    transactions: Vec<TransactionPayloadDto>,
}

impl Serialize for InclusionProof {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let inclusion_proof = InclusionProofDto {
            latest_output_id: self.latest_output_id.to_string(),
            message: MessageDto::from(&self.message),
            transactions: self
                .transactions
                .iter()
                .map(TransactionPayloadDto::from)
                .collect(),
        };
        inclusion_proof.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for InclusionProof {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value: InclusionProofDto = InclusionProofDto::deserialize(d)?;
        let inclusion_proof =
            InclusionProof {
                latest_output_id: OutputId::from_str(&value.latest_output_id)
                    .map_err(D::Error::custom)?,
                message: Message::try_from(&value.message).map_err(D::Error::custom)?,
                transactions:
                    value
                        .transactions
                        .iter()
                        .map(TransactionPayload::try_from)
                        .collect::<Result<
                            Vec<TransactionPayload>,
                            iota_client::bee_rest_api::types::error::Error,
                        >>()
                        .map_err(D::Error::custom)?,
            };
        Ok(inclusion_proof)
    }
}

impl InclusionProof {
    /// Create a new InclusionProof from an existing OutputId and Message
    pub fn new(output: OutputId, message: Message, transactions: Vec<TransactionPayload>) -> Self {
        InclusionProof {
            latest_output_id: output,
            message,
            transactions,
        }
    }
    /// Verify transaction chain and check if latest output is unspent
    pub async fn is_valid(&self, client: &Client) -> Result<bool> {
        is_valid_proof(client, &self).await
    }
}
