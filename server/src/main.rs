use std::sync::Arc;

// for tokio-console
use console_subscriber::init;

use server::TradingServer;




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    init();

    let server = TradingServer::new().await;

    TradingServer::run(Arc::new(server)).await;

    Ok(())

}
