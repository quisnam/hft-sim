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
    price_level: &mut VecDeque<Arc<RwLock<Order>>>)
{
    
    // The current price level is empty. This can happen because I use lazy deletion
    if price_level_info.get_count(*price) == 0 {
        return ;
    }

    for bid in price_level {
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
        
        // drop reading lock in order to write lock next
        drop(bid_guard);
            
        match rem_quan.cmp(&order.remaining_quantity()) {
            // The order can fill an old order, but is not fulfilled yet
            Ordering::Less => {

                // update trade_info
                trade_info.d_total += order.remaining_quantity() * bid_price;
                trade_info.d_prices.push(bid_price);
                trade_info.d_quantity += rem_quan;

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
                let mut bid_lk = bid.write().await;
                bid_lk.fill(&order.remaining_quantity());
                order.fill_all();
            }
        }
    }
}


