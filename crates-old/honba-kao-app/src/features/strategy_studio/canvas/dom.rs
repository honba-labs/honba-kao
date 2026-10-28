//! Thin DOM shims for the canvas (HBD-2026-DES-006 P2).
//!
//! `DataTransfer` and `DOMRect` are reached through `js_sys::Reflect`
//! rather than typed `web-sys` bindings, so the editor needs no extra
//! `web-sys` cargo features beyond the event types Leptos already enables.
//! Every shim degrades to a no-op / `None` instead of panicking.

use wasm_bindgen::{JsCast, JsValue};

fn get(target: &JsValue, key: &str) -> Option<JsValue> {
    js_sys::Reflect::get(target, &JsValue::from_str(key))
        .ok()
        .filter(|value| !value.is_undefined() && !value.is_null())
}

fn call(target: &JsValue, method: &str, args: &[&JsValue]) -> Option<JsValue> {
    let function: js_sys::Function = get(target, method)?.dyn_into().ok()?;
    let result = match args {
        [] => function.call0(target),
        [a] => function.call1(target, a),
        [a, b] => function.call2(target, a, b),
        _ => return None,
    };
    result.ok()
}

fn data_transfer(ev: &web_sys::DragEvent) -> Option<JsValue> {
    get(ev.as_ref(), "dataTransfer")
}

/// `ev.dataTransfer.setData(mime, data)` + `effectAllowed = "copy"`.
pub fn drag_set_data(ev: &web_sys::DragEvent, mime: &str, data: &str) {
    if let Some(transfer) = data_transfer(ev) {
        let _ = call(
            &transfer,
            "setData",
            &[&JsValue::from_str(mime), &JsValue::from_str(data)],
        );
        // Some browsers refuse to start a drag with only a custom MIME type.
        let _ = call(
            &transfer,
            "setData",
            &[&JsValue::from_str("text/plain"), &JsValue::from_str(data)],
        );
        let _ = js_sys::Reflect::set(
            &transfer,
            &JsValue::from_str("effectAllowed"),
            &JsValue::from_str("copy"),
        );
    }
}

/// `ev.dataTransfer.getData(mime)`, `None` when empty.
pub fn drag_get_data(ev: &web_sys::DragEvent, mime: &str) -> Option<String> {
    let transfer = data_transfer(ev)?;
    call(&transfer, "getData", &[&JsValue::from_str(mime)])
        .and_then(|value| value.as_string())
        .filter(|text| !text.is_empty())
}

/// `ev.dataTransfer.dropEffect = "copy"` (the dragover cursor).
pub fn drag_set_drop_effect(ev: &web_sys::DragEvent) {
    if let Some(transfer) = data_transfer(ev) {
        let _ = js_sys::Reflect::set(
            &transfer,
            &JsValue::from_str("dropEffect"),
            &JsValue::from_str("copy"),
        );
    }
}

/// The element's `getBoundingClientRect()` `(left, top, width, height)`.
pub fn client_rect(element: &web_sys::Element) -> Option<(f64, f64, f64, f64)> {
    let rect = call(element.as_ref(), "getBoundingClientRect", &[])?;
    let number = |key: &str| get(&rect, key).and_then(|v| v.as_f64());
    Some((
        number("left")?,
        number("top")?,
        number("width")?,
        number("height")?,
    ))
}

/// Whether a key event comes from an editable control, where Delete /
/// Backspace / Escape belong to the control (fe's INPUT/TEXTAREA guard,
/// widened to SELECT and contenteditable).
pub fn is_editable_target(ev: &web_sys::KeyboardEvent) -> bool {
    let Some(element) = ev
        .target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
    else {
        return false;
    };
    let tag = element.tag_name().to_ascii_uppercase();
    if matches!(tag.as_str(), "INPUT" | "TEXTAREA" | "SELECT") {
        return true;
    }
    element
        .get_attribute("contenteditable")
        .is_some_and(|value| value != "false")
}
