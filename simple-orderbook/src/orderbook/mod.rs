use tokio::sync::RwLock;

pub mod misc;
mod match_orders;

use std::{
    collections::{
        BTreeMap,
        HashMap,
        VecDeque,
    },
    sync::Arc, 
};

use crate::order;
pub use crate::{ Side, LogicError };
pub use crate::order::{
    Order,
    OrderType,
    order_creator::{
        OrderRequest,
        OrderCreator,
    }
};

pub use self::misc::{
    PriceLevelInfo,
    TradeInfo,
};

use self::match_orders::match_order_and_price_level;

/// OrderBook struct 
/// d_orders: HashMap that maps the order's id to a pointer
///     to the order
/// d_order_creator: creates orders may be made static
/// d_asks/d_bids: Maps price to orders at price level
/// d_price_level_info: contains info about all price levels 
/// may be made static
pub struct OrderBook {
    d_orders: HashMap<u64, Arc<RwLock<Order>>>,
    d_order_creator: OrderCreator,
    d_asks: BTreeMap<u32, VecDeque<Arc<RwLock<Order>>>>,
    d_bids: BTreeMap<u32, VecDeque<Arc<RwLock<Order>>>>,
    d_bids_level_info: PriceLevelInfo,
    d_asks_level_info: PriceLevelInfo,
}


impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook { 
            d_orders: HashMap::new(),
            d_order_creator: OrderCreator::new(),
            d_asks: BTreeMap::new(),
            d_bids: BTreeMap::new(),
            d_bids_level_info: PriceLevelInfo::new(),
            d_asks_level_info: PriceLevelInfo::new(),
        }
    }

    // checks if there are valid orders with a price higher or equal than price
    pub fn is_in_highest_bids(&self, price: u32) -> bool {
       let Some((highest_price, _)) = self.d_bids.last_key_value() else {
           return false;
       };

       // Check if price is within top N bids
       self.d_bids
           .range(price..=*highest_price)
           .any(|(p, _orders)| {
               // *p == price && 
               // !orders.is_empty() && 
               self.d_bids_level_info.get_count(*p) > 0
           })
    }   

    // checks if there are valid orders with a price lower or equal than price (bid)
    pub fn is_in_lowest_asks(&self, price: u32) -> bool {
        let Some((lowest_price, _)) = self.d_asks.first_key_value() else {
            return false
        };

        self.d_asks
            .range(*lowest_price..price)
            .any(|(p, _)| {
                self.d_asks_level_info.get_count(*p) > 0
            })
    }

    // checks if there are matches for the order
    pub fn can_match(&self, order: &Order) -> bool {
        let price = order.price();

        match order.side() {
            Side::Sell => self.is_in_highest_bids(price),
            Side::Buy => self.is_in_lowest_asks(price),
        }
    }

    pub async fn can_match_fully(&self, order: &Order) -> bool {
    let price = order.price();
    let quantity = order.remaining_quantity();

    match order.side() {
        Side::Sell => {
            let Some((highest_price, _)) = self.d_bids.last_key_value() else {
                return false;
            };

            let mut available_quantity = 0;

            for (p, orders) in self.d_bids.range(price..=*highest_price) {
                if self.d_bids_level_info.get_count(*p) > 0 {
                    for order in orders {
                        let order_lk = order.read().await;
                        if order_lk.valid() {
                            available_quantity += order_lk.remaining_quantity() as u64;
                        }
                    }
                }
            }

            available_quantity > quantity as u64
        }

        Side::Buy => {
            let Some((lowest_price, _)) = self.d_asks.first_key_value() else {
                return false;
            };

            let mut available_quantity = 0;

            for (p, orders) in self.d_asks.range(*lowest_price..=price) {
                if self.d_asks_level_info.get_count(*p) > 0 {
                    for order in orders {
                        let order_lk = order.read().await;
                        if order_lk.valid() {
                            available_quantity += order_lk.remaining_quantity() as u64;
                        }
                    }
                }
            }

            available_quantity > quantity as u64
        }
    }
}
    
    // buy 
    async fn immediate_buy_order(&mut self, order: &mut Order) -> TradeInfo {
        let mut trade_info = TradeInfo::new();
        let max_price = order.price();
        for  (price, asks) in self.d_asks.range_mut(..=max_price) {
            if order.remaining_quantity() == 0 {
                break;
            }
            match_order_and_price_level(&mut self.d_asks_level_info, &mut trade_info, order, price, asks).await;
        }
        trade_info
    }

    // sell
    async fn immediate_sell_order(&mut self, order: &mut Order) -> TradeInfo {
        let mut trade_info = TradeInfo::new();
        let lowest_ask = order.price();
        
        for (price, bid) in self.d_bids.range_mut(lowest_ask..) {
            if order.remaining_quantity() == 0 {
                break;
            }            
            match_order_and_price_level(&mut self.d_bids_level_info, &mut trade_info, order, price, bid).await;
        }
        trade_info
    }
    // execute trade with given order
    async fn execute_trade_immediately(&mut self, order: &mut Order) -> TradeInfo {
        match order.side() {
            Side::Buy => self.immediate_buy_order(order).await,
            Side::Sell => self.immediate_sell_order(order).await,
        }
    }

    // Add a new OrderRequest
    pub async fn add_order(&mut self, order_request: OrderRequest) -> Result<(u64, Option<TradeInfo>), LogicError> {
        // create order request and save parameters for easier access in the future
        let mut order = self.d_order_creator.create_order(order_request);
        let order_id = order.id();
        let order_price = order.price();
        let order_type = order.order_type();
        let side = order.side();

        let mut _trade_info = TradeInfo::new();

        // check if an Order can be executed [Maybe the can match will be merged with execute_trade_immediately
        // as it kind of checks twice]
        match order_type {
            OrderType::FillAndKill => {
                if self.can_match_fully(&order).await {
                    _trade_info = self.execute_trade_immediately(&mut order).await;
                }
            }
            _ => if self.can_match(&order) {
                    _trade_info = self.execute_trade_immediately(&mut order).await;
                }
        }
        

        let rem_quan = order.remaining_quantity();

        let order = Arc::new(RwLock::new(order));

        if self.d_orders.insert(order_id, Arc::clone(&order)).is_some() {
            return Err(LogicError::KeyOverflow);
        };

        if order_type == OrderType::FillAndKill || order_type == OrderType::FillOrKill {
            return  Ok((order_id, Some(_trade_info)));
        }

        let price_levels = match order.read().await.side() {
            Side::Buy => &mut self.d_bids,
            Side::Sell => &mut self.d_asks,
        };

        price_levels
            .entry(order.read().await.price())
            .or_insert_with(VecDeque::new)
            .push_back(Arc::clone(&order));
        
        // If the order has been fulfilled it will not be added to the orders
        // waiting to be executed. However, it is still inserted so the trade can be
        // logged
        if rem_quan != 0 {
            match side {
                Side::Buy => self.d_bids_level_info.increment(order_price),
                Side::Sell => self.d_asks_level_info.increment(order_price),
            }
        }
        
        if _trade_info.is_empty() {
            Ok((order_id, None))
        } else {
            Ok((order_id, Some(_trade_info)))
        }
    }

    pub async fn display_ob(&self) {
        for (_, order) in &self.d_orders {
            let order_read = order.read().await;
            if order_read.valid() {            
                println!("{}", order_read);
            }
        }
    }

    pub async fn cancel_order(&mut self, id: u64) ->  Result<(), LogicError> {
        match self.d_orders.get(&id) {
            Some(order) => {
                let mut order_lk = order.write().await;
                match order_lk.side() {
                    Side::Sell => self.d_asks_level_info.decrement(order_lk.price()),
                    Side::Buy => self.d_bids_level_info.decrement(order_lk.price()),
                    
                }
                order_lk.invalidate()?;
                Ok(())
            },
            None => Err(LogicError::OrderNotFound),
        }
    }
}
