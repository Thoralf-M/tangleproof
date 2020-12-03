use iota::OutputId;
use iota::Payload;
use tangleproof::error::Result;
use tangleproof::io;
use tangleproof::proof::InclusionProof;
use tangleproof::tangle::send_transaction;

#[tokio::main]
async fn main() -> Result<()> {
    let jsonproof = io::read_from_file("proof.json")?;
    // println!("{:?}", r);
    // println!("{:?}",jsonproof);
    if let Some(p) = jsonproof.clone() {
        println!("Proof object is valid: {}", p.is_valid().await?)
    }
    // let (id, msg) = send_transaction("Test").await?;
    // println!("{:?}", id);
    // // println!("{:?}", msg);
    // if let Payload::Transaction(tx) = msg.payload().as_ref().expect("No payload") {
    //     println!("txid{:?}", tx.id());
    //     let endproof = match jsonproof {
    //         Some(mut proof) => {
    //             proof.latest_output_id = OutputId::new(tx.id(), 0).expect("Can't get output id");
    //             proof.messages.push(msg.clone());
    //             proof
    //         }
    //         _ => {
    //             let proof = InclusionProof::new(
    //                 OutputId::new(tx.id(), 0).expect("Can't get output id"),
    //                 msg,
    //             );
    //             proof
    //         }
    //     };
    //     // println!("{:?}", proof);
    //     io::write_to_file("proof.json", endproof)?;
    // }
    Ok(())
}
