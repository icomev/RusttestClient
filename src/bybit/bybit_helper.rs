use serde::Deserialize;
use std::sync::Arc;
use reqwest::Client;
use serde::Serialize;


use std::collections::HashMap;
use serde_json::Value;
use std::error::Error;

use crate::state::bybit_state::BybitState;

const CATEGORY: &str = "linear";
const URL: &str = "https://api.bybit.com";
const INSTRUMENTS_INFO_PATH: &str = "/v5/market/instruments-info";


#[derive(Serialize, Clone, Debug)]
pub struct SymbolInfo {
    pub max_order_qty: f64,
    pub adjusted_prices: Vec<f64>,
    pub num_contracts: Vec<f64>,
}

#[derive(Debug, Deserialize)]

#[allow(non_snake_case)]
struct InstrumentsResponse {
    retCode: i32,
    retMsg: String,
    result: ResultPayload,
}

#[derive(Debug, Deserialize)]
struct ResultPayload {
    list: Vec<InstrumentInfo>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct InstrumentInfo {
    symbol: String,
    lotSizeFilter: LotSizeFilter,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct LotSizeFilter {
    maxOrderQty: String,
}


pub async fn handle_ticker_update(message: &str, shared_state: Arc<BybitState>) {
    let v: Value = match serde_json::from_str(message) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Failed to parse JSON: {}. The message was: {}", err, message);
            return;
        }
    };

    if let Some(symbol) = v["data"]["symbol"].as_str() {
        if let Some(last_price_str) = v["data"]["lastPrice"].as_str() {
            if let Ok(last_price) = last_price_str.parse::<f64>() {
                shared_state.update_contract_details(symbol, last_price).await;
            } else {
                eprintln!("Failed to parse 'lastPrice' field as a floating-point number from string: {}", last_price_str);
            }
        }
    }
}

    

pub async fn get_currency_pairs(less_important_client: Arc<Client>) -> Result<HashMap<String, SymbolInfo>, Box<dyn Error + Send + Sync>> {
    let params = [("category", CATEGORY)];
    
        let res = less_important_client
            .get(&format!("{}{}", URL, INSTRUMENTS_INFO_PATH))
            .query(&params)
            .send()
            .await?;
        
        let instruments_response: InstrumentsResponse = res.json().await?;
    
        if instruments_response.retCode != 0 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, instruments_response.retMsg)));
        }
    
        let mut symbol_info_map: HashMap<String, SymbolInfo> = HashMap::new();
        
        for instrument in instruments_response.result.list {
            let symbol = &instrument.symbol;
            if symbol.ends_with("USDT") {
                let max_order_qty = instrument.lotSizeFilter.maxOrderQty.parse::<f64>().unwrap_or(0.0);
                symbol_info_map.insert(symbol.clone(), SymbolInfo {
                    adjusted_prices: vec![0.0],
                    max_order_qty: max_order_qty,
                    num_contracts: vec![0.0],
                });
            }
        }
    
        Ok(symbol_info_map)
    }
