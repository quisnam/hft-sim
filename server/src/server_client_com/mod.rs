use super::{
    ProtocolError,
    TradeNotification,
    Side,
    OrderType,
    OrderRequest,
};

pub fn compute_crc(data_stream: &[u8]) -> u32 {
    const POLY: u32 = 0x82F63B78;
    let mut crc: u32 = 0xFFFFFFFF;

    for &byte in data_stream {
        crc ^= byte as u32;

        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ POLY;
            } else {
                crc >>= 1;
            }
        }
    }

    crc ^ 0xFFFFFFFF
}

pub fn validate_crc(data_stream: &[u8], crc: &[u8]) -> bool {
    u32::from_le_bytes(crc[0..4].try_into().unwrap()) == compute_crc(data_stream)
}

pub fn serialize_trade_notification<'a>(
    trade_notification: &TradeNotification,
    buffer: &'a mut [u8; 32],
) -> &'a [u8; 32] {
    buffer[0] = 1;
    buffer[1] = 32;

    buffer[2..10].copy_from_slice(&trade_notification.d_order_id.to_le_bytes());

    let counter_party  = trade_notification.d_counter_party
        .map(|id| id.to_le_bytes())
        .unwrap_or([0xFF;8]);
    buffer[10..18].copy_from_slice(&counter_party);

    buffer[18..22].copy_from_slice(&trade_notification.d_price.to_le_bytes());
    buffer[22..26].copy_from_slice(&trade_notification.d_filled_quantity.to_le_bytes());
    buffer[27] = u8::from(trade_notification.d_fully_filled);
    
    let crc = compute_crc(&buffer[..27]);
    buffer[27..31].copy_from_slice(&crc.to_le_bytes());
    buffer
}

pub fn parse_order(data_stream: &[u8; 16]) -> Option<OrderRequest> {
    // OrderType:
    // 1 -> GoodTillCancel
    // 2 -> FillAndKill
    // 4 -> FillOrKill
    // 8 -> MarketOrder
    //
    if !validate_crc(&data_stream[..10], &data_stream[10..14]) {
        return None;
    }

    let order_type = match data_stream[0] {
        0x01 => OrderType::GoodTillCancel,
        0x02 => OrderType::FillAndKill,
        0x04 => OrderType::FillOrKill,
        0x08 => OrderType::Market,
        _ => return None,
    };

    let side = if data_stream[1] != 0 {
        Side::Buy
    } else {
        Side::Sell
    };

    let quantity = u32::from_le_bytes(data_stream[2..6].try_into().unwrap());

    if order_type == OrderType::Market {
        return Some(OrderRequest::market_order_request(side, quantity).expect("ok"));
    }

    let price = u32::from_le_bytes(data_stream[6..10].try_into().unwrap());

    Some(OrderRequest::new(side, price, quantity, order_type).expect("Ok"))
}
pub fn deserialize_stream(data_stream: &[u8], order_amount: u32) -> Result<(Vec<OrderRequest>, Vec<u32>), ProtocolError> {
    // data_stream:
    // message_len: 4 bytes
    // 2 byte buffer: 0xFFFF
    // order_amount: 4 bytes
    // crc: 4 bytes
    // 2 byte buffer
    // OrderRequest: 16 bytes ->
    // d_order_type: u8 -> 1, 2, 4, 8 -> 1 byte
    // Side: Buy/Sell: true, false -> 1 byte
    // d_filled_quantity: u32 -> 4 bytes
    // d_price: u32 -> 4 bytes
    // crc: 4 bytes
    // 2 byte buffer
    
    
    let mut invalid_orders = Vec::new();
    let mut order_requests =  Vec::new();

    let offset = 16;
    for index in 0..order_amount {
        let end = offset + 16;

        if end > data_stream.len() {
            invalid_orders.push(index);
            break
        }

        let order_slice: &[u8; 16] = match data_stream[offset..end].try_into() {
            Ok(s) => s,
            Err(_) => {
                invalid_orders.push(index);
                break;
            }
        };

        match parse_order(order_slice) {
            Some(order) => order_requests.push(order),
            None =>  invalid_orders.push(index),
        }
    }

    Ok((order_requests, invalid_orders))
}
