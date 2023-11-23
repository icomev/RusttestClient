use std::collections::HashMap;
use parking_lot::Mutex;


use crate::bybit::bybit_helper::SymbolInfo;


//const INVESTMENT_CASH: f64 = 14000.0;
const INVESTMENT_CASH: f64 = 14000.0;


pub struct BybitState {
    contracts: Mutex<HashMap<String, SymbolInfo>>,
}

impl BybitState {
    pub fn new() -> Self {
        BybitState {
            contracts: Mutex::new(HashMap::new()),
        }
    }

    pub fn update_bybit(&self, new_contracts: HashMap<String, SymbolInfo>) {
        let mut contracts = self.contracts.lock();
        *contracts = new_contracts;
    }

    pub fn get_all_contracts(&self) -> HashMap<String, SymbolInfo> {
        let contracts = self.contracts.lock();
        contracts.clone() // Clone the HashMap to use outside of the lock
    }

    pub async fn print_all_contracts(&self) {
        let contracts = self.contracts.lock();
        for (symbol, details) in contracts.iter() {
            println!("Symbol: {}, Details: {:?}", symbol, details);
        }
    }
    pub async fn print_bybit_contract(&self, symbol: &str) {
        println!("symbol {}", symbol);

        let contracts = self.contracts.lock();
        if let Some(contract) = contracts.get(symbol) {
            println!("{:?}", contract);
        } else {
            println!("Contract for symbol {} not found", symbol);
        }
    }

    pub async fn get_specific_contract_details(&self, token: &str) -> Option<(f64, f64)> {
        let contracts = self.contracts.lock();
        contracts.get(token).and_then(|symbol_info| {
            let adjusted_price = *symbol_info.adjusted_prices.last()?;
            let num_contracts = *symbol_info.num_contracts.last()?;
            Some((adjusted_price, num_contracts))
        })
    }

    pub async fn update_contract_details(&self, symbol: &str, last_price: f64) {
        let decimal_places = format!("{:e}", last_price).split('.').nth(1).map_or(0, |fraction| fraction.len());
        let multiplier = 10f64.powi(decimal_places as i32);
        let adjusted_price = (last_price * 1.05 * multiplier).round() / multiplier;
        let num_contracts = (INVESTMENT_CASH / adjusted_price).round();

        let mut contracts = self.contracts.lock();

        if let Some(symbol_info) = contracts.get_mut(symbol) {
            if symbol_info.adjusted_prices.len() >= 4 { symbol_info.adjusted_prices.remove(0); }
            if symbol_info.num_contracts.len() >= 4 { symbol_info.num_contracts.remove(0); }

            symbol_info.adjusted_prices.push(adjusted_price);
            let calculated_amount = num_contracts.min(symbol_info.max_order_qty).round();
            symbol_info.num_contracts.push(calculated_amount);
        }
    }

}