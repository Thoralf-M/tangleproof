use iota::OutputId;
use iota::Payload;
use tangleproof::error::Result;
use tangleproof::io;
use tangleproof::proof::InclusionProof;
use tangleproof::tangle::send_transaction;

#[tokio::main]
async fn main() -> Result<()> {
    let node_url = "http://192.168.178.22:14265";
    let proof_name = "proof.json";
    let message = "Testnachricht";
    let indexation_tag = "InclusionProof";
    match io::read_from_file(proof_name)? {
        Some(proof) => {
            println!("Proof is valid: {}", proof.is_valid(node_url).await?)
        }
        _ => println!("No proof available"),
    }
    let (id, msg) = send_transaction(indexation_tag, message, 2779530283277761, node_url).await?;
    println!("{:?}", id);
    if let Payload::Transaction(tx) = msg.payload().as_ref().expect("No payload") {
        println!("txid{:?}", tx.id());
        let endproof = match io::read_from_file(proof_name)? {
            Some(mut proof) => {
                // Update existing proof
                proof.latest_output_id = OutputId::new(tx.id(), 0).expect("Can't get output id");
                proof.messages.push(msg.clone());
                proof
            }
            _ => {
                // Create a new proof
                InclusionProof::new(OutputId::new(tx.id(), 0).expect("Can't get output id"), msg)
            }
        };
        io::write_to_file(proof_name, endproof)?;
    }
    Ok(())
}
