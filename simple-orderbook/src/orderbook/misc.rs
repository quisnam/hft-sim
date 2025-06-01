use std::{
    sync::atomic::{
        AtomicU64,
        Ordering as AtomicOrdering,
    },
    collections::HashMap,
};

/// Struct to encapsulate:
/// d_prices: a vector of the prices that an order matched
/// d_total: the total price paid/money gained from the given order
/// d_quantity: the the d_quantity bought/sold
#[derive(Debug)]
pub struct TradeInfo {
    pub d_prices: Vec<u32>,
    pub d_total: u32,
    pub d_quantity: u32,
}

impl Default for TradeInfo {
    fn default() -> Self {
        TradeInfo::new()
    }
}

impl TradeInfo {
    pub fn new() -> Self {
        TradeInfo { d_prices: Vec::new(), d_total: 0, d_quantity: 0}
    }

    // checks if the TradeInfo is in its initial state
    pub fn is_empty(&self) -> bool {
        self.d_prices.is_empty() && self.d_total == 0 && self.d_quantity == 0
    }
}

/// Struct to encapsulate:
/// d_level_info: a HashMap mapping an u32 to an AtomicU64
/// key: a price,
/// value: the amount of valid orders at that price
///
/// uses lock-free updates to the HashMap
pub struct PriceLevelInfo {
    d_level_info: HashMap<u32, AtomicU64>,
}

impl Default for PriceLevelInfo {
    fn default() -> Self {
        PriceLevelInfo::new()
    }
}

impl PriceLevelInfo {
    pub fn new() -> Self {
        PriceLevelInfo { d_level_info: HashMap::new() }
    }

    // either create a price level or add one to the number of orders waiting
    pub fn increment(&mut self, price: u32) {
        self.d_level_info
            .entry(price)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, AtomicOrdering::SeqCst);
    }

    // decrement the number of orders waiting
    pub fn decrement(&mut self, price: u32) {
        if let Some(counter) = self.d_level_info.get(&price) {
            counter.fetch_sub(1, AtomicOrdering::SeqCst);
        }
    }

    // get amount of orders waiting at a level
    pub fn get_count(&self, price: u32) -> u64 {
        self.d_level_info
            .get(&price)
            .map(|atom| atom.load(AtomicOrdering::Relaxed))
            .unwrap_or(0)
    }
}
