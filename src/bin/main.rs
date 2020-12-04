use std::time::Duration;
use tangleproof::error::Result;
use tangleproof::proof::InclusionProof;
use tokio::time;

#[tokio::main]
async fn main() -> Result<()> {
    let node_url = "http://192.168.178.22:14265";
    let proof_name = "proof.json";
    let message = "Testnachricht";
    let indexation_tag = "InclusionProof";
    let amount = 2779530283277761;
    let (msgid, txid, proof) =
        InclusionProof::send_data(indexation_tag, message, amount, node_url, proof_name).await?;
    println!("Message sent: {}", msgid.to_string());
    println!("Transaction id in message: {}", txid.to_string());
    // Wait so the transaction can get confirmed so the output is available
    time::delay_for(Duration::from_secs(15)).await;
    println!("Proof is valid: {}", proof.is_valid(node_url).await?);
    Ok(())
}
