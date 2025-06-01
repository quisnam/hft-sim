use tokio::{sync::RwLock, time::sleep};
use tokio::sync::mpsc::Sender;

pub mod misc;
mod match_orders;

use std::sync::atomic::{self, AtomicU64};
use std::time::Duration;
use std::{
    collections::{
        BTreeMap,
        HashMap,
        VecDeque,
    },
    sync::Arc, 
};

pub use crate::{ Side, SimError };
pub use crate::order::{
    Order,
    OrderType,
};

use crate::orderbook::match_orders::match_order_and_price_level;
pub use crate::trades::*;

pub use self::misc::{
    PriceLevelInfo,
    TradeInfo,
};


/// OrderBook struct 
/// d_orders: HashMap that maps the order's id to a pointer
///     to the order
/// d_order_creator: creates orders may be made static
/// d_asks/d_bids: Maps price to orders at price level
/// d_price_level_info: contains info about all price levels 
/// may be made static
pub struct OrderBook {
    d_orders: HashMap<u64, Arc<RwLock<Order>>>,
    d_asks: BTreeMap<u32, VecDeque<Arc<RwLock<Order>>>>,
    d_bids: BTreeMap<u32, VecDeque<Arc<RwLock<Order>>>>,
    d_bids_level_info: PriceLevelInfo,
    d_asks_level_info: PriceLevelInfo,
    d_trades_queue: Sender<Trades>,
}


impl OrderBook {
    pub fn new(trades_queue: Sender<Trades>) -> Self {
        OrderBook { 
            d_orders: HashMap::new(),
            d_asks: BTreeMap::new(),
            d_bids: BTreeMap::new(),
            d_bids_level_info: PriceLevelInfo::new(),
            d_asks_level_info: PriceLevelInfo::new(),
            d_trades_queue: trades_queue,
        }
    }

    pub fn remove(&mut self, order_id: &u64) -> Option<Arc<RwLock<Order>>> {
        self.d_orders.remove(order_id)
    }

    pub fn len(&self) -> usize {
        self.d_orders.len()
    }

