use crate::error::Result;
use iota_client::{
    api::ClientMessageBuilder,
    bee_message::prelude::{Message, OutputId, UtxoInput},
    node::OutputsOptions,
    Client, Seed,
};
use tokio::time::sleep;

const IOTA_AMOUNT: u64 = 1_000_000;

/// Function to get an outputid
pub async fn is_output_known(client: &Client, output_id: &OutputId) -> bool {
    matches!(client.get_output(&UtxoInput::from(*output_id)).await, Ok(_))
}

/// Function to send a transaction with an indexation payload
pub async fn send_transaction(
    client: &Client,
    indexation_tag: &str,
    data: &str,
    inputs: Option<Vec<OutputId>>,
    seed: &str,
    inclusion_position: u64,
) -> Result<Message> {
    let seed = Seed::from_bytes(&hex::decode(seed)?);

    let addresses = client
        .get_addresses(&seed)
        .with_range(0..crate::chronist::INCLUSION_STRUCTURE_ROWS as usize)
        .finish()
        .await?;

    let mut message_builder = client.message().with_seed(&seed);

    message_builder = message_builder
        .with_index(indexation_tag)
        .with_data(data.as_bytes().to_vec());
    // For first transaction in row we have to get the input manually from the address
    let row_for_position = crate::inclusion_structure::get_row_for_position(
        inclusion_position,
        crate::chronist::INCLUSION_STRUCTURE_ROWS,
        crate::chronist::INCLUSION_STRUCTURE_SECTION_LENGTH,
    );
    for row in 0..row_for_position + 1 {
        message_builder =
            message_builder.with_output(&addresses[row as usize].clone(), IOTA_AMOUNT)?;
    }
    // For first tx in row get outputs from the address
    if inclusion_position
        == crate::inclusion_structure::get_row_starting_position(
            row_for_position,
            crate::chronist::INCLUSION_STRUCTURE_SECTION_LENGTH,
        )
    {
        let outputs = client
            .get_address()
            .outputs(
                &addresses[row_for_position as usize],
                OutputsOptions::default(),
            )
            .await?;
        if outputs.is_empty() {
            return Err(crate::error::Error::UtxoInputNotFound);
        }
        let mut output_index = 0;
        for (index, output) in outputs.iter().enumerate() {
            let output_data = client.get_output(&output).await?;
            let (amount, _, _) =
                ClientMessageBuilder::get_output_amount_and_address(&output_data.output)?;
            if amount == IOTA_AMOUNT {
                output_index = index;
            }
        }
        message_builder = message_builder.with_input(outputs[output_index].clone());
    }

    if let Some(inputs) = inputs {
        for input in inputs {
            message_builder = message_builder.with_input(UtxoInput::from(input));
        }
    }
    let message = message_builder.finish().await?;
    Ok(message)
}

/// Function to split initial funds
pub async fn split_funds(client: &Client, rows: u64, seed: &str) -> Result<Message> {
    let seed = Seed::from_bytes(&hex::decode(seed)?);

    let addresses_from_seed = client
        .get_addresses(&seed)
        .with_range(0..rows as usize)
        .finish()
        .await?;

    while client.get_balance(&seed).finish().await? < rows * IOTA_AMOUNT {
        println!("Send {}i to {}", rows * IOTA_AMOUNT, addresses_from_seed[0]);
        sleep(std::time::Duration::from_secs(10)).await;
    }

    let mut message_builder = client.message().with_seed(&seed);
    for i in 0..rows {
        message_builder =
            message_builder.with_output(&addresses_from_seed[i as usize], IOTA_AMOUNT)?;
    }
    let message = message_builder.with_index("Chronist").finish().await?;

    println!(
        "Split transaction sent: https://explorer.iota.org/testnet/message/{}",
        message.id().0
    );
    let _ = client
        .retry_until_included(&message.id().0, None, None)
        .await?;
    Ok(message)
}
