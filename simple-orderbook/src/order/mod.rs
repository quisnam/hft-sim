use std::fmt;

use crate::{
    SimError,
    Side,
    OrderType,
    Order,
};

pub mod order_creator;

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}




impl Order {
    pub fn new(id: u64, side: Side, price: u32, quantity: u32, order_type: OrderType) -> Self {
        Order { 
            d_id: id,
            d_side: side,
            d_price: price,
            d_initial_quantity: quantity,
            d_remaining_quantity: quantity,
            d_valid: true,
            d_order_type: order_type,
        }
    }

    // maybe not needed anymore
    pub fn new_invalid() -> Self {
        Order {
            d_id: 0,
            d_side: Side::Sell,
            d_price: 0,
            d_initial_quantity: 0,
            d_remaining_quantity: 0,
            d_valid: false,
            d_order_type: OrderType::FillAndKill,
        }
    }

    pub fn id(&self) -> u64 {
        self.d_id
    }

    pub fn side(&self) -> Side {
        self.d_side.clone()
    }
    
    pub fn price(&self) -> u32 {
        self.d_price
    }

    // returns two data members to avoid locking to often
    pub fn price_and_remaining_quantity(&self) -> (u32, u32) {
        (self.d_price, self.d_remaining_quantity)
    }

    pub fn initial_quantity(&self) -> u32 {
        self.d_initial_quantity
    }

    pub fn remaining_quantity(&self) -> u32 {
        self.d_remaining_quantity
    }

    pub fn valid(&self) -> bool {
        self.d_valid
    }

    pub fn order_type(&self) -> OrderType {
        self.d_order_type.clone()
    }

    pub fn invalidate(&mut self) -> Result<(), SimError> {
        if !self.d_valid {
            Err(SimError::CancelationError)
        } else {
            self.d_valid = false;
            Ok(())
        }
    }

    // fills the order completely
    pub fn fill_all(&mut self) {
        self.d_remaining_quantity = 0;
        self.d_valid = false;
    }

    // fills the order partially
    pub fn fill(&mut self, quantity: &u32) {
        self.d_remaining_quantity -= quantity;
    }

    pub fn trade_info(&self) -> (u64, u32, u32, Side) {
        (
            self.d_id,
            self.d_remaining_quantity,
            self.d_price,
            self.d_side.clone(),
        )
    }

}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"
ID:         {},
SIDE:       {},
PRICE:      {},
QUANTITY:   {}
            "#,
            self.d_id,
            self.d_side,
            self.d_price,
            self.d_remaining_quantity
        )
    }
}
