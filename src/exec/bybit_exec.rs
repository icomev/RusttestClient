use url::Url;
use reqwest::Client;
use std::sync::Arc;
use chrono::Utc;


use crate::state::bybit_state::BybitState;

use crate::bybit::bybit_helper::get_currency_pairs;
use crate::bybit::bybit_ws::connect_bybit_websocket;

pub async fn run_bybit(bybit_lesser: &Arc<Client>, bybit_state: Arc<BybitState>) {

    match get_currency_pairs(bybit_lesser.clone()).await {
        Ok(symbol_info_map) => {
            bybit_state.update_bybit(symbol_info_map);

            let bybit_url = Url::parse("wss://stream.bybit.com/v5/public/linear").expect("Failed to parse WebSocket URL");
            let bybit_shared_state = bybit_state.clone();

                loop {
                    match connect_bybit_websocket(&bybit_url, bybit_shared_state.clone()).await {
                        
                        Ok(()) => {
                            let now = Utc::now();
                            eprintln!("{} - WebSocket connection closed cleanly, will attempt to reconnect...", now.to_rfc3339());
                        },
                        Err(e) => {
                            let now = Utc::now();
                            eprintln!("{} - Error with WebSocket connection: {:?}", now.to_rfc3339(), e);
                        }
                    }
            
                    // Reconnection backoff
                    eprintln!("{} - Attempting to reconnect to WebSocket...", Utc::now().to_rfc3339());
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
             
        },
        Err(e) => {
            eprintln!("Failed to get futures contracts: {:?}", e);
        },
    }

    

}