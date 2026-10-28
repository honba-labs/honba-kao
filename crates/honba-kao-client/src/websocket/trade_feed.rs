use std::cell::RefCell;
use std::rc::Rc;

use super::connection::{WsConnection, WsConfig, WsEvent};
use crate::dto::market::TradeTick;

pub struct TradeFeed {
    pub conn: Rc<RefCell<Option<WsConnection>>>,
}

impl TradeFeed {
    pub fn subscribe<F>(ws_url: &str, symbol: &str, mut on_trade: F) -> Result<Self, String>
    where
        F: FnMut(TradeTick) + 'static,
    {
        let sub_msg = serde_json::json!({
            "action": "subscribe",
            "channel": "trades",
            "symbol": symbol,
        })
        .to_string();
        let sub_msg_clone = sub_msg.clone();

        // #66: re-send the subscription on every (re)connect so a new socket
        // after a network drop does not stay idle and unsubscribed.
        let holder = Rc::new(RefCell::new(None::<WsConnection>));
        let holder_for_event = holder.clone();

        let conn = WsConnection::connect(
            ws_url,
            move |raw| {
                if let Ok(trade) = serde_json::from_str::<TradeTick>(&raw) {
                    on_trade(trade);
                }
            },
            move |event| {
                if let WsEvent::Open = event {
                    if let Some(conn) = holder_for_event.borrow().as_ref() {
                        let _ = conn.send(&sub_msg_clone);
                    }
                }
            },
            WsConfig::default(),
        )?;
        *holder.borrow_mut() = Some(conn);

        let _ = holder.borrow().as_ref().unwrap().send(&sub_msg);

        Ok(Self { conn: holder })
    }
}
