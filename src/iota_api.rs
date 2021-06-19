use crate::error::Result;
use iota_client::{
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
    println!("inclusion_position {}", inclusion_position);
    for row in 0..row_for_position + 1 {
        println!("row {} output_address {}", row, addresses[row as usize]);
        message_builder =
            message_builder.with_output(&addresses[row as usize].clone(), IOTA_AMOUNT)?;
    }
    // For first tx in row
    if inclusion_position
        == crate::inclusion_structure::get_row_starting_position(
            row_for_position,
            crate::chronist::INCLUSION_STRUCTURE_SECTION_LENGTH,
        )
    {
        println!(
            "first time row {} inclusion_position: {}",
            row_for_position, inclusion_position
        );
        let outputs = client
            .get_address()
            .outputs(
                &addresses[row_for_position as usize],
                OutputsOptions::default(),
            )
            .await?;
        // todo return error if empty
        message_builder = message_builder.with_input(outputs[0].clone());
    }

    if let Some(inputs) = inputs {
        println!("Inputs: {:?}", inputs);
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

    while client.get_balance(&seed).finish().await? < rows * 1_000_000 {
        println!("Send {}i to {}", rows * 1_000_000, addresses_from_seed[0]);
        sleep(std::time::Duration::from_secs(10)).await;
    }

    let mut message_builder = client.message().with_seed(&seed);
    for i in 0..rows {
        message_builder =
            message_builder.with_output(&addresses_from_seed[i as usize], 1_000_000)?;
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
