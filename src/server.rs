use crate::{
    chronist::{Chronist, UtxoData, INCLUSION_INDEX, TRANSACTION_MESSAGE_KEY},
    error::Result,
};
use iota_client::{
    bee_message::{Message, MessageId},
    bee_rest_api::types::dtos::MessageDto,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{path, Filter, Rejection, Reply};

/// Start the API server
pub async fn start(chronist: Chronist, port: u16) -> Result<()> {
    let chronist = Arc::new(RwLock::new(chronist));
    // GET /
    let api_endpoints = warp::any().map(|| {
        "Available endpoints:\nGET /proof/create/:messageId\nGET /proof/get/:messageId\nPOST /proof/is-valid/\nGET /messages/list\nGET /messages/:messageId\nGET /messages/position/:index"
    });

    // GET /proof/create/:messageId
    let create = warp::path("proof")
        .and(path("create"))
        .and(warp::path::param())
        .and_then({
            let chronist_ = chronist.clone();
            move |m| proof_creation_handler(m, chronist_.clone())
        });

    // GET /proof/get/:messageId
    let get = warp::path("proof")
        .and(path("get"))
        .and(warp::path::param())
        .and_then({
            let chronist_ = chronist.clone();
            move |m| proof_get_handler(m, chronist_.clone())
        });

    // POST /proof/is-valid/
    let is_valid = warp::post()
        .and(path("proof"))
        .and(path("is-valid"))
        .and(warp::body::json())
        .and_then({
            let chronist_ = chronist.clone();
            move |inclusion_proof: crate::inclusion_proof::InclusionProof| {
                proof_is_valid_handler(inclusion_proof, chronist_.clone())
            }
        });

    // GET /messages/list
    let messages = warp::path("messages").and(path("list")).and_then({
        let chronist_ = chronist.clone();
        move || list_messages_handler(chronist_.clone())
    });

    // GET /messages/:messageId
    let message = warp::path("messages").and(warp::path::param()).and_then({
        let chronist_ = chronist.clone();
        move |m| messages_get_handler(m, chronist_.clone())
    });

    // GET /messages/position/:index
    let message_position = warp::path("messages")
        .and(path("position"))
        .and(warp::path::param())
        .and_then({
            let chronist_ = chronist.clone();
            move |p| messages_position_get_handler(p, chronist_.clone())
        });

    let routes = is_valid.or(create
        .or(get)
        .or(messages)
        .or(message)
        .or(message_position)
        .or(api_endpoints));
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    Ok(())
}

pub async fn proof_creation_handler(
    message_id: String,
    chronist: Arc<RwLock<Chronist>>,
) -> std::result::Result<impl Reply, Rejection> {
    let chronist = chronist.read().await;
    chronist.save_message(&message_id).await?;
    println!("proof_creation_handler message_id: {}", message_id);
    Ok(warp::reply::json(&MessageIdResponse { message_id }))
}

pub async fn proof_get_handler(
    message_id: String,
    chronist: Arc<RwLock<Chronist>>,
) -> std::result::Result<impl Reply, Rejection> {
    println!("proof_get_handler message_id: {}", message_id);
    let chronist = chronist.read().await;
    let proof = chronist.get_message_proof(&message_id).await?;
    Ok(warp::reply::json(&proof))
}

pub async fn proof_is_valid_handler(
    inclusion_proof: crate::inclusion_proof::InclusionProof,
    chronist: Arc<RwLock<Chronist>>,
) -> std::result::Result<impl Reply, Rejection> {
    let chronist = chronist.read().await;
    let is_valid = inclusion_proof.is_valid(&chronist.iota_client).await?;
    println!("Requested proof is valid: {}", is_valid);

    Ok(warp::reply::json(&is_valid))
}

pub async fn list_messages_handler(
    chronist: Arc<RwLock<Chronist>>,
) -> std::result::Result<impl Reply, Rejection> {
    let chronist = chronist.read().await;
    let message_ids = chronist.message_ids.read().await;
    Ok(warp::reply::json(
        &message_ids.iter().collect::<Vec<&MessageId>>(),
    ))
}

pub async fn messages_get_handler(
    message_id: String,
    chronist: Arc<RwLock<Chronist>>,
) -> std::result::Result<impl Reply, Rejection> {
    let chronist = chronist.read().await;
    let message = chronist.get_message(&message_id).await?;

    let response = MessageResponse {
        data: MessageDto::from(&message.message),
        inclusion_position: message.inclusion_position,
    };
    Ok(warp::reply::json(&response))
}
use warp::reject;

#[derive(Debug, Clone)]
pub(crate) enum CustomRejection {
    // Forbidden,
    BadRequest(String),
    // NotFound(String),
    // ServiceUnavailable(String),
    // InternalError,
    // StorageBackend,
}
impl warp::reject::Reject for CustomRejection {}
pub async fn messages_position_get_handler(
    position: u64,
    chronist: Arc<RwLock<Chronist>>,
) -> std::result::Result<impl Reply, Rejection> {
    let chronist = chronist.read().await;
    let position_data: UtxoData = match serde_json::from_str(
        &chronist
            .db
            .lock()
            .await
            .get(&format!("{}{}", INCLUSION_INDEX, position))
            .await?,
    ) {
        Ok(id) => id,
        Err(_) => {
            return Err(reject::custom(CustomRejection::BadRequest(
                "Couldn't decode for message".to_string(),
            )))
        }
    };
    let message: Message = match serde_json::from_str(
        &chronist
            .db
            .lock()
            .await
            .get(&format!(
                "{}{}",
                TRANSACTION_MESSAGE_KEY,
                position_data.message_id.to_string()
            ))
            .await?,
    ) {
        Ok(message) => message,
        Err(_) => {
            return Err(reject::custom(CustomRejection::BadRequest(
                "Couldn't decode for message".to_string(),
            )))
        }
    };

    let response = MessageResponse {
        data: MessageDto::from(&message),
        inclusion_position: Some(position_data.position_index),
    };
    Ok(warp::reply::json(&response))
}

/// Response of GET /api/v1/messages?index={INDEX}.
/// Returns all messages ids that match a given indexation key.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageIdResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
}

/// Response of GET /api/v1/messages?index={INDEX}.
/// Returns all messages ids that match a given indexation key.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub data: MessageDto,
    pub inclusion_position: Option<u64>,
}

// impl BodyInner for MessagesIdResponse {}
// https://github.com/iotaledger/bee/blob/chrysalis-pt-2/bee-api/bee-rest-api/src/endpoints/path_params.rs
// pub(super) fn message_id() -> impl Filter<Extract = (MessageId,), Error = Rejection> + Copy {
//     warp::path::param().and_then(|value: String| async move {
//         match value.parse::<MessageId>() {
//             Ok(id) => Ok(id),
//             Err(_) => Err(reject::custom(CustomRejection::BadRequest(
//                 "invalid message id".to_string(),
//             ))),
//         }
//     })
// }

// #[derive(Serialize)]
// struct ErrorResponse {
//     message: String,
// }
