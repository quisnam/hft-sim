use std::sync::Arc;
use console_subscriber::{
    init,
    ConsoleLayer,
};

use server::TradingServer;




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    init();
    //let console_layer = ConsoleLayer::builder()
    //  .server_addr(([0, 0, 0, 0], 8080))
    //  .init();

    let server = TradingServer::new().await;

    TradingServer::run(Arc::new(server)).await;

    Ok(())

}
