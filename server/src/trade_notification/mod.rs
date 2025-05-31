use super::{
    Trades,
    TradeNotification,
};

impl  TradeNotification {
    pub fn shutdown() -> Self {
        Self { d_order_id: 0, d_counter_party: None, d_price: 0, d_filled_quantity: 0, d_fully_filled: false }
    }

    pub fn from_trade(trade: &Trades, buyer: bool) -> Self {
        match buyer {
            true => TradeNotification {
                d_order_id: trade.buyer(),
                d_counter_party: Some(trade.seller()),
                d_price: trade.price(),
                d_filled_quantity: trade.quantity(),
                d_fully_filled: trade.buyer_filled() 
            },

            false => TradeNotification {
                d_order_id: trade.seller(),
                d_counter_party: Some(trade.buyer()),
                d_price: trade.price(),
                d_filled_quantity: trade.quantity(),
                d_fully_filled: trade.seller_filled() 
            },
        }
    }
}
