use super::{
    ProtocolError,
    TradeNotification,
    Side,
    OrderType,
    OrderRequest,
};

/// Compute the CRC with the reflected polynom from the
/// CRC32C
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

/// Validate a crc
pub fn validate_crc(data_stream: &[u8], crc: &[u8]) -> bool {
    u32::from_le_bytes(crc[0..4].try_into().unwrap()) == compute_crc(data_stream)
}

/// Serialize a Trade Notification.
/// zeroeth byte:  Type => 1 == normal Notification
/// first byte: Length => always 32 bytes
/// then 16 bytes for the order ids of buyer and seller
/// then 4 bytes price and 4 bytes quantity
/// then 1 byte if it  is fully filled
/// finally 4 bytes CRC
pub fn serialize_trade_notification<'a>(
    trade_notification: &TradeNotification,
    buffer: &'a mut [u8; 32],
) -> &'a [u8; 32] {
    eprintln!("Serializing");
    buffer[0] = 1;
    buffer[1] = 32;

    buffer[2..10].copy_from_slice(&trade_notification.d_order_id.to_le_bytes());

    let counter_party  = trade_notification.d_counter_party
        .map(|id| id.to_le_bytes())
        .unwrap_or([0xFF;8]);
    buffer[10..18].copy_from_slice(&counter_party);

    buffer[18..22].copy_from_slice(&trade_notification.d_price.to_le_bytes());
    buffer[22..26].copy_from_slice(&trade_notification.d_filled_quantity.to_le_bytes());

    buffer[26] = u8::from(trade_notification.d_fully_filled);
    
    let crc = compute_crc(&buffer[..27]);

    buffer[27..31].copy_from_slice(&crc.to_le_bytes());
    eprintln!("finished serializing");
    buffer
}

// Parse Orders out from a data stream 
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

    // Market Orders don't have a price
    if order_type == OrderType::Market {
        return Some(OrderRequest::market_order_request(side, quantity).expect("ok"));
    }

    let price = u32::from_le_bytes(data_stream[6..10].try_into().unwrap());

    Some(OrderRequest::new(side, price, quantity, order_type).expect("Ok"))
}

// Deserialize the whole stream
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

    let mut offset = 16;
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
        offset += 16;
    }
    
    Ok((order_requests, invalid_orders))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_crc1() {
        let result = 0xE3069283;
        let input  = b"123456789";
        assert_eq!(result, compute_crc(input));
    }

    #[test]
    fn test_compute_crc2() {
        let result = 0x00000000;
        let input = b"";
        assert_eq!(result, compute_crc(input));
    }

    #[test]
    fn test_compute_crc3() {
        let result = 0x22620404;
        let input = b"The quick brown fox jumps over the lazy dog";
        assert_eq!(result, compute_crc(input));
    }
    
    
    #[test]
    fn test_validate_crc1() {
        let data = b"123456789";
        let crc: [u8; 4] = 0xE3069283u32.to_le_bytes(); // [0x83, 0x92, 0x06, 0xE3]
        assert!(validate_crc(data, &crc));
    }

    #[test]
    fn test_validate_crc2() {
        let data = b"1212121212";
        let crc: [u8; 4] = 0x12121212u32.to_le_bytes();

        assert!(!validate_crc(data, &crc));
    }

    #[test]
    fn test_serialize_trade_notification() {
        let mut buffer = [0u8; 32];
        let trade = TradeNotification {
            d_order_id: 1,
            d_counter_party: Some(2),
            d_price: 3,
            d_filled_quantity: 4,
            d_fully_filled: true,
        };
        let res: [u8; 27] = [
            1,                      // Type
            32,                     // Length
            1, 0, 0, 0, 0, 0, 0, 0, // 1u64.to_le_bytes()
            2, 0, 0, 0, 0, 0, 0, 0, // 2u64.to_le_bytes()
            3, 0, 0, 0,             // 3u32.to_le_bytes()
            4, 0, 0, 0,             // 4u32.to_le_bytes()
            1,                      // true -> 0x1
        ];
        assert_eq!(res, serialize_trade_notification(&trade, &mut buffer)[..27]);
    }
}
