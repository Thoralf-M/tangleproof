use crate::error::Result;
use bee_common::packable::Packable;
use iota::signing::{binary::Ed25519PrivateKey, Signer};
use iota::Indexation;
use iota::{
    Client, Ed25519Address, Ed25519Signature, Message, MessageId, Output, OutputId, Payload,
    SignatureLockedSingleOutput, SignatureUnlock, TransactionBuilder, TransactionEssenceBuilder,
    UTXOInput, UnlockBlock,
};
use std::convert::From;
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
    url: &str,
) -> Result<(MessageId, Message)> {
    let client = Client::builder().node(url)?.build()?;
    let private_key = Ed25519PrivateKey::from_bytes(&hex::decode(
        "256a818b2aac458941f7274985a410e57fb750f3a3a67969ece5bd9ae7eef5b2",
    )?)
    .unwrap();
    let public_key = private_key.generate_public_key().to_bytes();

    let mut output_address = [0u8; 32];
    hex::decode_to_slice(
        "6920b176f613ec7be59e68fc68f597eb3393af80f74c7c3db78198147d5f1f92",
        &mut output_address,
    )?;
    let output_address = Ed25519Address::new(output_address);
    let inputs = client
        .get_address()
        .outputs(&output_address.clone().into())
        .await?;
    let output = Output::from(SignatureLockedSingleOutput::new(
        output_address.into(),
        NonZeroU64::new(amount).expect("Couldn't create NonZeroU64 amount"),
    ));
    let essence = TransactionEssenceBuilder::new()
        .add_input(inputs[0].clone().into())
        .add_output(output)
        .with_payload(Payload::Indexation(Box::new(
            Indexation::new(indexation_tag.to_string(), data.as_bytes()).unwrap(),
        )))
        .finish()
        .unwrap();
    let mut serialized_essence = vec![];
    essence.pack(&mut serialized_essence).unwrap();

    let signature = Box::new(private_key.sign(&serialized_essence).to_bytes());
    let unlock = UnlockBlock::Signature(SignatureUnlock::Ed25519(Ed25519Signature::new(
        public_key, signature,
    )));

    let transaction = TransactionBuilder::new()
        .with_essence(essence)
        .add_unlock_block(unlock)
        .finish()
        .unwrap();

    // println!("{:?}", transaction);
    // println!("essence: {:#?}", transaction.essence());
    let tips = client.get_tips().await.unwrap();
    let message = Message::builder()
        .with_network_id(0)
        .with_parent1(tips.0)
        .with_parent2(tips.1)
        .with_payload(Payload::Transaction(Box::new(transaction)))
        .finish()
        .unwrap();

    let hash = client.post_message(&message).await?;

    Ok((hash, message))
}
