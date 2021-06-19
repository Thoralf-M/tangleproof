//! cargo run --example mqtt_spam --release

use iota_client::{Client, MqttEvent, Topic};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{mpsc::channel, Arc, Mutex};
use std::time::Duration;
use tangleproof::{chronist::Chronist, error::Result, server, server::MessageIdResponse};
use tokio::time::sleep;
extern crate dotenv;
use dotenv::dotenv;
use reqwest;

/// In this example we will save messages from mqtt

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let chronist = Chronist::new(
        &"maindb",
        &env::var("IOTA_NODE").unwrap(),
        &"256a818b2aac458941f2274945a410e57fb750f3a3a67969ece5bd9ae7eef5b2",
    )
    .await?;

    tokio::spawn(async move {
        server::start(chronist, 3030).await.unwrap();
    });

    let mut iota = Client::builder()
        .with_node("https://api.hornet-0.testnet.chrysalis2.com/")?
        .finish()
        .await?;

    let (tx, rx) = channel();
    let tx = Arc::new(Mutex::new(tx));

    let mut event_rx = iota.mqtt_event_receiver();
    tokio::spawn(async move {
        while event_rx.changed().await.is_ok() {
            let event = event_rx.borrow();
            if *event == MqttEvent::Disconnected {
                println!("mqtt disconnected");
                std::process::exit(1);
            }
        }
    });

    iota.subscriber()
        .with_topics(vec![Topic::new("messages/referenced").unwrap()])
        .subscribe(move |event| match event.topic.as_str() {
            "messages/referenced" => {
                #[derive(Serialize, Deserialize, Debug)]
                pub struct Referenced {
                    #[serde(rename = "messageId")]
                    pub message_id: String,
                }
                let message: Referenced = serde_json::from_str(&event.payload).unwrap();
                tx.lock().unwrap().send(message.message_id).unwrap();
            }
            _ => println!("{:?}", event),
        })
        .await
        .unwrap();

    for outer in 0..200 {
        println!("outer loop {}", outer);
        let mut tasks = Vec::new();
        for _ in 0..10 {
            let message_id = rx.recv().unwrap();
            tasks.push(async move {
                tokio::spawn(async move {
                    let _resp =
                        reqwest::get(format!("http://localhost:3030/proof/create/{}", message_id))
                            .await
                            .unwrap()
                            .json::<MessageIdResponse>()
                            .await
                            .unwrap();
                    // println!("{:#?}", resp);
                })
                .await
            });
        }
        let _results = futures::future::try_join_all(tasks)
            .await
            .expect("failed to sync addresses");

        sleep(Duration::from_secs(10)).await;

        if outer % 10 == 0 {
            // let tips = chronist_
            //     .read()
            //     .await
            //     .iota_client
            //     .read()
            //     .await
            //     .get_tips()
            //     .await
            //     .unwrap();

            // let now = std::time::Instant::now();
            // chronist_
            //     .read()
            //     .await
            //     .save_message(&tips[0].to_string())
            //     .await
            //     .unwrap();
            // println!("save_message took: {:.2?}", now.elapsed());
            // let _msg = chronist_
            //     .read()
            //     .await
            //     .get_message(&tips[0].to_string())
            //     .await
            //     .unwrap();
            // println!("get_message took: {:.2?}", now.elapsed());
            let message_ids = reqwest::get("http://localhost:3030/messages/list")
                .await
                .unwrap()
                .json::<Vec<String>>()
                .await
                .unwrap();
            println!("message_ids len: {}", message_ids.len());
            // println!("message_ids took: {:.2?}", now.elapsed());
        }
    }

    // let tips = chronist_
    //     .read()
    //     .await
    //     .iota_client
    //     .read()
    //     .await
    //     .get_tips()
    //     .await
    //     .unwrap();

    // let now = std::time::Instant::now();
    // chronist_
    //     .read()
    //     .await
    //     .save_message(&tips[0].to_string())
    //     .await
    //     .unwrap();
    // println!("save_message took: {:.2?}", now.elapsed());
    // let _msg = chronist_
    //     .read()
    //     .await
    //     .get_message(&tips[0].to_string())
    //     .await
    //     .unwrap();
    // println!("get_message took: {:.2?}", now.elapsed());

    // let message_ids = chronist_.read().await.get_message_ids().await.unwrap();
    // assert_eq!(message_ids.len(), 10000);

    iota.subscriber().disconnect().await.unwrap();

    Ok(())
}
