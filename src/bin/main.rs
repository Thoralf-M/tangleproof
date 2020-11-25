use iota::Payload;
use tangleproof::error::Result;
use tangleproof::io;
use tangleproof::proof::InclusionProof;
use tangleproof::tangle::send_transaction;

#[tokio::main]
async fn main() -> Result<()> {
    let jsonproof = io::read_from_file("proof.json")?;
    // println!("{:?}", r);
    if let Some(p) = jsonproof{
        println!("{}", p.is_valid().await?)
    }
    let (id, msg) = send_transaction("Test").await?;
    println!("{:?}", id);
    // println!("{:?}", msg);
    if let Payload::Transaction(tx) = msg.payload().as_ref().expect("No payload") {
        let proof = InclusionProof::new(tx.essence().outputs()[0].clone(), msg);
        // println!("{:?}", proof);
        io::write_to_file("proof.json", proof)?;
    }
    Ok(())
}
