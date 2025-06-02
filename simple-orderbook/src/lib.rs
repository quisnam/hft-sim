pub mod order;
pub mod trades;
pub mod orderbook;

use std::{
    sync::{
        Arc,
        atomic::AtomicU64,
    },
    collections::{
        HashMap,
        BTreeMap,
        VecDeque,
    }
};

use tokio::sync::{
    RwLock,
    mpsc::Sender,
};

#[derive(Debug)]

pub enum SimError {
    InvalidOrder,
    KeyOverflow,
    OrderNotFound,
    CancelationError,
    None,
    NoMatchFound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderType {
    GoodTillCancel,
    FillAndKill,
    FillOrKill,
    Market,
}


#[derive(Clone, Debug)]
pub struct OrderRequest {
    d_side: Side,
    d_price: u32,
    d_quantity: u32,
    d_order_type: OrderType,
}

/// Order struct
/// d_id: unique identifier,
/// d_side: Buy/Sell order,
/// price: max/min price,
/// d_initial_quantity,
/// d_remaining_quantity: quantity that remains to be fulfilled
/// d_valid: is the order valid? Needed for cancelations and 
/// fulfilled orders that are to be removed (lazy)
#[derive(PartialEq, Eq)]
pub struct Order {
    d_id: u64,
    d_side: Side,
    d_price: u32,
    d_initial_quantity: u32,
    d_remaining_quantity: u32,
    d_valid: bool,
    d_order_type: OrderType,
}

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

/// Struct to encapsulate:
/// d_level_info: a HashMap mapping an u32 to an AtomicU64
/// key: a price,
/// value: the amount of valid orders at that price
///
/// uses lock-free updates to the HashMap
pub struct PriceLevelInfo {
    d_level_info: HashMap<u32, AtomicU64>,
}

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


pub struct Trades {
    d_seller: u64,
    d_buyer: u64,
    d_quantity: u32,
    d_price: u32,
    d_seller_filled: bool,
    d_buyer_filled: bool,
    pub d_error_indication: SimError,
}

pub enum ServerNotification {
    Trade(Trades),
    Error(String),
}

impl std::fmt::Display for SimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            SimError::InvalidOrder => "The order is invalid.",
            SimError::KeyOverflow => "Order key overflowed",
            SimError::NoMatchFound => "No matching order was found",
            SimError::OrderNotFound => "The specified order was not found.",
            SimError::CancelationError => "The specified order is already invalid",
            SimError::None => "No specific error",
        };

        write!(f, "{}", message)
    }
}
