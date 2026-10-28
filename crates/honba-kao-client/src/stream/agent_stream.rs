use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{EventSource, MessageEvent};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgentStep {
    pub agent_name: String,
    pub step_type: String,
    pub content: String,
    pub timestamp: String,
}

pub fn connect_agent_stream<F>(url: &str, mut on_step: F) -> Result<EventSource, String>
where
    F: FnMut(AgentStep) + 'static,
{
    let es = EventSource::new(url).map_err(|e| format!("Failed to create EventSource: {e:?}"))?;
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        if let Some(txt) = e.data().as_string() {
            if let Ok(step) = serde_json::from_str::<AgentStep>(&txt) {
                on_step(step);
            }
        }
    });

    es.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    Ok(es)
}
