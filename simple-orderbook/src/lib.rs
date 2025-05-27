pub mod order;
pub mod orderbook;

pub use crate::order::*;
pub use crate::orderbook::*;

#[derive(Debug)]
pub enum LogicError {
    InvalidOrder,
    KeyOverflow,
    OrderNotFound,
    CancelationError,
}
