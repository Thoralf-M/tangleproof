use config::Config;
use config::File;
use iota::client::Client;
use iota::BIP32Path;
use iota::Seed;
use std::io;
use std::path::Path;
use std::time::Duration;
use tangleproof::error::Result;
use tangleproof::proof::InclusionProof;
use tokio::time;

#[tokio::main]
async fn main() -> Result<()> {
    let mut settings: Config = Config::default();
    settings
        .merge(File::from(Path::new("config.json")))
        .unwrap();

    let node_url = settings.get_str("node_url").unwrap();
    let proof_name = settings.get_str("proof_name").unwrap();
    let amount = settings.get_int("amount").unwrap() as u64;
    let seed = settings.get_str("seed").unwrap();
    let bip32path = settings.get_str("bip32path").unwrap();

    println!("Your address is {}", get_address(&seed, &bip32path)?);

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
        &seed,
        &bip32path,
        &proof_name,
    )
    .await?;
    println!("Message sent: {}", msgid.to_string());
    println!("Transaction id in message: {}", txid.to_string());
    // Wait so the transaction can get confirmed so the output is available
    time::delay_for(Duration::from_secs(15)).await;
    // let proof = InclusionProof::from_file(proof_name).await?;
    println!("Proof is valid: {}", proof.is_valid(&node_url).await?);
    Ok(())
}

fn get_address(seed: &str, bip32path: &str) -> Result<String> {
    let client = Client::builder().node("http:localhost")?.build()?;
    let seed = Seed::from_ed25519_bytes(&hex::decode(seed)?).unwrap();
    let path = BIP32Path::from_str(&bip32path).unwrap();
    let address = client.find_addresses(&seed).path(&path).range(0..1).get()?;
    Ok(address[0].to_bech32())
}
