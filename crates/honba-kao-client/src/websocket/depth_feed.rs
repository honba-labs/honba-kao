use std::cell::RefCell;
use std::rc::Rc;

use super::connection::{WsConnection, WsConfig, WsEvent};
use crate::dto::market::Level2Book;

pub struct DepthFeed {
    pub conn: Rc<RefCell<Option<WsConnection>>>,
}

impl DepthFeed {
    pub fn subscribe<F>(ws_url: &str, symbol: &str, mut on_book: F) -> Result<Self, String>
    where
        F: FnMut(Level2Book) + 'static,
    {
        let sub_msg = serde_json::json!({
            "action": "subscribe",
            "channel": "depth",
            "symbol": symbol,
        })
        .to_string();
        let sub_msg_clone = sub_msg.clone();

        // The subscription must be re-sent on every (re)connect, because an
        // auto-reconnect opens a brand-new socket that is not subscribed (#66).
        // `connect` consumes the `on_event` closure before `conn` exists, so we
        // hold the connection in a shared cell that the event handler can reach.
        let holder = Rc::new(RefCell::new(None::<WsConnection>));
        let holder_for_event = holder.clone();

        let conn = WsConnection::connect(
            ws_url,
            move |raw| {
                if let Ok(book) = serde_json::from_str::<Level2Book>(&raw) {
                    on_book(book);
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

        // Initial subscription (queued and flushed if the socket is still opening).
        let _ = holder.borrow().as_ref().unwrap().send(&sub_msg);

        Ok(Self { conn: holder })
    }
}
