pub use crate::order::*;
use crate::SimError;

pub struct Trades {
    d_seller: u64,
    d_buyer: u64,
    d_quantity: u32,
    d_price: u32,
    d_seller_filled: bool,
    d_buyer_filled: bool,
    pub d_error_indication: SimError,
}

impl Trades {
    pub fn new(seller_id: u64, buyer_id: u64, quantity: u32, price: u32, seller_filled: bool, buyer_filled: bool) -> Self {
        Self {
            d_seller: seller_id,
            d_buyer: buyer_id,
            d_quantity: quantity,
            d_price: price,
            d_seller_filled: seller_filled,
            d_buyer_filled: buyer_filled,
            d_error_indication: SimError::None,
        }
    }

    pub fn error(error_code: SimError) -> Self {
        Self {
            d_seller: 0,
            d_buyer: 0,
            d_quantity: 0,
            d_price: 0,
            d_seller_filled: true,
            d_buyer_filled: true,
            d_error_indication: error_code,
        }

    }

    pub fn seller(&self) -> u64 {
        self.d_seller
    }

    pub fn buyer(&self) -> u64 {
        self.d_buyer
    }

    pub fn quantity(&self) -> u32 {
        self.d_quantity
    }

    pub fn price(&self) -> u32 {
        self.d_price
    }

    pub fn seller_filled(&self) -> bool {
        self.d_seller_filled
    }

    pub fn buyer_filled(&self) -> bool {
        self.d_buyer_filled
    }
}
