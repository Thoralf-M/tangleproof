use crate::{error::Result, iota_api::send_transaction, storage::RocksdbStorage};
use chrono::{DateTime, Utc};
use iota_client::bee_message::prelude::{
    Address, Essence, Message, MessageId, Output, OutputId, Payload,
};
use iota_client::{Client, Seed};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, str::FromStr, sync::Arc};
use tokio::{
    sync::{Mutex, RwLock},
    time::sleep,
};

const MESSAGE_IDS_KEY: &str = "message_ids";
const TRANSACTION_INDEX_KEY: &str = "transaction_index";
pub(crate) const TRANSACTION_MESSAGE_KEY: &str = "transaction_message";
const CHRONIST_INDEX: &str = "Chronist";
pub(crate) const INCLUSION_INDEX: &str = "inclusion_index";

pub(crate) const INCLUSION_STRUCTURE_ROWS: u64 = 10;
pub(crate) const INCLUSION_STRUCTURE_SECTION_LENGTH: u64 = 3;

pub struct Chronist {
    pub(crate) db: Arc<Mutex<RocksdbStorage>>,
    pub iota_client: Arc<Client>,
    pub(crate) message_ids: Arc<RwLock<HashSet<MessageId>>>,
    pub(crate) pending_message_ids: Arc<RwLock<HashSet<MessageId>>>,
    seed: String,
    sending_transacion: Arc<Mutex<()>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InclusionData {
    pub time: DateTime<Utc>,
    pub message_ids: Vec<MessageId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageWrapper {
    pub inclusion_position: Option<u64>,
    pub message: Message,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UtxoData {
    pub position_index: u64,
    pub message_id: MessageId,
}

impl Chronist {
    pub async fn new(path: &str, node_url: &str, seed: &str) -> Result<Self> {
        let db = Arc::new(Mutex::new(RocksdbStorage::new(path)?));
        let iota_client = Arc::new(Client::builder().with_node(node_url)?.finish().await?);

        // init transaction index
        let db_ = db.clone();
        let mut database = db_.lock().await;
        if database.get(TRANSACTION_INDEX_KEY).await.is_err() {
            crate::iota_api::split_funds(&iota_client, INCLUSION_STRUCTURE_ROWS, seed).await?;
            database.set(TRANSACTION_INDEX_KEY, "1".to_string()).await?;
            database.set(MESSAGE_IDS_KEY, "[]".to_string()).await?;
        }

        let message_ids: HashSet<MessageId> =
            serde_json::from_str(&database.get(MESSAGE_IDS_KEY).await?)?;

        let chronist = Self {
            db,
            iota_client,
            message_ids: Arc::new(RwLock::new(message_ids)),
            pending_message_ids: Arc::new(RwLock::new(HashSet::new())),
            seed: seed.to_owned(),
            sending_transacion: Arc::new(Mutex::new(())),
        };
        let chronist_ = Self {
            db: chronist.db.clone(),
            iota_client: chronist.iota_client.clone(),
            message_ids: chronist.message_ids.clone(),
            pending_message_ids: chronist.pending_message_ids.clone(),
            seed: seed.to_owned(),
            sending_transacion: chronist.sending_transacion.clone(),
        };
        chronist_.start_sending_transactions();
        Ok(chronist)
    }

    fn start_sending_transactions(self) {
        let pending_message_ids = self.pending_message_ids.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(async {
                loop {
                    let latest_transaction_index = {
                        let db = self.db.lock().await;
                        u64::from_str(
                            &db.get(TRANSACTION_INDEX_KEY)
                                .await
                                .expect("Couldn't get transaction index"),
                        )
                        .expect("Couldn't convert transaction index")
                    };
                    let row = crate::inclusion_structure::get_row_for_position(
                        latest_transaction_index,
                        INCLUSION_STRUCTURE_ROWS,
                        INCLUSION_STRUCTURE_SECTION_LENGTH,
                    );
                    // Only include messages in row 0
                    let message_ids: Vec<MessageId> = if row == 0 {
                        if pending_message_ids.read().await.is_empty() {
                            sleep(std::time::Duration::from_secs(10)).await;
                            continue;
                        }
                        pending_message_ids
                            .read()
                            .await
                            .clone()
                            .iter()
                            // Up to 100 message ids per transaction to stay below 10000 bytes length for faster PoW
                            // even with 10 in and outputs https://gist.github.com/Wollac/82d211781535ad95d39c7db7ae093204
                            .take(100)
                            .cloned()
                            .collect()
                    } else {
                        vec![]
                    };

                    match self.send_transaction(message_ids.clone()).await {
                        Ok(r) => {
                            println!("Transaction {} sent {}", latest_transaction_index, r);
                            // remove message ids that got sent
                            let mut pending_message_ids = pending_message_ids.write().await;
                            for message_id in message_ids {
                                pending_message_ids.remove(&message_id);
                            }
                        }
                        Err(e) => println!("{}", e),
                    }
                    sleep(std::time::Duration::from_secs(10)).await;
                }
            });
        });
    }
    async fn send_transaction(&self, message_ids: Vec<MessageId>) -> Result<MessageId> {
        println!("send_transaction start");
        // lock sending_transaction so no conflicts are generated
        self.sending_transacion.lock().await;

        let mut latest_transaction_index = {
            let db = self.db.lock().await;
            u64::from_str(&db.get(TRANSACTION_INDEX_KEY).await?)?
        };

        let inclusion_data = InclusionData {
            time: Utc::now(),
            message_ids,
        };

        // get inputs
        let input_indexes =
            crate::inclusion_structure::get_previous_indexes_for_each_row_at_position(
                latest_transaction_index,
                INCLUSION_STRUCTURE_ROWS,
                INCLUSION_STRUCTURE_SECTION_LENGTH,
            );

        let iota_client = self.iota_client.clone();
        let addresses: Vec<Address> = iota_client
            .get_addresses(&Seed::from_bytes(&hex::decode(&self.seed)?))
            .with_range(0..crate::chronist::INCLUSION_STRUCTURE_ROWS as usize)
            .get_all_raw()
            .await?
            .into_iter()
            .filter(|a| !a.1)
            .map(|a| a.0)
            .collect();

        let mut inputs = Vec::new();
        for input in input_indexes {
            let database = self.db.lock().await;
            let position_data: UtxoData = serde_json::from_str(
                &database
                    .get(&format!("{}{}", INCLUSION_INDEX, input.0))
                    .await?,
            )?;
            let message: Message = serde_json::from_str(
                &database
                    .get(&format!(
                        "{}{}",
                        TRANSACTION_MESSAGE_KEY,
                        position_data.message_id.to_string()
                    ))
                    .await?,
            )?;

            if let Some(Payload::Transaction(tx)) = message.payload() {
                let Essence::Regular(essence) = tx.essence();
                if essence.outputs().len() < input.1 as usize {
                    return Err(crate::error::Error::UtxoInputNotFound);
                }
                // todo return error if output doesn't exists
                let output_position = essence
                    .outputs()
                    .iter()
                    .position(|output| {
                        let address = match output {
                            Output::Treasury(_) => {
                                panic!("Treasury output is not supported");
                            }
                            Output::SignatureLockedSingle(ref r) => match &r.address() {
                                Address::Ed25519(addr) => Address::Ed25519(*addr),
                            },
                            Output::SignatureLockedDustAllowance(ref r) => match &r.address() {
                                Address::Ed25519(addr) => Address::Ed25519(*addr),
                            },
                        };
                        addresses[input.1 as usize] == address
                    })
                    .expect("No output with address");
                inputs.push(OutputId::new(tx.id(), output_position as u16)?);
            } else {
                return Err(crate::error::Error::UtxoInputNotFound);
            }
        }

        // Send new transaction with message
        let transaction_message = send_transaction(
            &iota_client,
            CHRONIST_INDEX,
            &serde_json::to_string(&inclusion_data)?,
            Some(inputs),
            &self.seed,
            latest_transaction_index,
        )
        .await?;

        let utxo_data = UtxoData {
            position_index: latest_transaction_index,
            message_id: transaction_message.id().0,
        };

        let mut database = self.db.lock().await;
        // store new utxo data
        database
            .set(
                &format!("{}{}", INCLUSION_INDEX, latest_transaction_index),
                serde_json::to_string(&utxo_data)?,
            )
            .await?;

        // store new transaction_message
        database
            .set(
                &format!(
                    "{}{}",
                    TRANSACTION_MESSAGE_KEY,
                    transaction_message.id().0.to_string()
                ),
                serde_json::to_string(&transaction_message)?,
            )
            .await?;

        // store message ids and update inclusion_position
        let mut message_ids = self.message_ids.write().await;
        for message_id in inclusion_data.message_ids {
            message_ids.insert(message_id.to_owned());
            // update inclusion_position
            let mut message_without_inclusion_position: MessageWrapper =
                serde_json::from_str(&database.get(&message_id.to_string()).await?)?;
            message_without_inclusion_position.inclusion_position = Some(latest_transaction_index);
            database
                .set(
                    &message_id.to_string(),
                    serde_json::to_string(&message_without_inclusion_position)?,
                )
                .await?;
        }

        database
            .set(MESSAGE_IDS_KEY, serde_json::to_string(&*message_ids)?)
            .await?;

        // update transaction index
        latest_transaction_index += 1;
        // store transaction_index
        database
            .set(TRANSACTION_INDEX_KEY, latest_transaction_index.to_string())
            .await?;

        drop(database);
        let _ = iota_client
            .retry_until_included(&transaction_message.id().0, None, None)
            .await?;
        Ok(transaction_message.id().0)
    }

    pub async fn save_message(&self, message_id: &str) -> Result<()> {
        let msg_id = MessageId::from_str(message_id)?;
        {
            if self.message_ids.read().await.contains(&msg_id)
                || self.pending_message_ids.read().await.contains(&msg_id)
            {
                println!("Message {} already stored or pending", msg_id);
                return Ok(());
            }
        }
        let message = self.iota_client.get_message().data(&msg_id).await?;

        // store new message
        let mut database = self.db.lock().await;
        database
            .set(
                message_id,
                serde_json::to_string(&MessageWrapper {
                    inclusion_position: None,
                    message,
                })?,
            )
            .await?;

        // add message_id to pending_message_ids so it gets included in transactions
        self.pending_message_ids.write().await.insert(msg_id);

        Ok(())
    }

    pub async fn get_message_proof(
        &self,
        message_id: &str,
    ) -> Result<crate::inclusion_proof::InclusionProof> {
        let database = self.db.lock().await;
        let message_wrapper = database.get(message_id).await?;
        let message_wrapper: MessageWrapper = serde_json::from_str(&message_wrapper)?;

        let inclusion_position = match message_wrapper.inclusion_position {
            Some(position) => position,
            None => return Err(crate::error::Error::InclusionPositionNotSet),
        };
        let input_positions = crate::inclusion_structure::get_path(
            inclusion_position,
            u64::from_str(&database.get(TRANSACTION_INDEX_KEY).await?)? - 1,
            INCLUSION_STRUCTURE_ROWS,
            INCLUSION_STRUCTURE_SECTION_LENGTH,
        );

        let mut path_transactions = Vec::new();
        for input_positions in input_positions {
            let position_data: UtxoData = serde_json::from_str(
                &database
                    .get(&format!("{}{}", INCLUSION_INDEX, input_positions.0))
                    .await?,
            )?;

            let position_message: Message = serde_json::from_str(
                &database
                    .get(&format!(
                        "{}{}",
                        TRANSACTION_MESSAGE_KEY,
                        position_data.message_id.to_string(),
                    ))
                    .await?,
            )?;
            let path_tx = match position_message.payload() {
                Some(Payload::Transaction(tx)) => tx,
                _ => return Err(crate::error::Error::NoTransactionPayload),
            };
            path_transactions.push(*path_tx.clone());
        }
        drop(database);
        let latest_output_id = {
            let tx = path_transactions
                .last()
                .expect("No transactions for proof available ");
            let Essence::Regular(essence) = tx.essence();

            let addresses: Vec<Address> = self
                .iota_client
                .get_addresses(&Seed::from_bytes(&hex::decode(&self.seed)?))
                .with_range(0..crate::chronist::INCLUSION_STRUCTURE_ROWS as usize)
                .get_all_raw()
                .await?
                .into_iter()
                .filter(|a| !a.1)
                .map(|a| a.0)
                .collect();
            // Get output id from with highest address/row index because they will stay valid the longest time
            let output_position = essence
                .outputs()
                .iter()
                .position(|output| {
                    let address = match output {
                        Output::Treasury(_) => {
                            panic!("Treasury output is not supported");
                        }
                        Output::SignatureLockedSingle(ref r) => match &r.address() {
                            Address::Ed25519(addr) => Address::Ed25519(*addr),
                        },
                        Output::SignatureLockedDustAllowance(ref r) => match &r.address() {
                            Address::Ed25519(addr) => Address::Ed25519(*addr),
                        },
                    };
                    addresses[essence.outputs().len() - 1] == address
                })
                .expect("No output with address");

            OutputId::new(tx.id(), output_position as u16)?
        };

        let inclusion_proof = crate::inclusion_proof::InclusionProof {
            // get output id with the highest index, because that will stay the longest time available
            latest_output_id,
            message: message_wrapper.message,
            transactions: path_transactions,
        };

        println!(
            "Is valid: {}",
            inclusion_proof.is_valid(&self.iota_client).await?
        );
        Ok(inclusion_proof)
    }

    pub async fn get_message(&self, message_id: &str) -> Result<MessageWrapper> {
        let message = self.db.lock().await.get(message_id).await?;
        let message: MessageWrapper = serde_json::from_str(&message)?;
        Ok(message)
    }

    pub async fn get_message_ids(&self) -> Result<Vec<String>> {
        let message_ids = self.db.lock().await.get(MESSAGE_IDS_KEY).await?;
        let message_ids: Vec<String> = serde_json::from_str(&message_ids)?;
        Ok(message_ids)
    }
}
