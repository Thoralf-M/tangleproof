use crate::error::Result;
use iota::Indexation;
use iota::{BIP32Path, Client, Message, MessageId, OutputId, Seed, UTXOInput};
use std::num::NonZeroU64;

pub async fn is_output_spent(output_id: &OutputId, url: &str) -> Result<bool> {
    let r = Client::builder()
        .node(url)?
        .build()?
        .get_output(
            &UTXOInput::new(*output_id.transaction_id(), 0).expect("Couldn't convert output"),
        )
        .await?;
    Ok(r.is_spent)
}

pub async fn send_transaction(
    indexation_tag: &str,
    data: &str,
    amount: u64,
    node_url: &str,
    seed: &str,
    bip32path: &str,
) -> Result<(MessageId, Message)> {
    let client = Client::builder().node(node_url)?.build()?;
    let seed = Seed::from_ed25519_bytes(&hex::decode(seed)?).unwrap();

    let path = BIP32Path::from_str(bip32path).unwrap();

    let address = client.find_addresses(&seed).path(&path).range(0..1).get()?;
    let messageid = client
        .send(&seed)
        .path(&path)
        .output(address[0].clone(), NonZeroU64::new(amount).unwrap())
        .indexation(Indexation::new(indexation_tag.to_string(), data.as_bytes()).unwrap())
        .post()
        .await?;
    let message = client.get_message().data(&messageid).await?;
    Ok((message.id(), message))
}
