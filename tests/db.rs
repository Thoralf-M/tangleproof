use tangleproof::chronist::Chronist;
#[tokio::test]
async fn db() {
    let chronist = Chronist::new(
        "testdb",
        "https://api.lb-0.h.chrysalis-devnet.iota.cafe/",
        "256a818b2aac458941f7274985a410e57fb750f3a3a67969ece5bd9ae7eef5b2",
    )
    .await
    .unwrap();

    let tips = chronist.iota_client.get_tips().await.unwrap();

    chronist.save_message(&tips[0].to_string()).await.unwrap();
    let msg = chronist.get_message(&tips[0].to_string()).await.unwrap();
    println!("{:?}", msg);
    assert_eq!(msg.message.id().0, tips[0]);
    let message_ids = chronist.get_message_ids().await.unwrap();
    println!("{:?}", message_ids);
}

use iota_client::{bee_message::Message, Client, MqttEvent, Result, Topic};
use std::sync::{mpsc::channel, Arc, Mutex};

#[tokio::test]
async fn spam() -> Result<()> {
    let chronist = Chronist::new(
        "testdb",
        "https://chrysalis-nodes.iota.org",
        "256a818b2aac458941f7274985a410e57fb750f3a3a67969ece5bd9ae7eef5b2",
    )
    .await
    .unwrap();

    let mut iota = Client::builder()
        .with_node("https://chrysalis-nodes.iota.org")?
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
        .with_topics(vec![Topic::new("messages").unwrap()])
        .subscribe(move |event| match event.topic.as_str() {
            "messages" => {
                let message: Message = serde_json::from_str(&event.payload).unwrap();
                tx.lock().unwrap().send(message).unwrap();
            }
            _ => println!("{:?}", event),
        })
        .await
        .unwrap();

    let chronist_ = Arc::new(tokio::sync::RwLock::new(chronist));
    for outer in 0..2000 {
        let mut tasks = Vec::new();
        for _ in 0..50 {
            let message = rx.recv().unwrap();
            let chronist__ = chronist_.clone();
            tasks.push(async move {
                tokio::spawn(async move {
                    let _ = chronist__
                        .read()
                        .await
                        .save_message(&message.id().0.to_string())
                        .await
                        .unwrap();
                })
                .await
            });
        }
        println!("{}", outer);
        let _results = futures::future::try_join_all(tasks)
            .await
            .expect("failed to sync addresses");

        let tips = chronist_.read().await.iota_client.get_tips().await.unwrap();

        let now = std::time::Instant::now();
        chronist_
            .read()
            .await
            .save_message(&tips[0].to_string())
            .await
            .unwrap();
        println!("save_message took: {:.2?}", now.elapsed());
        let _msg = chronist_
            .read()
            .await
            .get_message(&tips[0].to_string())
            .await
            .unwrap();
        println!("get_message took: {:.2?}", now.elapsed());
        let message_ids = chronist_.read().await.get_message_ids().await.unwrap();
        println!("message_ids len: {}", message_ids.len());
    }

    let tips = chronist_.read().await.iota_client.get_tips().await.unwrap();

    let now = std::time::Instant::now();
    chronist_
        .read()
        .await
        .save_message(&tips[0].to_string())
        .await
        .unwrap();
    println!("save_message took: {:.2?}", now.elapsed());
    let _msg = chronist_
        .read()
        .await
        .get_message(&tips[0].to_string())
        .await
        .unwrap();
    println!("get_message took: {:.2?}", now.elapsed());

    let message_ids = chronist_.read().await.get_message_ids().await.unwrap();
    assert_eq!(message_ids.len(), 10000);

    iota.subscriber().disconnect().await.unwrap();
    Ok(())
}