    pub async fn lazy_deletion(&mut self) {
        loop {
            sleep(Duration::from_secs(2)).await;

            // Clean asks
            for asks in self.d_asks.values_mut() {
                // eprintln!("asks before cleaning: {}", asks.len());
                let mut pops: u32 = 0;
                for ask in asks.iter_mut() {
                    if !ask.read().await.valid() {
                        pops += 1;
                    }
                }
                for _ in 0..pops {
                    let id = asks.pop_front().unwrap().read().await.id();
                    asks.remove(id as usize);

                }
                // eprintln!("asks after cleaning:  {}", asks.len());
            }

            // Clean bids
            for bids in self.d_bids.values_mut() {
                // eprintln!("bids before cleaning:  {}", bids.len());
                let mut pops: u32 = 0;
                for bid in bids.iter_mut() {
                    if !bid.read().await.valid() {
                        pops += 1;
                    }
                }
                for _ in 0..pops  {
                    let id  = bids.pop_front().unwrap().read().await.id();
                    bids.remove(id as usize);
                }



            }
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

        let mut available_quantity = 0;

        match order.side() {
            Side::Sell => {
                let Some((highest_price, _)) = self.d_bids.last_key_value() else {
                    return false;
                };


                if price > *highest_price {
                    return false;
                }
                for (p, orders) in self.d_bids.range(price..=*highest_price) {
                    if quantity < available_quantity {
                        break;
                    }
                    if self.d_bids_level_info.get_count(*p) > 0 {
                        for order in orders {
                            let order_lk = order.read().await;
                            if order_lk.valid() {
                                available_quantity += order_lk.remaining_quantity();
                            }
                        }
                    }
                }

            }

            Side::Buy => {
                let Some((lowest_price, _)) = self.d_asks.first_key_value() else {
                    return false;
                };


                if *lowest_price > price  {
                    return false;
                }

                for (p, orders) in self.d_asks.range(*lowest_price..=price) {
                    if quantity < available_quantity {
                        break;
                    }
                    if self.d_asks_level_info.get_count(*p) > 0 {
                        for order in orders {
                            let order_lk = order.read().await;
                            if order_lk.valid() {
                                available_quantity += order_lk.remaining_quantity();
                            }
                        }
                    }
                }

            }
        }
        available_quantity > quantity
    }
    
    // buy 
    async fn immediate_buy_order(&mut self, order: &mut Order) -> TradeInfo {
        let mut trade_info = TradeInfo::new();
        let max_price = order.price();

        // in case there are no asks with a lower ask
        match self.d_asks.first_key_value() {
            Some((lowest_ask, _)) => {
                if *lowest_ask < max_price{

                }
            }
            _ => {
                return TradeInfo::new();
            }
        }
        
        for  (price, asks) in self.d_asks.range_mut(..=max_price) {
            if order.remaining_quantity() == 0 {
                break;
            }
            match_order_and_price_level(&mut self.d_asks_level_info, &mut trade_info, order, price, asks, &mut self.d_trades_queue).await;
        }
        trade_info
    }

    // sell
    async fn immediate_sell_order(&mut self, order: &mut Order) -> TradeInfo {
        let mut trade_info = TradeInfo::new();
        let lowest_ask = order.price();

        // in case there are no bids with a higher bid
        match self.d_bids.last_key_value() {
            Some((highest_price, _)) => {
                if lowest_ask <  *highest_price {

                }
            }
            _ => {
                return TradeInfo::new();
            },
        }
        
        for (price, bid) in self.d_bids.range_mut(lowest_ask..) {
            if order.remaining_quantity() == 0 {
                break;
            }            
            match_order_and_price_level(&mut self.d_bids_level_info, &mut trade_info, order, price, bid, &mut self.d_trades_queue).await;
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
    pub async fn add_order(&mut self, mut order: Order) -> u64 {
        // {
        //     static ATOM: AtomicU64 = AtomicU64::new(0);
        //     ATOM.fetch_add(1, atomic::Ordering::SeqCst);
        //     let val = ATOM.load(atomic::Ordering::SeqCst);
        //     //eprintln!("{}", val)
        // }
        // create order request and save parameters for easier access in the future
        // eprintln!("Total orders in orderbook: {}\nIn asks: {}\nIn bids: {}", self.d_orders.len(), self.d_asks.len(), self.d_bids.len());
        
        eprintln!("add_order called");
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
            },
            _ => {
                _trade_info = self.execute_trade_immediately(&mut order).await;
            },
        }

        let rem_quan = order.remaining_quantity();

        eprintln!("Debug");

        if order_type == OrderType::FillOrKill || order_type  == OrderType::FillAndKill {
            eprintln!("I am here");
            if _trade_info.is_empty() {
                // This causes a bug
                // let _ = self.d_trades_queue.send(Trades::error(SimError::NoMatchFound)).await;
                eprintln!("good to know");
            }
            eprintln!("otherwise");
            return order_id;
        }


        eprintln!("Debug");
        let order_arc = Arc::new(RwLock::new(order));
        if self.d_orders.insert(order_id, Arc::clone(&order_arc)).is_some() {
            // let _ = self.d_trades_queue.send(Trades::error(SimError::KeyOverflow)).await;
        };




        let price_levels = match side {
            Side::Buy => &mut self.d_bids,
            Side::Sell => &mut self.d_asks,
        };

        // If the order has been fulfilled it will not be added to the orders
        // waiting to be executed.
        if rem_quan != 0 {
            match side {
                Side::Buy => self.d_bids_level_info.increment(order_price),
                Side::Sell => self.d_asks_level_info.increment(order_price),
            }

            price_levels
                .entry(order_price)
                .or_insert_with(VecDeque::new)
                .push_back(Arc::clone(&order_arc));

        }

        order_id
    }

    pub async fn display_ob(&self) {
        for (_, order) in &self.d_orders {
            let order_read = order.read().await;
            if order_read.valid() {            
                println!("{}", order_read);
            }
        }
    }

    pub async fn cancel_order(&mut self, id: u64) ->  Result<(), SimError> {
        match self.d_orders.get(&id) {
            Some(order) => {
                let order_lk = order.read().await;
                let side =  order_lk.side();
                let price = order_lk.price();
                
                drop(order_lk);
                match side {
                    Side::Sell => {
                        self.d_asks_level_info.decrement(price);
                        
                        if let Some(queue) = self.d_asks.get_mut(&price) {
                            Self::remove_order_from_price_level(queue, &id).await;
                        }
                    },

                    Side::Buy => {
                        self.d_bids_level_info.decrement(price);
                        
                        if let Some(queue) = self.d_bids.get_mut(&price) {
                            Self::remove_order_from_price_level(queue, &id).await;
                        }
                    },
                    
                }
                self.d_orders.remove(&id);
                Ok(())
            },
            None => Err(SimError::CancelationError),
        }
    }

    async fn remove_order_from_price_level(price_level: &mut VecDeque<Arc<RwLock<Order>>>, order_id: &u64) {
        
        price_level.retain(|order| {
            let curr_id = tokio::task::block_in_place(|| {
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async {
                    order.read().await.id()
                })
            });
            curr_id == *order_id
        });

    }
}
