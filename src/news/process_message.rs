use futures::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::Client;
use std::sync::Arc;

use std::error::Error;

use crate::news::news_logic::execute ;
use crate::state::bybit_state::BybitState;



pub async fn connect_websocket(server_url: &str, bybit_client: &Arc<Client>, bybit_state: &Arc<BybitState>) -> Result<(), Box<dyn Error>> {
    let url = url::Url::parse(server_url)?;

    let (ws_stream, _) = connect_async(&url).await?;

    println!("WebSocket client connected to {}", server_url);
    let (_write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(msg)) => {
                println!("Msg: {}", msg);

                let bybit_clone = bybit_client.clone();
                let bybit_state_clone = bybit_state.clone();
                execute(bybit_clone, msg, bybit_state_clone).await;

                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis();
                println!("Timestamp: {} ms", timestamp);
                
            },
            Ok(_) => eprintln!("Received non-text message"),
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
        }
    }

    println!("WebSocket client disconnected");
    Ok(())
}