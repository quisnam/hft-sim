use std::sync::atomic::{
    AtomicU64,
    Ordering
};

use crate::{
    Side,
    SimError,
    Order,
    OrderType,
    OrderRequest,
};

impl std::fmt::Display for OrderRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        write!(
            f,
            "OrderType: {:?}\nSide: {:?}\nQuantity: {}\nPrice: {}",
            self.d_order_type, self.d_side, self.d_quantity, self.d_price
        )
}
}

impl OrderRequest {
    pub fn new(side: Side, price: u32, quantity: u32, order_type: OrderType)
        -> Result<OrderRequest, SimError> 
    {
        Self::valid(side, price, quantity, order_type)
    }

    pub fn market_order_request(side: Side, quantity: u32) 
        -> Result<OrderRequest, SimError> 
    {
        Ok(
            OrderRequest {
                d_side: side,
                d_price: 0,
                d_quantity: quantity,
                d_order_type: OrderType::Market,
            }
        )
    }

    fn valid(side: Side, price: u32, quantity: u32, order_type: OrderType) 
        -> Result<OrderRequest, SimError> 
    {
        
       
        Ok(OrderRequest {
            d_side: side,
            d_price: price,
            d_quantity: quantity,
            d_order_type: order_type,
        })
    }

    pub fn request(&self) -> (Side, u32, u32, OrderType) {
        (self.d_side.clone(), self.d_price, self.d_quantity, self.d_order_type.clone())
    }
}

fn order_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub fn create_order(order_request: OrderRequest) -> Order {

    Order::new(
        order_id(),
        order_request.d_side,
        order_request.d_price,
        order_request.d_quantity,
        order_request.d_order_type,
    )
}
