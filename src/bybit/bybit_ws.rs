use serde_json::json;
use tokio_tungstenite::connect_async;
use tungstenite::protocol::Message;
use futures_util::stream::StreamExt;
use futures::SinkExt;
use url::Url;
use std::sync::Arc;
use tokio::time::{Duration, interval};
use std::error::Error;
use serde::Deserialize;
use serde::Serialize;
use chrono::Utc;



use crate::bybit::bybit_helper::handle_ticker_update;
use crate::state::bybit_state::BybitState;


// Define your SubscribeMessage struct here if needed
#[derive(Serialize, Deserialize, Debug)]
struct SubscribeMessage {
    op: String,
    args: Vec<String>,
}


pub async fn connect_bybit_websocket(
    url: &Url,
    shared_state: Arc<BybitState>
) -> Result<(), Box<dyn Error>> {
    let (ws_stream, _) = connect_async(url).await?;
    println!("Connected to {}", url);

    let (mut write, mut read) = ws_stream.split();
    let mut ping_interval = interval(Duration::from_secs(20));

    let bybit_instruments = shared_state.get_all_contracts();

    for (name, _info) in bybit_instruments.iter() {
        let subscribe_message = json!({
            "op": "subscribe",
            "args": [format!("tickers.{}", name)]
        });

        write.send(Message::Text(subscribe_message.to_string())).await.expect("Failed to send message");
    }

    let ping_task = tokio::spawn(async move {
        loop {
            ping_interval.tick().await; // Wait for the next interval tick

            let ping_message = json!({
                "req_id": "100001", // Example req_id, you can customize this
                "op": "ping"
            }).to_string();
    
            if write.send(Message::Text(ping_message)).await.is_err() {
                eprintln!("Failed to send ping message");
                return;
            }
        }
    });
    
    let read_task = tokio::spawn(async move {
        while let Some(message) = read.next().await {

            match message {
                Ok(text_message) => {
                    if let Message::Text(message) = text_message {
                        
                        let shared_state_clone = Arc::clone(&shared_state);
                        let handle = tokio::task::spawn(async move {
                            handle_ticker_update(&message, shared_state_clone).await;
                        });
                        
                        if let Err(e) = handle.await {
                            let now = Utc::now();
                            eprintln!("{} - Task panicked::: {:?}", now.to_rfc3339(), e);
                            break;

                            
                        }
                    }
                }
                Err(e) => {
                    let now = Utc::now();
                    eprintln!("{} - Read task: error receiving message:: {:?}", now.to_rfc3339(), e);
                }
            }
        }
    });

    tokio::select! {
        _ = read_task => eprintln!("Read task finished unexpectedly."),
        _ = ping_task => eprintln!("Ping task finished unexpectedly."),
    }

    Ok(())
}

