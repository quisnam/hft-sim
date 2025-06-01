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


//use orderbook::order_creator::{
//    OrderRequest,
//    create_order,
//};
//use orderbook::OrderType;
//use orderbook::Side;
//
//
//
//use rand::Rng;
//use tokio::sync::mpsc;
//
//
//
//fn create_order_request() -> OrderRequest {
//    let mut rng = rand::thread_rng();
//
//    let order_type = match rng.gen_range(0..4) {
//        0 => OrderType::GoodTillCancel,
//        1 => OrderType::FillAndKill,
//        2 => OrderType::FillOrKill,
//        3 => OrderType::Market,
//        _ => unreachable!(),
//    };
//
//    let side = match rng.gen_range(0..2) {
//        0 => Side::Buy,
//        1 => Side::Sell,
//        _ => unreachable!(),
//    };
//
//    let quantity = rng.gen_range(2..20);
//
//    if order_type == OrderType::Market {
//        OrderRequest::market_order_request(side, quantity).unwrap()
//    } else {
//        let price = rng.gen_range(15..25);
//        OrderRequest::new(side, price, quantity, order_type).unwrap()
//    }
//}
//
//#[tokio::main]
//async fn main() {
//    let(tx, _rx) = mpsc::channel(10);
//    let mut ob = orderbook::OrderBook::new(tx);
//    for _ in 0..10 {
//        let request =  create_order_request();
//        eprintln!("{}", request);
//
//        ob.add_order(create_order(request)).await;
//        //ob.display_ob().await;
//    }
//
//    let buy = OrderRequest::new(Side::Buy, 10, 5, OrderType::GoodTillCancel).expect("ok");
//    let sell = OrderRequest::new(Side::Sell, 8, 5, OrderType::GoodTillCancel).expect("ok");
//
//    ob.add_order(create_order(buy)).await;
//    ob.add_order(create_order(sell)).await;
//
//    eprintln!("//////////");
//    ob.display_ob().await;
//}
