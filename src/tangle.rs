use crate::error::{Error, Result};
// use bee_common::packable::Packable;
use iota::common::packable::Packable;
use iota::signing::{binary::Ed25519PrivateKey, Seed, Signer};
use iota::{
    BIP32Path, Client, ClientMiner, Ed25519Signature, Indexation, Input, Message, MessageBuilder,
    MessageId, Output, OutputId, Payload, SignatureLockedSingleOutput, SignatureUnlock,
    TransactionBuilder, TransactionEssenceBuilder, UTXOInput, UnlockBlock,
};
use std::num::NonZeroU64;
/// Function to get the spent status of an outputid
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

/// Function to send a transaction with an indexation payload
pub async fn send_transaction(
    indexation_tag: &str,
    data: &str,
    amount: u64,
    input: Option<OutputId>,
    node_url: &str,
    local_pow: bool,
    seed: &str,
    bip32path: &str,
) -> Result<(MessageId, Message)> {
    let client = Client::builder()
        .node(node_url)?
        .local_pow(local_pow)
        .build()?;
    let seed = Seed::from_ed25519_bytes(&hex::decode(seed)?).unwrap();

    let mut path = BIP32Path::from_str(bip32path).unwrap();

    let address = client.find_addresses(&seed).path(&path).range(0..1).get()?;

    let output_address = address[0].clone();
    let input = match input {
        Some(input) => {
            let utxo_input = UTXOInput::from(input);
            // Check if already spent before adding it
            let metadata = client.get_output(&utxo_input).await?;
            if metadata.is_spent {
                panic!("Output is already spent")
            }
            Input::UTXO(utxo_input)
        }
        _ => {
            let address_outputs = client
                .get_address()
                .outputs(&output_address.clone())
                .await?;

            let mut outputs = vec![];
            for output_id in address_outputs.iter() {
                let curr_outputs = client.get_output(output_id).await?;
                outputs.push(curr_outputs);
            }
            if outputs.is_empty() {
                panic!("No ouput available")
            }
            let mut final_output = None;
            for (index, output) in outputs.into_iter().enumerate() {
                if !output.is_spent && output.amount >= amount {
                    final_output = Some(address_outputs[index].clone());
                }
            }
            match final_output {
                Some(output) => Input::UTXO(output),
                _ => panic!("Not enough balance"),
            }
        }
    };

    let output = Output::from(SignatureLockedSingleOutput::new(
        output_address,
        NonZeroU64::new(amount).expect("Couldn't create NonZeroU64 amount"),
    ));
    let essence = TransactionEssenceBuilder::new()
        .add_input(input)
        .add_output(output)
        .with_payload(Payload::Indexation(Box::new(
            Indexation::new(indexation_tag.to_string(), data.as_bytes()).unwrap(),
        )))
        .finish()
        .unwrap();
    let mut serialized_essence = vec![];
    essence.pack(&mut serialized_essence).unwrap();

    let unlock_block;
    match &seed {
        Seed::Ed25519(s) => {
            const HARDEND: u32 = 1 << 31;
            path.push(0 as u32 + HARDEND);
            let private_key = Ed25519PrivateKey::generate_from_seed(&s, &path)
                .map_err(|_| Error::InvalidParameter("seed inputs".to_string()))?;
            let public_key = private_key.generate_public_key().to_bytes();
            // The block should sign the entire transaction essence part of the transaction payload
            let signature = Box::new(private_key.sign(&serialized_essence).to_bytes());
            unlock_block = UnlockBlock::Signature(SignatureUnlock::Ed25519(Ed25519Signature::new(
                public_key, signature,
            )));
        }
        Seed::Wots(_) => panic!("Wots signing scheme isn't supported."),
    }

    let transaction = TransactionBuilder::new()
        .with_essence(essence)
        .add_unlock_block(unlock_block)
        .finish()
        .unwrap();

    let tips = client.get_tips().await.unwrap();
    let message = MessageBuilder::<ClientMiner>::new()
        .with_network_id(6530425480034647824)
        .with_parent1(tips.0)
        .with_parent2(tips.1)
        .with_payload(Payload::Transaction(Box::new(transaction)))
        .with_nonce_provider(client.get_pow_provider(), 4000f64)
        .finish()
        .unwrap();
    let message_id = client.post_message(&message).await?;

    Ok((message_id, message))
}
