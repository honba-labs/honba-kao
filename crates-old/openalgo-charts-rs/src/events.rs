//! Event types, closures, and RAII subscription handles.

use wasm_bindgen::prelude::*;

use crate::types::Bar;

/// Event payload emitted when the user moves the crosshair across the chart canvas.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CrosshairMoveEvent {
    pub time: Option<i64>,
    pub price: Option<f64>,
    pub point_x: Option<f64>,
    pub point_y: Option<f64>,
    pub bar: Option<Bar>,
}

impl CrosshairMoveEvent {
    pub fn from_js(val: &JsValue) -> Self {
        if !val.is_object() {
            return Self::default();
        }

        let time = js_sys::Reflect::get(val, &JsValue::from_str("time"))
            .ok()
            .and_then(|v| v.as_f64())
            .map(|t| t as i64);

        let point_x = js_sys::Reflect::get(val, &JsValue::from_str("point"))
            .ok()
            .and_then(|pt| js_sys::Reflect::get(&pt, &JsValue::from_str("x")).ok())
            .and_then(|v| v.as_f64());

        let point_y = js_sys::Reflect::get(val, &JsValue::from_str("point"))
            .ok()
            .and_then(|pt| js_sys::Reflect::get(&pt, &JsValue::from_str("y")).ok())
            .and_then(|v| v.as_f64());

        let price = js_sys::Reflect::get(val, &JsValue::from_str("price"))
            .ok()
            .and_then(|v| v.as_f64());

        let bar = js_sys::Reflect::get(val, &JsValue::from_str("seriesPrices"))
            .ok()
            .or_else(|| js_sys::Reflect::get(val, &JsValue::from_str("bar")).ok())
            .and_then(|b| serde_wasm_bindgen::from_value::<Bar>(b).ok());

        Self {
            time,
            price,
            point_x,
            point_y,
            bar,
        }
    }
}

/// Order click event from Buy/Sell buttons or interactive overlays.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderClickEvent {
    pub side: String,
    pub symbol: String,
    pub quantity: i64,
}

#[allow(dead_code)]
struct ClosureHolder(Box<dyn std::any::Any>);
unsafe impl Send for ClosureHolder {}
unsafe impl Sync for ClosureHolder {}

/// RAII handle for an event subscription. When dropped, automatically unsubscribes.
pub struct Subscription {
    unsubscribe_fn: Option<js_sys::Function>,
    _closure: Option<ClosureHolder>,
}

unsafe impl Send for Subscription {}
unsafe impl Sync for Subscription {}

impl Subscription {
    pub fn new(unsubscribe_fn: js_sys::Function, closure: Box<dyn std::any::Any>) -> Self {
        Self {
            unsubscribe_fn: Some(unsubscribe_fn),
            _closure: Some(ClosureHolder(closure)),
        }
    }

    /// Manually trigger unsubscription.
    pub fn unsubscribe(mut self) {
        if let Some(unsub) = self.unsubscribe_fn.take() {
            let _ = unsub.call0(&JsValue::NULL);
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(unsub) = self.unsubscribe_fn.take() {
            let _ = unsub.call0(&JsValue::NULL);
        }
    }
}
