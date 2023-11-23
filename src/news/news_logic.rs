use reqwest::Client;
use std::time::Duration;
use std::sync::Arc;
use tokio::task::spawn;
use futures::future::join_all;
use tokio::time::sleep;

use crate::state::bybit_state::BybitState;
use crate::bybit::bybit_orders::{ execute_bybit_order, execute_sell_bybit };


const SELL_DELAYS: [f32; 5] = [0.8, 1.0, 1.4, 1.8, 2.4];




pub async fn bybit_calc(bybit_clone: &Client, value: String, shared_state: &Arc<BybitState>) {
    if let Some((adjusted_price, num_contracts)) = shared_state.get_specific_contract_details(&format!("{}USDT", value)).await {

        let mut buy_tasks = Vec::new();
        for _ in 0..2 { // Number of buy orders
            let client_clone = bybit_clone.clone();
            let value_clone = value.clone();

            let buy_task = spawn(async move {
                execute_bybit_order(&client_clone, &value_clone, adjusted_price, num_contracts)
                    .await
                    .unwrap_or_else(|err| {
                        eprintln!("Failed to execute buy order: {}", err);
                        0.0
                    })
            });

            buy_tasks.push(buy_task);
        }

        let buy_results = join_all(buy_tasks).await;

        let total_bought_amount: f64 = buy_results.into_iter()
            .map(|result| result.unwrap_or(0.0))
            .sum();

        if total_bought_amount == 0.0 {
            return;
        }

        println!("Successfully bought total size bybit: {}", total_bought_amount);

        let sell_amount = (total_bought_amount / 5.0).ceil();
        let mut tasks = Vec::new();
        for i in 0..5 {
            let client_clone = bybit_clone.clone();
            let value_clone = format!("{}USDT", value);
            let sell_delay = SELL_DELAYS[i];

            let task = spawn(async move {
                sleep(Duration::from_secs_f32(sell_delay)).await;
                if let Err(err) = execute_sell_bybit(&client_clone, value_clone.clone(), sell_amount).await {
                    eprintln!("Failed to execute sell order: {}", err);
                }
            });

            tasks.push(task);
        }

        let _ = join_all(tasks).await;
    } else {
        eprintln!("Couldn't find details for symbol bybit: {}", value);
    }
}







pub async fn execute(bybit_clone: Arc<Client>, token: String, bybit_state: Arc<BybitState>) {
 
    let token_bybit = token.clone();

    tokio::task::spawn(async move {
        bybit_calc(&bybit_clone, token_bybit, &bybit_state).await;
    });


}



/*




pub async fn bybit_calc(bybit_clone: &Client, value: String, shared_state: &Arc<BybitState>) {

    if let Some((adjusted_price, num_contracts)) = shared_state.get_specific_contract_details(&format!("{}USDT", value)).await {

            let bought_amount = execute_bybit_order(&bybit_clone, &value, adjusted_price, num_contracts)
            .await
                .unwrap_or_else(|err| {
                    eprintln!("Failed to execute buy order: {}", err);
                    0.0
                });

            if bought_amount == 0.0 {
                return;
            }
            println!("Successfully bought size bybit {}", bought_amount);

            let sell_amount = (bought_amount / 5.0).ceil();
            let mut tasks = Vec::new();
            for i in 0..5 {
                let client_clone = bybit_clone.clone();
                let value_clone = format!("{}USDT", value);
                let sell_delay = SELL_DELAYS[i];

                let task = spawn(async move {
                    sleep(Duration::from_secs_f32(sell_delay)).await;
                    if let Err(err) = execute_sell_bybit(&client_clone, value_clone.clone(), sell_amount).await {
                        eprintln!("Failed to execute sell order: {}", err);
                    }
                });

                tasks.push(task);
            }

            let _ = join_all(tasks).await;
        } else {
            eprintln!("Couldn't find details for symbol bybit: {}", value);
        }
    }

*/