use std::sync::{
    atomic::AtomicU64,
    Arc,
};

use std::collections::HashMap;

use futures::io;
use tokio::sync::{
    mpsc,
    RwLock
};

use orderbook::{
    order_creator::OrderRequest,
    orderbook::*
};

use futures::future::AbortHandle;

use dashmap::DashMap;

pub mod error;
pub mod server_client_com;
pub mod trade_notification;
pub mod logger;
pub mod trading_server;

pub use server_client_com::{
    compute_crc,
    validate_crc,
    serialize_trade_notification,
    deserialize_stream,
    parse_order,
};

const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

pub struct TradingServer {
    d_orderbook: Arc<RwLock<OrderBook>>,

    d_client_registry: Arc<RwLock<HashMap<u64, mpsc::Sender<TradeNotification>>>>,

    d_order_id_to_client_id: Arc<DashMap<u64, u64>>,

    d_trade_processor: AbortHandle,
}

#[derive(Debug)]
pub enum ProtocolError {
    Io(io::Error),
    MessageTooLarge(usize),
    ContentError(String),
    ConnectionClosed,
    Timeout,
}

#[derive(Clone, PartialEq, Eq)]
pub struct TradeNotification {
    pub d_order_id: u64,
    pub d_counter_party: Option<u64>,
    pub d_price: u32,
    pub d_filled_quantity: u32,
    pub d_fully_filled: bool,
}



fn generate_order_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);

    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed).into()
}
