use reqwest::Client;
use std::sync::Arc;

use crate::news::process_message::connect_websocket ;
use crate::state::bybit_state::BybitState;
use crate::bybit::bybit_orders::keep_alive_order;



pub async fn run_client(bybit_client: &Arc<Client>, bybit_state: Arc<BybitState>) {
    //let bybit_client = Arc::new(Client::new());
    //let bybit_state = Arc::new(BybitState::new());


    let server_url = "ws://13.113.245.177:9000/websocket"; // Change this to your server's URL

    let keep_alive_bybit = tokio::spawn(keep_alive_order(Arc::clone(&bybit_client)));

    let bybit_clone1 = bybit_client.clone();
    let server_task = tokio::spawn(async move {
    loop {
        let bybit_clone = Arc::clone(&bybit_clone1);
        let bybit_state_clone = Arc::clone(&bybit_state);

        println!("Attempting to connect to WebSocket server...");
        match connect_websocket(server_url, &bybit_clone, &bybit_state_clone).await {
            Ok(_) => {
                println!("WebSocket connection closed gracefully. Attempting to reconnect...");
            }
            Err(e) => {
                eprintln!("WebSocket client encountered an error: {}. Attempting to reconnect...", e);
            }
        }
        // Wait a bit before attempting to reconnect
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
});


    let _ = keep_alive_bybit.await;
    let _ = tokio::try_join!(server_task);


}
