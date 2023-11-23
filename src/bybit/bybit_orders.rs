use std::time::SystemTime;
use hmac::{Hmac, NewMac, Mac};
use sha2::Sha256;
use hex;
use reqwest::header;
use reqwest::Client;
use serde_json;
use std::error::Error;
use tokio::time::sleep;
use std::time::Duration;
use std::sync::Arc;
use once_cell::sync::Lazy;


const URL: &str = "https://api.bybit.com";
const CATEGORY: &str = "linear";
const SIDE: &str = "Buy";
const SELL_SIDE: &str = "Sell";
const ORDER_TYPE: &str = "Limit";
const SELL_ORDER_TYPE: &str = "Market";
const API_KEY: &str = "APIKEY";
const API_SECRET: &str = "SQKEY";
const PATH: &str = "/v5/order/create";
const TIME_IN_FORCE: &str = "IOC";



static BASE_HEADERS: Lazy<header::HeaderMap> = Lazy::new(|| {
    let mut headers = header::HeaderMap::new();
    headers.insert("X-BAPI-API-KEY", header::HeaderValue::from_static(API_KEY));
    headers
});

static HMAC_KEY: Lazy<Hmac<Sha256>> = Lazy::new(|| {
    Hmac::<Sha256>::new_varkey(API_SECRET.as_bytes()).expect("HMAC can take key of any size")
});

pub async fn keep_alive_order(client: Arc<Client>) {
    loop {
        let timestamp_millis = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            .to_string();

        let params = serde_json::json!({
            "category": CATEGORY,
            "side": SIDE,
            "orderType": ORDER_TYPE,
            "symbol": "DOGEUSDT",
            "qty": "200.0",
            "price": "0.0682",
            "timeInForce": "IOC",
        });

        let message = format!("{}{}{}", &timestamp_millis, API_KEY, &params.to_string());

        let mut mac = HMAC_KEY.clone();
        mac.update(message.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        let sign = hex::encode(code_bytes);

        let mut headers = BASE_HEADERS.clone();
        headers.insert("X-BAPI-TIMESTAMP", header::HeaderValue::from_str(&timestamp_millis).unwrap());
        headers.insert("X-BAPI-SIGN", header::HeaderValue::from_str(&sign).unwrap());

        match client
            .post(&format!("{}{}", URL, PATH))
            .headers(headers)
            .json(&params)
            .send()
            .await {
            Ok(res) => {
                if !res.status().is_success() {
                    match res.text().await {
                        Ok(response_text) => eprintln!("Received non-successful status. Response text: {}", response_text),
                        Err(err) => eprintln!("Failed to read response text: {}", err),
                    }
                }
            },
            Err(err) => eprintln!("Failed to send request: {}", err),
        }

        sleep(Duration::from_secs(8)).await;
    }
}

pub async fn execute_bybit_order(client: &Client, currency_pair: &str, adjusted_price: f64, amount: f64) -> Result<f64, Box<dyn Error>> {
    let timestamp_millis = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
        .to_string();

        let params = format!(
            r#"{{"category": "{}", "side": "{}", "orderType": "{}", "timeInForce": "{}", "symbol": "{}USDT", "qty": "{}", "price": "{}"}}"#,
            CATEGORY, SIDE, ORDER_TYPE, TIME_IN_FORCE, currency_pair, amount, adjusted_price
        );

    let message = format!("{}{}{}", &timestamp_millis, API_KEY, &params);

    let mut mac = HMAC_KEY.clone();
    mac.update(message.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    let sign = hex::encode(code_bytes);

    let mut headers = BASE_HEADERS.clone();
    headers.insert("X-BAPI-TIMESTAMP", header::HeaderValue::from_str(&timestamp_millis).unwrap());
    headers.insert("X-BAPI-SIGN", header::HeaderValue::from_str(&sign).unwrap());

    let res = client
        .post(&format!("{}{}", URL, PATH))
        .headers(headers)
        .body(params)
        .send()
        .await?;

    let response_text = res.text().await?;
    println!("Response text: {}", response_text);
    Ok(amount)
}



pub async fn execute_sell_bybit(client: &Client, currency_pair: String, amount: f64) -> Result<(), Box<dyn Error>> {

    let timestamp_millis = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .expect("Time went backwards")
    .as_millis()
    .to_string();

    let params = serde_json::json!({
        "category": CATEGORY,
        "side": SELL_SIDE,
        "orderType": SELL_ORDER_TYPE,
        "symbol": currency_pair,
        "qty": amount.to_string(),
        "reduceOnly": true,

    });
    let message = format!(
        "{}{}{}",
        &timestamp_millis, API_KEY, &params.to_string()
    );

    let mut mac = Hmac::<Sha256>::new_varkey(API_SECRET.as_bytes()).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    let sign = hex::encode(code_bytes);

    let mut headers = header::HeaderMap::new();
    headers.insert("X-BAPI-API-KEY", header::HeaderValue::from_str(API_KEY).unwrap());
    headers.insert("X-BAPI-TIMESTAMP", header::HeaderValue::from_str(&timestamp_millis).unwrap());
    headers.insert("X-BAPI-SIGN", header::HeaderValue::from_str(&sign).unwrap());

    let res = client
        .post(&format!("{}{}", URL, PATH))
        .headers(headers)
        .json(&params)  
        .send()
        .await?;

    let response_text = res.text().await?;
    println!("Sell response: {}", response_text); 

    Ok(())
}
