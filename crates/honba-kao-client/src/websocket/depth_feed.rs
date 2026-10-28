use super::connection::{WsConnection, WsConfig};
use crate::dto::market::Level2Book;

pub struct DepthFeed {
    pub conn: WsConnection,
}

impl DepthFeed {
    pub fn subscribe<F>(ws_url: &str, symbol: &str, mut on_book: F) -> Result<Self, String>
    where
        F: FnMut(Level2Book) + 'static,
    {
        let conn = WsConnection::connect(
            ws_url,
            move |raw| {
                if let Ok(book) = serde_json::from_str::<Level2Book>(&raw) {
                    on_book(book);
                }
            },
            |_| {}, // lifecycle events are surfaced via `conn.is_connected()`
            WsConfig::default(),
        )?;

        let sub_msg = serde_json::json!({
            "action": "subscribe",
            "channel": "depth",
            "symbol": symbol,
        });
        let _ = conn.send(&sub_msg.to_string());

        Ok(Self { conn })
    }
}
