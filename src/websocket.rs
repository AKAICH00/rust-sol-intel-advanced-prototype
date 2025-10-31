use anyhow::Result;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use serde_json::json;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::metrics::TICKS_RECEIVED;

use crate::types::TickData;

/// Connects to the given WebSocket URL & market, streaming TickData over the channel.
#[tracing::instrument(name = "websocket_stream", skip(tx, url, market))]
pub async fn stream_jupiter_websocket(
    url: String,
    market: String,
    tx: UnboundedSender<TickData>,
) -> Result<()> {
    let (ws_stream, _) = connect_async(&url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Subscribe to memecoin ticker
    let subscribe = json!({
        "op": "subscribe",
        "channel": "ticker",
        "market": market,
    });
    write.send(Message::Text(subscribe.to_string())).await?;

    // Read loop
    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            if let Ok(tick) = serde_json::from_str::<TickData>(&text) {
                TICKS_RECEIVED.inc();
                let _ = tx.send(tick);
            }
        }
    }
    Ok(())
}
