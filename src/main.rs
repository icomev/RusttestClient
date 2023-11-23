mod bybit;
mod exec;
mod news;
mod state;

use crate::exec::bybit_exec::run_bybit;
use crate::exec::client_exec::run_client;
use crate::state::bybit_state::BybitState;

use reqwest::Client;
use std::sync::Arc;



#[tokio::main]
async fn main() {
    let bybit_lesser = Arc::new(Client::new());
    let bybit_client = Arc::new(Client::new());

    let bybit_state = Arc::new(BybitState::new());

    // Clone bybit_state for the first task
    let bybit_state_clone1 = Arc::clone(&bybit_state);
    tokio::spawn(async move {
        run_client(&bybit_client, bybit_state_clone1).await;
        // Since run_client returns (), there is no error handling needed here
    });

    // Clone bybit_state again for the second task
    let bybit_state_clone2 = Arc::clone(&bybit_state);
    tokio::spawn(async move {
        run_bybit(&bybit_lesser, bybit_state_clone2).await;
        // Since run_bybit returns (), there is no error handling needed here
    });



    futures::future::pending::<()>().await;
}

/*
    let bybit_state_clone3 = Arc::clone(&bybit_state);
     let test_task = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        // Uncomment if you need to use these lines
        //bybit_state.print_bybit_contract("DOGEUSDT").await;
        //tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

        // Assuming "HIFI" is the value you want to test with
        let test_value = "HIFI".to_string(); // Use the exact key as stored in the state
        // You would need to ensure `client` and `bybit_state` are available in this scope
        bybit_calc(&test, test_value, &bybit_state_clone3).await;
    });

    // If you want your main function to complete only when test_task completes,
    // you can await on it. Otherwise, if you want it to run independently, you can
    // just leave it as is.

    // Uncomment the line below to wait for test_task to complete
    test_task.await.expect("Test task failed");
 */
