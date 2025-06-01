use tokio::{
    fs::File,
    io::AsyncWriteExt,
};
use async_trait::async_trait;
use super::Trades;


/// Logging all executed trades.

#[async_trait]
pub trait TradeLogger: Send + Sync {
    async fn log(&self, trade: &Trades);
}

pub struct FileTradeLogger {
    file: tokio::sync::Mutex<File>,
}

impl FileTradeLogger {
    pub async fn new(path: &str) -> std::io::Result<Self> {
        let file = File::create(path).await?;
        Ok(Self {
            file: tokio::sync::Mutex::new(file),
        })
    }
}

#[async_trait]
impl TradeLogger for FileTradeLogger {
    async fn log(&self, trade: &Trades) {
        let log_line = format!(
            "Trade: {}@{} between {} and {}\n",
            trade.quantity(), trade.price(), trade.buyer(), trade.seller()
        );

        let mut file = self.file.lock().await;
        if let Err(e) = file.write_all(log_line.as_bytes()).await {
            eprintln!("Failed to log trade: {}", e);
        }
    }
}
