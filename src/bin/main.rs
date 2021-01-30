extern crate dotenv;
use dotenv::dotenv;
use iota::{client::Client, Seed};
use std::env;
use std::{io, time::Duration};
use tangleproof::{error::Result, proof::InclusionProof, tangle::retry};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    // create_proof().await?;
    check_proof().await?;
    Ok(())
}

#[allow(dead_code)]
async fn create_proof() -> Result<()> {
    let node_url = env::var("node_url").unwrap();
    let proof_name = env::var("proof_name").unwrap();
    let amount: u64 = env::var("amount").unwrap().parse().unwrap();
    let seed = env::var("seed").unwrap();
    let local_pow: bool = env::var("local_pow").unwrap().parse().unwrap();

    println!("Your address is {}", get_address(&seed)?);
    println!("Send {}i to it before you continue", amount);

    println!("Enter indexation tag");
    let mut indexation_tag = String::new();
    io::stdin().read_line(&mut indexation_tag).unwrap();
    println!("Enter message");
    let mut message = String::new();
    io::stdin().read_line(&mut message).unwrap();

    println!("Sending transaction...");

    let (msgid, txid, proof) = InclusionProof::send_data(
        &indexation_tag,
        &message,
        amount,
        &node_url,
        local_pow,
        &seed,
        &proof_name,
    )
    .await?;
    println!("Message sent: {}", msgid.to_string());
    println!("Transaction id in message: {}", txid.to_string());
    sleep(Duration::from_secs(5)).await;
    retry(&msgid, &node_url, local_pow).await?;
    println!("Proof is valid: {}", proof.is_valid(&node_url).await?);
    Ok(())
}

#[allow(dead_code)]
async fn check_proof() -> Result<()> {
    let node_url = env::var("node_url").unwrap();
    let proof_name = env::var("proof_name").unwrap();

    let proof = InclusionProof::from_file(&proof_name).await?;
    println!("Proof is valid: {}", proof.is_valid(&node_url).await?);
    Ok(())
}

fn get_address(seed: &str) -> Result<String> {
    let client = Client::builder().with_node("http:localhost")?.finish()?;
    let seed = Seed::from_ed25519_bytes(&hex::decode(seed)?).unwrap();
    let address = client
        .find_addresses(&seed)
        .with_account_index(0)
        .with_range(0..1)
        .finish()?;
    Ok(address[0].to_string())
}
