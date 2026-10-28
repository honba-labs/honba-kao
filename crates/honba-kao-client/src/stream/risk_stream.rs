use crate::dto::risk::LiquidationWarning;
use wasm_bindgen::prelude::*;
use web_sys::{EventSource, MessageEvent};

pub fn connect_risk_stream<F>(url: &str, mut on_warning: F) -> Result<EventSource, String>
where
    F: FnMut(LiquidationWarning) + 'static,
{
    let es = EventSource::new(url).map_err(|e| format!("Failed to create EventSource: {e:?}"))?;
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        if let Some(txt) = e.data().as_string() {
            if let Ok(warning) = serde_json::from_str::<LiquidationWarning>(&txt) {
                on_warning(warning);
            }
        }
    });

    es.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    Ok(es)
}
