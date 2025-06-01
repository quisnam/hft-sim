use super::*;
use crate::logger::{
    TradeLogger, FileTradeLogger,
};

use orderbook::order_creator::create_order;

use tokio::{
    io::{
        AsyncReadExt, AsyncWriteExt
    },
    net::{
        TcpListener, TcpSocket, TcpStream
    },
    time::{
        sleep,
        Duration,
    }
};

use futures::future::Abortable;

impl TradingServer {

    pub async fn new() -> Self {
        let (trade_tx, trade_rx) = mpsc::channel(100);
        let orderbook = Arc::new(RwLock::new(OrderBook::new(trade_tx)));
        let clients = Arc::new(RwLock::new(HashMap::new()));
        let logger = Arc::new(FileTradeLogger::new("trades.log").await.unwrap());


        let (abort_handle,  abort_registration) = AbortHandle::new_pair();
        let task = Abortable::new(
            Self::process_trades(trade_rx, Arc::clone(&clients), Arc::clone(&orderbook), logger)
            , abort_registration);
        
        tokio::spawn(task);

        Self {
            d_orderbook: orderbook,
            d_client_registry: clients,
            d_trade_processor: abort_handle,
        }

    }


    
    pub async fn run(server: Arc<TradingServer>) {
        let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();


        //let socket = TcpSocket::new_v4().expect("Err");
        //socket.set_reuseaddr(true).expect("Err2");
        //socket.bind("127.0.0.1:6669".parse().unwrap()).expect("Err3");

        //let listener = socket.listen(1024).expect("err4");
        ////let listener = TcpListener::bind("127.0.0.1:6669").await.unwrap();

        // Spawn shutdown timer
        //let shutdown_server = Arc::clone(&server);
        //tokio::spawn(async move {
        //    sleep(Duration::from_secs(300)).await; // 5 minutes = 300 seconds
        //    shutdown_server.shutdown().await;
        //    shutdown_server.d_trade_processor.abort();
        //});


        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let server = Arc::clone(&server);
    
            eprintln!("Client connected");
            tokio::spawn(async move {
                // Create per-client resources
                let (client_tx, client_rx) = mpsc::channel(32);
                let client_id = server.register_client(client_tx).await;
    
                let server_backup = Arc::clone(&server);
                // Handle connection
                if let Err(e) = Self::handle_connection(socket, server, client_rx).await {
                    eprintln!("Client {} error: {}", client_id, e);
                }
    
                // Cleanup
                server_backup.unregister_client(client_id).await;
                eprintln!("Client_disconnected");
            });
        }

    }



    
    async fn handle_connection(
        socket: TcpStream,
        server: Arc<TradingServer>,
        mut client_rx: mpsc::Receiver<TradeNotification>,
    ) -> Result<(), ProtocolError> {
        let (mut reader, mut writer) = tokio::io::split(socket);
        let mut connection_active = true;
    
        while connection_active {
            tokio::select! {
                order_result = Self::read_orders(&mut reader) => {
                    eprintln!("read  orders");
                    match order_result {
                        Ok((valid_orders, invalid_orders)) => {
                            eprintln!("received order");

                            Self::process_orders(Arc::clone(&server), valid_orders).await;
                            let _ = Self::confirm_orders(&mut writer, invalid_orders).await;
                        }
                        Err(e) if e.is_fatal() => {
                            connection_active =  false;
                            Self::send_error(&mut writer, e).await?;
                            eprintln!("is_fatal");
                        },
                        Err(e) => {
                            eprintln!("not fatal");
                            Self::send_error(&mut writer, e).await?;
                        }
                    }
                }
    
                recv_result = client_rx.recv() => {
                    match recv_result {
                        Some(notification) => {
                            Self::send_notification(&mut writer, notification).await?;
                        }
                        None => {
                            connection_active = false; // Server shutdown
                        }
                    }
                },
            }
        }
    
        Ok(())
    }


    async fn confirm_orders(writer: &mut tokio::io::WriteHalf<tokio::net::TcpStream>, invalid_orders: Vec<u32>) -> Result<(), ProtocolError> {
        let message = b"The orders in the following indices were not accepted: ";

        // Convert the indices to a comma-separated string like "1, 4, 7"
        let indices_string = invalid_orders
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        let full_message = format!("{}{}\n", String::from_utf8_lossy(message), indices_string);
        writer.write_all(&[2u8]).await?;
        writer.write_all(full_message.as_bytes()).await?;

        Ok(())
    }

    async fn send_error(writer: &mut tokio::io::WriteHalf<tokio::net::TcpStream>, e: ProtocolError) -> Result<(), ProtocolError>{
        writer.write_all(&[4u8]).await?;
        writer.write_all(e.to_string().as_bytes()).await?;
        Ok(())
    }

    async fn process_orders(server: Arc<TradingServer>, order_request:  Vec<OrderRequest>) {
        let mut orderbook_lk = server.d_orderbook.write().await;
        for request in order_request {
            eprintln!("{:?}", request);
            let order = create_order(request);
            eprintln!("order created");
            let _ = orderbook_lk.add_order(order).await;
        }

        eprintln!("Added orders to orderbook");
    }

    async fn read_orders(reader: &mut tokio::io::ReadHalf<tokio::net::TcpStream>) -> Result<(Vec<OrderRequest>, Vec<u32>), ProtocolError> {
        eprintln!("Reading orders");

        let mut len_buf = [0u8; 16];
        let _ = reader.read_exact(&mut len_buf).await;
        if len_buf[4] != 0xFF && len_buf[5] != 0xFF {
            return Err(ProtocolError::ContentError("Missing seperator".to_string()));
        }
        let mut message_len: u32 = u32::from_le_bytes(len_buf[0..4].try_into().unwrap());
        let order_amount = u32::from_le_bytes(len_buf[6..10].try_into().unwrap());
        
        if message_len as usize > MAX_MESSAGE_SIZE {
            return Err(ProtocolError::MessageTooLarge(message_len.try_into().unwrap()));
        }
        
        if !validate_crc(&len_buf[0..10], &len_buf[10..14]) {
            return Err(ProtocolError::ContentError("Invalid CRC for header".to_string()));
        }

        message_len -= 16; // already read  16 bytes


        let mut buffer = vec![0u8; message_len as usize];

        eprintln!("creating buffer");
        eprintln!("Message length is: {}", message_len);
        reader.read_exact(&mut buffer).await?;
        eprintln!("read buffer");
        deserialize_stream(&buffer, order_amount)

    }

    async fn send_notification(
        writer: &mut tokio::io::WriteHalf<tokio::net::TcpStream>,
        notification: TradeNotification,
    ) -> Result<(), ProtocolError> {

        let mut buffer = [0; 32];
        let bytes = serialize_trade_notification(&notification, &mut buffer);

        writer.write_all(&(bytes.len() as u32).to_be_bytes()).await?;
        writer.write_all(bytes).await?;
        writer.flush().await?;

        Ok(())
    }

    async fn process_trades(
        mut trade_rx: mpsc::Receiver<Trades>,
        clients: Arc<RwLock<HashMap<u64, mpsc::Sender<TradeNotification>>>>,
        orderbook: Arc<RwLock<OrderBook>>,
        logger: Arc<dyn TradeLogger>,
    ) {
        //{
        //    orderbook.write().await.lazy_deletion().await;
        //}
        while let Some(trade) = trade_rx.recv().await {
            // 1. Notify clients
            let clients_read = clients.read().await;
            if let Some(buyer_tx) = clients_read.get(&trade.buyer()) {
                let _ = buyer_tx.send(TradeNotification::from_trade(&trade, true)).await;
            }
            if let Some(seller_tx) = clients_read.get(&trade.seller()) {
                let _ = seller_tx.send(TradeNotification::from_trade(&trade, false)).await;
            }
            drop(clients_read); // Explicitly release lock
    
            // 2. Cleanup orders
            {
                if trade.buyer_filled() {
                    clients.write().await.remove(&trade.buyer());
                }

                if trade.seller_filled() {
                    clients.write().await.remove(&trade.seller());
                }
            } // Lock automatically released here
    
            // 3. Log trade
            logger.log(&trade).await;
        }
    }

        
    async fn register_client(&self, sender: mpsc::Sender<TradeNotification>) -> u64 {
        let client_id = generate_order_id();
        self.d_client_registry.write().await.insert(client_id, sender);
        client_id
    }

    async fn unregister_client(&self, client: u64) {
        self.d_client_registry.write().await.remove(&client);
    }

    // async fn map_clients_to_trade(&self, trade: &Trades) {
    //     let clients = self.d_client_registry.read().await;

    //     let buyer_notification = TradeNotification {
    //         d_order_id: trade.buyer(),
    //         d_counter_party: Some(trade.seller()),
    //         d_price: trade.price(),
    //         d_filled_quantity: trade.quantity(),
    //         d_fully_filled: trade.buyer_filled(),
    //     };

    //     let seller_notification = TradeNotification {
    //         d_order_id: trade.seller(),
    //         d_counter_party: Some(trade.seller()),
    //         d_price: trade.price(),
    //         d_filled_quantity: trade.quantity(),
    //         d_fully_filled: trade.seller_filled(),
    //     };

    //     if let Some(sender) = clients.get(&trade.buyer()) {
    //         let _ = sender.send(buyer_notification.clone()).await;
    //     }

    //     if let Some(sender) = clients.get(&trade.seller()) {
    //         let _ = sender.send(seller_notification.clone()).await;
    //     }
    // }

    async fn shutdown(&self) {
        self.d_trade_processor.abort();
        let clients = self.d_client_registry.read().await;
        for sender in clients.values() {
            let _ =  sender.send(TradeNotification::shutdown()).await;
        }
    }
}
