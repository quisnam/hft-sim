use std::fmt;

use crate::LogicError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderType {
    GoodTillCancel,
    FillAndKill,
    FillOrKill,
    Market,
}

pub mod order_creator;

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

    pub fn invalidate(&mut self) -> Result<(), LogicError> {
        if !self.d_valid {
            Err(LogicError::CancelationError)
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
