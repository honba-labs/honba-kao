use super::connection::{WsConnection, WsConfig};
use crate::dto::market::TradeTick;

pub struct TradeFeed {
    pub conn: WsConnection,
}

impl TradeFeed {
    pub fn subscribe<F>(ws_url: &str, symbol: &str, mut on_trade: F) -> Result<Self, String>
    where
        F: FnMut(TradeTick) + 'static,
    {
        let conn = WsConnection::connect(
            ws_url,
            move |raw| {
                if let Ok(trade) = serde_json::from_str::<TradeTick>(&raw) {
                    on_trade(trade);
                }
            },
            |_| {}, // lifecycle events are surfaced via `conn.is_connected()`
            WsConfig::default(),
        )?;

        let sub_msg = serde_json::json!({
            "action": "subscribe",
            "channel": "trades",
            "symbol": symbol,
        });
        let _ = conn.send(&sub_msg.to_string());

        Ok(Self { conn })
    }
}
