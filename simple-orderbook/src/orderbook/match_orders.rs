use super::*;
use std::cmp::Ordering;

/// Matches a given order against a price level
/// """
/// price_level_info: see PriceLevelInfo
/// trade_info: see TradeInfo
/// order: see Order
/// price: current price of price_level that is matched against
/// price_level: Deque of all Buy/Sell orders at the given price level
///
/// returns: nothing
/// => Modifies trade_info in place
/// """
pub async fn match_order_and_price_level(
        price_level_info: &mut PriceLevelInfo,
        trade_info: &mut TradeInfo,
        order: &mut Order,
        price: &u32,
        price_level: &mut VecDeque<Arc<RwLock<Order>>>,
        trades_queue: &mut Sender<Trades>
    )
{
    
    // In case the current price_level is empty
    if price_level_info.get_count(*price) == 0 {
        return ;
    }

    let mut pops = 0;

    for bid in &mut *price_level {
        // The quantity of the order is exhausted
        if order.remaining_quantity() == 0 {
            break;
        }

        // lock reading access
        let bid_guard = bid.read().await;
        if !bid_guard.valid() {
            continue;
        }
         
        let (bid_price, rem_quan) = bid_guard.price_and_remaining_quantity();

        let current_order = bid_guard.trade_info();
        let matching_order = order.trade_info();
        
        // drop reading lock in order to write lock next
        drop(bid_guard);
            
        match rem_quan.cmp(&order.remaining_quantity()) {
            // The order can fill an old order, but is not fulfilled yet
            Ordering::Less => {

                // update trade_info
                trade_info.d_total += order.remaining_quantity() * bid_price;
                trade_info.d_prices.push(bid_price);
                trade_info.d_quantity += rem_quan;
                tokio::spawn(fill_trade(trades_queue.clone(), current_order, matching_order));

                // acquire  write lock and update price_level_info and
                // both orders
                let mut bid_lk = bid.write().await;
                price_level_info.decrement(bid_price);
                bid_lk.fill_all();
                order.fill(&rem_quan);

            }
            // Both orders cancel each other out
            Ordering::Equal => {
                trade_info.d_total += order.remaining_quantity() * bid_price;
                trade_info.d_prices.push(bid_price);
                trade_info.d_quantity += rem_quan;


                tokio::spawn(fill_trade(trades_queue.clone(), current_order, matching_order));

                let mut bid_lk = bid.write().await;
                price_level_info.decrement(bid_price);
                bid_lk.fill_all();
                order.fill_all();
            }

            // The order in the orderbook persists, the contentder (order)
            // is fulfilled
            Ordering::Greater => {
                trade_info.d_total += rem_quan * bid_price;
                trade_info.d_prices.push(bid_price);
                trade_info.d_quantity += order.remaining_quantity();

                tokio::spawn(fill_trade(trades_queue.clone(), current_order, matching_order));

                let mut bid_lk = bid.write().await;
                bid_lk.fill(&order.remaining_quantity());
                order.fill_all();
            }
        }

        if !bid.read().await.valid() {
            pops += 1;
        }
    }

    // This is kinda sus
    // theoretically the orders that are filled first should be in first in the queue so popping
    // in the front should work. But I may introduce some locking in the future
    for _ in 0..pops {
        price_level.pop_front();
    }

}


pub async fn fill_trade(
    trades_queue: Sender<Trades>,
    current_order: (u64, u32, u32, Side),
    matching_order: (u64, u32, u32, Side),
) {
    // eprintln!("Match");
    let (curr_id, curr_qty, curr_price, curr_side) = current_order;
    let (match_id, match_qty, _, _) = matching_order;

    let (buyer_id, seller_id) = match curr_side {
        Side::Buy => (curr_id, match_id),
        Side::Sell => (match_id, curr_id),
    };

    let trade_quantity = curr_qty.min(match_qty);
    let trade_price = curr_price;

    let buyer_filled = match curr_side {
        Side::Buy => curr_qty <= match_qty,
        Side::Sell => match_qty <= curr_qty,
    };

    let seller_filled = match curr_side {
        Side::Sell => curr_qty <= match_qty,
        Side::Buy => match_qty <= curr_qty,
    };

    let trade = Trades::new(
        seller_id,
        buyer_id,
        trade_quantity,
        trade_price,
        seller_filled,
        buyer_filled,
    );

    let _ = trades_queue.send(trade).await;

}
