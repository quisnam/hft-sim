use simple_orderbook::{
    order::*,
    orderbook::*,
};

#[tokio::main]
async fn main() {
    let mut orderbook = OrderBook::new();

    let order_request = OrderRequest::new(Side::Buy, 10, 5).expect("Invalid Price");

    let res = orderbook.add_order(order_request).await;

    let (id, _trade_info) = match res {
        Ok((id, Some(trade_info))) => (id, trade_info),
        Ok((id, None)) => (id, TradeInfo::new()),
        Err(e) => panic!("Err {:?} occured", e)
        
    };
    let order_request1 = OrderRequest::new(Side::Sell, 7, 3).expect("Invalid Price");
    let res = orderbook.add_order(order_request1).await;
    orderbook.display_ob().await;

    let (id, _trade_info) = match res {
        Ok((id, Some(trade_info))) => (id, trade_info),
        Ok((id, None)) => (id, TradeInfo::new()),
        Err(e) => panic!("Err {:?} occured", e)
        
    };

    let _ = orderbook.cancel_order(id).await;

    orderbook.display_ob().await;

    if !_trade_info.is_empty() {
        let trade = format!(r#"
        Total amount spent: {}
        Total amount bought: {}
    "#, _trade_info.d_total, _trade_info.d_quantity);
        println!("{trade}");
    } else {
        println!("No trades");
    }
}
