use crate::dto::order::OrderResponse;
use wasm_bindgen::prelude::*;
use web_sys::{EventSource, MessageEvent};

pub fn connect_orders_stream<F>(url: &str, mut on_order: F) -> Result<EventSource, String>
where
    F: FnMut(OrderResponse) + 'static,
{
    let es = EventSource::new(url).map_err(|e| format!("Failed to create EventSource: {e:?}"))?;
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        if let Some(txt) = e.data().as_string() {
            if let Ok(order) = serde_json::from_str::<OrderResponse>(&txt) {
                on_order(order);
            }
        }
    });

    es.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    Ok(es)
}
