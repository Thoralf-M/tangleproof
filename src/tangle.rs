use crate::error::Result;
use iota::{signing::Seed, Client, Message, MessageId, OutputId, UTXOInput};
use std::time::Duration;
use tokio::time::sleep;
/// Function to get the spent status of an outputid
pub async fn is_output_spent(output_id: &OutputId, url: &str) -> Result<bool> {
    let r = Client::build()
        .with_node(url)?
        .finish()?
        .get_output(
            &UTXOInput::new(*output_id.transaction_id(), 0).expect("Couldn't convert output"),
        )
        .await?;
    Ok(r.is_spent)
}

/// Function to send a transaction with an indexation payload
pub async fn send_transaction(
    indexation_tag: &str,
    data: &str,
    amount: u64,
    input: Option<OutputId>,
    node_url: &str,
    local_pow: bool,
    seed: &str,
) -> Result<(MessageId, Message)> {
    let client = Client::build()
        .with_node(node_url)?
        .with_local_pow(local_pow)
        .finish()?;

    let seed = Seed::from_ed25519_bytes(&hex::decode(seed)?).unwrap();

    let address = client
        .find_addresses(&seed)
        .with_account_index(0)
        .with_range(0..1)
        .finish()?;

    let mut message_builder = client
        .send()
        .with_seed(&seed)
        .with_output(&address[0].clone(), amount)?
        .with_index(indexation_tag)
        .with_data(data.as_bytes().to_vec());
    if let Some(input) = input {
        let utxo_input = UTXOInput::from(input);
        // Check if already spent before adding it
        let metadata = client.get_output(&utxo_input).await?;
        if metadata.is_spent {
            panic!("Output is already spent")
        }
        message_builder = message_builder.with_input(utxo_input);
    }
    let message_id = message_builder.finish().await?;

    let message = client.get_message().data(&message_id).await?;
    Ok((message_id, message))
}

/// Function to reattach or promote a transaction if it's unconfirmed
pub async fn retry(message_id: &MessageId, node_url: &str, local_pow: bool) -> Result<()> {
    let client = Client::build()
        .with_node(node_url)?
        .with_local_pow(local_pow)
        .finish()?;
    let mut latest_msg_id = *message_id;
    for _ in 0..40 {
        // Get the metadata to check if it needs to promote or reattach
        let message_metadata = client.get_message().metadata(&latest_msg_id).await?;
        if message_metadata.should_promote.unwrap_or(false) {
            println!("Promoted: {}", client.promote(&latest_msg_id).await?.0);
        } else if message_metadata.should_reattach.unwrap_or(false) {
            latest_msg_id = client.reattach(&latest_msg_id).await?.0;
            println!("Reattached: {} ", latest_msg_id);
        } else if let Some(state) = message_metadata.ledger_inclusion_state {
            println!("Leder inclustion state: {}", state);
            return Ok(());
        }
        sleep(Duration::from_secs(10)).await;
    }
    Ok(())
}
