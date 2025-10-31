use crate::types::TickData;
use anyhow::Result;
use questdb::ingress::Sender;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Client for ingesting ticks into QuestDB with shared, asynchronous sender.
#[derive(Clone)]
pub struct QuestDBClient {
    sender: Arc<Mutex<Sender>>,
}

impl QuestDBClient {
    /// Create a new client with a QuestDB HTTP ingress connection string.
    pub fn new(conf: &str) -> Self {
        let sender = Sender::from_conf(conf).expect("Invalid QuestDB config");
        QuestDBClient {
            sender: Arc::new(Mutex::new(sender)),
        }
    }

    /// Insert a tick into QuestDB (async compatible).
    pub async fn insert_tick(&self, tick: &TickData) -> Result<()> {
        let mut sender = self.sender.lock().await;
        let mut buffer = sender.new_buffer();
        buffer
            .table("memecoin_ticks")?
            .symbol("symbol", &tick.symbol)?
            .column_f64("price", tick.price)?
            .column_f64("volume", tick.volume)?
            .at_now()?;
        // Flush to QuestDB
        sender.flush(&mut buffer)?;
        Ok(())
    }
}
