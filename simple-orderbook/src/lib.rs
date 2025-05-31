pub mod order;
pub mod trades;
pub mod orderbook;

pub use crate::trades::*;
pub use crate::orderbook::*;

#[derive(Debug)]

pub enum SimError {
    InvalidOrder,
    KeyOverflow,
    OrderNotFound,
    CancelationError,
    None,
    NoMatchFound,
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
