use js_sys::Promise;
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;
use iota_client::{
    bee_message::prelude::{Message, OutputId, TransactionPayload},
    bee_rest_api::types::dtos::{MessageDto, TransactionPayloadDto},
    Client,
};
use serde::{de::Error, Deserialize, Serialize};
use std::{convert::TryFrom, str::FromStr};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::future_to_promise;

mod validation;

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

/// Convert errors so they are readable in JS
pub fn err<T>(error: T) -> JsValue
where
    T: ToString,
{
    error.to_string().into()
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    Ok(())
}

#[wasm_bindgen]
pub fn validate_inclusion_proof(proof: String, node: String) -> Result<Promise, JsValue> {
    let promise: Promise = future_to_promise(async move {
        let iota = Client::builder()
            .with_node(&node)
            .map_err(err)?
            .finish()
            .await
            .map_err(err)?;

        let proof: InclusionProof = serde_json::from_str(&proof).map_err(err)?;
        let is_valid = validation::is_valid_proof(&iota, &proof)
            .await
            .map_err(err)?;

        // result.map_err(wasm_error)
        // .and_then(|res| JsValue::from_serde(&res).map_err(wasm_error))
        JsValue::from_serde(&is_valid).map_err(err)
        // Ok(())
    });
    Ok(promise)
}
