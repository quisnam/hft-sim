use server::{compute_crc, TradeNotification};

use tokio::{
    net::TcpStream,
    io::{
        AsyncWriteExt,
        AsyncReadExt,
    },
    time::{
        sleep,
        Duration,
    }
};
use orderbook::{
    Side,
    OrderType,
    OrderRequest,
};

use rand::Rng;

// generate a random OrderRequest
fn create_order_request() -> OrderRequest {
    let mut rng = rand::thread_rng();

    let order_type = match rng.gen_range(0..4) {
        0 => OrderType::GoodTillCancel,
        1 => OrderType::FillAndKill,
        2 => OrderType::FillOrKill,
        3 => OrderType::Market,
        _ => unreachable!(),
    };

    let side = match rng.gen_range(0..2) {
        0 => Side::Buy,
        1 => Side::Sell,
        _ => unreachable!(),
    };

    let quantity = rng.gen_range(2..20);

    if order_type == OrderType::Market {
        OrderRequest::market_order_request(side, quantity).unwrap()
    } else {
        let price = rng.gen_range(15..25);
        OrderRequest::new(side, price, quantity, order_type).unwrap()
    }
}

// serialize the request into the requeired format
fn  serialize_request(request: &OrderRequest) -> Vec<u8> {
    let (side, price, quantity, order_type) = request.request();
    let mut buffer = [0u8; 16];
    
    let order_type = match order_type {
        OrderType::GoodTillCancel =>  0x01,
        OrderType::FillAndKill => 0x02,
        OrderType::FillOrKill => 0x04,
        OrderType::Market => 0x08,
    };

    let side = match side {
        Side::Buy => 0x01,
        Side::Sell => 0x00,
    };

    buffer[0] = order_type;
    buffer[1] = side;
    buffer[2..6].copy_from_slice(&quantity.to_le_bytes());
    
    buffer[6..10].copy_from_slice(&price.to_le_bytes());

    let crc = compute_crc(&buffer[..10]);
    buffer[10..14].copy_from_slice(&crc.to_le_bytes());
    buffer.to_vec()
}

// serialize the complete byte stream
fn serialize_stream(data: &Vec<OrderRequest>) -> Vec<u8> {
    let message_len: u32 = (16 + 16 * data.len()).try_into().unwrap();

    let order_amount: u32 = data.len().try_into().unwrap();

    let mut byte_stream: Vec<u8> = vec![
        0, 0, 0, 0, 0xFF, 0xFF, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0,
    ];

    byte_stream[0..4].copy_from_slice(&message_len.to_le_bytes());
    byte_stream[6..10].copy_from_slice(&order_amount.to_le_bytes());

    let crc = compute_crc(&byte_stream[..10]);
    byte_stream[10..14].copy_from_slice(&crc.to_le_bytes());

    for order in data {
        byte_stream.append(&mut serialize_request(order));
    }

    byte_stream
}

// Not implemented yet
fn deserialize_trade_information(bytes: Vec<u8>) -> Vec<TradeNotification> {
    Vec::new()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Connect to server
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let (mut reader, mut writer) = tokio::io::split(stream);
    
    // 2. Prepare and send orders
    let mut requests: Vec<OrderRequest> = Vec::new();
    for i in 0..1000 {
        requests.push(create_order_request());
        //eprintln!("{:?}", requests[i])
    }
    let bytes = serialize_stream(&requests);

    //eprintln!("printing: {}", bytes.len());
    
    writer.write_all(&bytes).await?;
    writer.flush().await?;
    println!("Sent {} orders ({} bytes)", requests.len(), bytes.len());
    
    // 3. Process responses
    let mut buf = [0u8; 1024];
    loop {
        match reader.read(&mut buf).await {
            Ok(0) => {
                println!("Server closed connection");
                break;
            }
            Ok(_) => {
                eprintln!("Response received");
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}
