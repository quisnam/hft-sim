use crate::{
    TradeInfo,
    PriceLevelInfo,
};

use std::{
    sync::atomic::{
        AtomicU64,
        Ordering as AtomicOrdering,
    },
    collections::HashMap,
};


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
