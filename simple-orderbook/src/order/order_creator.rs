use std::sync::atomic::{
    AtomicU64,
    Ordering
};
use crate::Side;
use crate::LogicError;
use super::Order;

/// Struct that returns a valid Order
/// uses an atomic lock-free counter to create a unique
/// order id. If there are u64::MAX + 1 ids the behaviour
/// is undefined
pub struct OrderCreator {
    d_order_id_gen: OrderIdGen,
}

impl Default  for OrderCreator {
    fn default() -> Self {
        Self::new()
    }
}
impl OrderCreator {
    pub fn new() -> OrderCreator {
        OrderCreator { 
            d_order_id_gen: OrderIdGen::new(),
        }
    }
    
    pub fn create_order(&self, order_request: OrderRequest) -> Order {
        Order::new(
            self.d_order_id_gen.next_id(),
            order_request.d_side,
            order_request.d_price,
            order_request.d_quantity
        )
    } 
}


/// Generates the ids
struct OrderIdGen {
    d_counter: AtomicU64,
}

impl OrderIdGen {
    pub fn new() -> Self {
        OrderIdGen {
            d_counter: AtomicU64::new(0),
        }
    }

    pub fn next_id(&self) -> u64 {
        self.d_counter.fetch_add(1, Ordering::SeqCst)
    }
}

/// The client creates an instance of this struct in order to create
/// an order.
/// The client has to specify a Side, a price and a quantity
pub struct OrderRequest {
    d_side: Side,
    d_price: u32,
    d_quantity: u32,
}

impl OrderRequest {
    pub fn new(side: Side, price: u32, quantity: u32) -> Result<OrderRequest, LogicError> {
        Self::valid(side, price, quantity)
    }

    fn valid(side: Side, price: u32, quantity: u32) -> Result<OrderRequest, LogicError> {
        
       
        Ok(OrderRequest {
            d_side: side,
            d_price: price,
            d_quantity: quantity
        })
    }

    pub fn request(&self) -> (Side, u32, u32) {
        (self.d_side.clone(), self.d_price, self.d_quantity)
    }
}
