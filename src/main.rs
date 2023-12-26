use clap::Parser;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Client, Collection,
};
use ord::{
    index::{Index, LocationUpdateEvent},
    options::Options,
};
use serde::{Deserialize, Serialize};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

static SHUTDOWN_SIGNAL: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Serialize, Deserialize)]
struct Transfer {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    op: Option<ObjectId>,
    tick: String,
    txid: String,
    acc: String,
    amt: f64,
    to: Option<String>,
    output: i32,
    value: Option<f64>,
    transfered: bool,
}

#[tokio::main]
async fn main() {
    println!("Starting custom ord indexer...");

    // MongoDB connection
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap();
    let database = client.database("cbrc_index_3000");
    let transfers_collection: Collection<Transfer> = database.collection("transfers");

    let count = transfers_collection
        .count_documents(doc! {}, None)
        .await
        .unwrap();
    println!("Collection 'transfers' contains: {}", count);

    let index_options = Options::parse();
    let mut index = Index::open(&index_options).unwrap();
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<LocationUpdateEvent>(128);
    index.with_event_sender(sender);

    // Handle Ctrl-C
    ctrlc::set_handler(move || {
        println!("Received Ctrl-C, shutting down...");
        SHUTDOWN_SIGNAL.store(true, Ordering::SeqCst);
    })
    .unwrap();

    let receiver_handle = tokio::spawn(async move {
        while !SHUTDOWN_SIGNAL.load(Ordering::SeqCst) {
            if let Some(event) = receiver.recv().await {
                match event {
                    LocationUpdateEvent::InscriptionCreated { .. } => {
                        println!("Inscription created: {:?}", event);
                    }
                    LocationUpdateEvent::InscriptionMoved { inscription_id, .. } => {
                        let txid = inscription_id.txid.to_string();
                        let index = inscription_id.index as i32;

                        println!("Debug: txid = {}", txid);
                        println!("Debug: index = {}", index);
                        let query = doc! { "txid": &txid, "output": index, "transfered": false };
                        println!("Debug: MongoDB = {:?}", query);

                        let result = transfers_collection.find_one(Some(query), None).await;

                        match result {
                            Ok(Some(transfer)) => {
                                println!("Move detected for existing transfer: {:?}", transfer);

                                // TODO: add event info + transfer id to xmails
                                // TODO: set transfered = true
                            }
                            Ok(None) => {
                                println!("Inscription moved: {:?}", event);
                            }
                            Err(e) => {
                                eprintln!("Err: {:?}", e);
                            }
                        }
                    } // ... autres cas ...
                }
            }
        }
    });

    let index_handle = thread::spawn(move || loop {
        if SHUTDOWN_SIGNAL.load(Ordering::SeqCst) {
            break;
        }
        index.update().unwrap();
        thread::sleep(std::time::Duration::from_secs(3));
    });

    receiver_handle.await.unwrap();
    index_handle.join().unwrap();

    println!("Finished custom ord indexer.");
}
