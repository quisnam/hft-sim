use super::{
    Trades,
    TradeNotification,
};

impl  TradeNotification {
    // The general way the server sends responses should be updated
    pub fn shutdown() -> Self {
        Self { d_order_id: 0, d_counter_party: None, d_price: 0, d_filled_quantity: 0, d_fully_filled: false }
    }

    pub fn from_trade(trade: &Trades, buyer: bool) -> Self {
        // eprintln!("creating TradeNotification");
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

impl std::fmt::Debug for TradeNotification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TradeNotification {{\n  order_id: {},\n  counterparty: {},\n  quantity: {}@{},\n  filled: {}\n}}",
            self.d_order_id,
            self.d_counter_party.map_or("None".to_string(), |id| id.to_string()),
            self.d_filled_quantity,
            self.d_price,
            self.d_fully_filled
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_trade() {
        let trade = Trades::new(0, 1, 2, 3, false, true);
        let res = TradeNotification {
            d_order_id: 0,
            d_counter_party: Some(1),
            d_price: 3,
            d_filled_quantity: 2,
            d_fully_filled: false,
        };
        assert_eq!(res, TradeNotification::from_trade(&trade, false));
    }
}
