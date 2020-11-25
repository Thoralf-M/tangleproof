use crate::error::Result;
use bee_common::packable::Packable;
use iota::signing::{
    binary::{BIP32Path, Ed25519PrivateKey},
    seed::Seed,
    Signer,
};
use iota::Indexation;
use iota::{
    hex_to_address, hex_to_message_id, hex_to_transaction_id, Address, Client, Ed25519Address,
    Ed25519Signature, Message, MessageId, MessageJson, Output, Payload,
    SignatureLockedSingleOutput, SignatureUnlock, TransactionBuilder, TransactionEssenceBuilder,
    UTXOInput, UnlockBlock,
};
use std::convert::From;
use std::num::NonZeroU64;

pub async fn send() -> Result<(String, String)> {
    let client = Client::new()
        .nodes(&vec!["http://localhost:14265"])
        .unwrap()
        .build()
        .unwrap();

    let index = Indexation::new(String::from("Hello"), String::from("Tangle").as_bytes()).unwrap();

    let tips = client.get_tips().await.unwrap();

    let message = Message::builder()
        .with_parent1(tips.0)
        .with_parent2(tips.1)
        .with_payload(Payload::Indexation(Box::new(index.clone())))
        .with_payload(Payload::Indexation(Box::new(index)))
        .finish()
        .unwrap();
    println!("message: {:?}", message);
    let sent_msg = client.post_message(&message).await.unwrap();

    println!("MessageId {}", sent_msg);

    let fetched_messages = client.get_message().index(&"Hello").await.unwrap();

    println!("{:#?}", fetched_messages);

    let r = client
        .get_message()
        .data(&hex_to_message_id(fetched_messages[0]).unwrap())
        .await
        .unwrap();

    println!("{:#?}", r);
    if let Payload::Indexation(i) = r.payload().as_ref().unwrap() {
        println!(
            "Data: {}",
            String::from_utf8(hex::decode(i.data()).unwrap()).expect("Found invalid UTF-8")
        );
    }
    let utxo = "String".into();
    Ok((utxo, serde_json::to_string(&message)?))
}

pub async fn fetch(_message_id: &Output) -> Result<String> {
    let r = Client::new()
        .node("http://localhost:14265")
        .unwrap()
        .build()
        .unwrap()
        .get_output(
            &UTXOInput::new(
                hex_to_transaction_id(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .unwrap(),
                0,
            )
            .unwrap(),
        )
        .await
        .unwrap();

    println!("output spent: {:#?}", r.is_spent);
    Ok("not done".into())
}

pub async fn send_transaction(data: &str) -> Result<(MessageId, Message)> {
    let client = Client::new()
        .node("http://localhost:14265")
        .unwrap()
        .build()
        .unwrap();
    let private_key = Ed25519PrivateKey::from_bytes(
        &hex::decode("256a818b2aac458941f7274985a410e57fb750f3a3a67969ece5bd9ae7eef5b2").unwrap(),
    )
    .unwrap();
    let public_key = private_key.generate_public_key().to_bytes();

    let mut output_address = [0u8; 32];
    hex::decode_to_slice(
        "6920b176f613ec7be59e68fc68f597eb3393af80f74c7c3db78198147d5f1f92",
        &mut output_address,
    )
    .unwrap();
    let output_address = Ed25519Address::new(output_address);
    let inputs = client
        .get_address()
        .outputs(&output_address.clone().into())
        .await
        .unwrap();

    // let address = client
    //     .get_unspent_address(&Seed::from_ed25519_bytes(&[0u8; 32]).unwrap())
    //     .path(&BIP32Path::from_str("m").unwrap())
    //     .get()
    //     .await
    //     .unwrap();
    let output = Output::from(SignatureLockedSingleOutput::new(
        // address.0,
        output_address.into(),
        NonZeroU64::new(2779530283277761).unwrap(),
    ));
    let essence = TransactionEssenceBuilder::new()
        .add_input(inputs[0].clone().into())
        .add_output(output)
        .with_payload(Payload::Indexation(Box::new(
            Indexation::new(String::from("Hello"), data.as_bytes()).unwrap(),
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

    println!("{:?}", transaction);
    println!("essence: {:#?}", transaction.essence());
    let tips = client.get_tips().await.unwrap();
    let message = Message::builder()
        .with_parent1(tips.0)
        .with_parent2(tips.1)
        .with_payload(Payload::Transaction(Box::new(transaction)))
        .finish()
        .unwrap();

    // let me: MessageJson = message.into();
    // println!("{}", me);
    let hash = client.post_message(&message).await.unwrap();

    Ok((hash, message))
}