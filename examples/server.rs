//! cargo run --example server --release

use iota_client::Client;
use std::env;
use tangleproof::{chronist::Chronist, error::Result, server};
extern crate dotenv;
use dotenv::dotenv;

/// In this example we will create a server

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let chronist = Chronist::new(
        &env::var("DB_PATH").unwrap(),
        &env::var("IOTA_NODE").unwrap(),
        &Client::mnemonic_to_hex_seed(&env::var("MNEMONIC").unwrap())?,
    )
    .await?;

    server::start(chronist, 3030).await?;
    Ok(())
}
