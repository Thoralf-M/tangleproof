use crate::chronist::Chronist;
use crate::error::Result;
use iota_client::bee_message::MessageId;
use iota_client::bee_rest_api::types::dtos::MessageDto;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{path, Filter};
use warp::{Rejection, Reply};

/// Start the API server
pub async fn start(chronist: Chronist, port: u16) -> Result<()> {
    let chronist = Arc::new(RwLock::new(chronist));
    // GET /
    let api_endpoints = warp::path("endpoints").map(|| {
        "Available endpoints:\nGET /proof/create/:messageId\nGET /proof/create/:messageId\nPOST /proof/is-valid/\nGET /messages/list\nGET /messages/:messageId"
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

    let routes = is_valid.or(create.or(get).or(messages).or(message).or(api_endpoints));
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    Ok(())
}

pub async fn proof_creation_handler(
    message_id: String,
    chronist: Arc<RwLock<Chronist>>,
) -> std::result::Result<impl Reply, Rejection> {
    let chronist = chronist.write().await;
    chronist.save_message(&message_id).await?;
    println!("message_id: {}", message_id);
    //todo create proof
    Ok(warp::reply::json(&MessageIdResponse { message_id }))
}

pub async fn proof_get_handler(
    message_id: String,
    chronist: Arc<RwLock<Chronist>>,
) -> std::result::Result<impl Reply, Rejection> {
    println!("message_id: {}", message_id);
    let chronist = chronist.read().await;
    let proof = chronist.get_message_proof(&message_id).await?;
    //todo get proof
    // let message_dto = match serde_json::to_string(&MessageDto::from(&message)) {
    //     Ok(m) => m,
    //     Err(e) => return Err(warp::reject::custom::<crate::error::Error>(e.into())),
    // };

    Ok(warp::reply::json(&proof))
}

pub async fn proof_is_valid_handler(
    inclusion_proof: crate::inclusion_proof::InclusionProof,
    chronist: Arc<RwLock<Chronist>>,
) -> std::result::Result<impl Reply, Rejection> {
    let chronist = chronist.read().await;
    // let inclusion_proof: crate::inclusion_proof::InclusionProof = match serde_json::from_str(&proof)
    // {
    //     Ok(proof) => proof,
    //     Err(e) => return Err(warp::reject::custom::<crate::error::Error>(e.into())),
    // };
    let client = chronist.iota_client.read().await;
    let is_valid = inclusion_proof.is_valid(&client).await?;
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

// use warp::reject::Reject;

// #[derive(Debug, Clone)]
// pub(crate) enum CustomRejection {
//     Forbidden,
//     BadRequest(String),
//     NotFound(String),
//     ServiceUnavailable(String),
//     InternalError,
//     StorageBackend,
// }

// #[derive(Serialize)]
// struct ErrorResponse {
//     message: String,
// }
