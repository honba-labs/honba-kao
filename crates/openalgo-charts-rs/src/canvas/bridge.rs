//! Low-level bindings to the vendored OpenAlgo Charts canvas engine.
//!
//! The bridge is loaded by the host page (`vendor/openalgo-charts/honba-bridge.js`)
//! and exposes `window.honbaCharts`. Every engine call below marshals JSON strings
//! across the wasm/JS boundary so the Rust side never holds JS objects directly.
//!
//! These functions degrade gracefully: when `window.honbaCharts` is absent (e.g.
//! native `cargo test` runs outside a browser) they return `Err` so the crate stays
//! testable on the host target. On non-`wasm32` targets the `JsValue` constructors
//! are not available at all, so every public function is duplicated behind
//! `#[cfg(target_arch = "wasm32")]` / `#[cfg(not(target_arch = "wasm32"))]`.

use wasm_bindgen::JsValue;

/// `Function` / `Reflect` are only exercised by the wasm32 `call` helper.
#[cfg(target_arch = "wasm32")]
use js_sys::{Function, Reflect};

/// Fetch `window.honbaCharts`, if present.
#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
fn honba_charts() -> Option<JsValue> {
    let window = web_sys::window()?;
    Reflect::get(&window, &JsValue::from_str("honbaCharts")).ok()
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
fn honba_charts() -> Option<JsValue> {
    None
}

/// Invoke a method on `window.honbaCharts` with the given JSON-serialisable args.
#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
fn call(method: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
    let hc = honba_charts().ok_or_else(|| JsValue::from_str("honbaCharts not mounted"))?;
    let f = Reflect::get(&hc, &JsValue::from_str(method))?;
    let func = Function::from(f);
    let arr = js_sys::Array::from_iter(args.iter().cloned());
    func.apply(&JsValue::NULL, &arr)
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
fn call(_method: &str, _args: &[JsValue]) -> Result<JsValue, JsValue> {
    Err(JsValue::NULL)
}

// ----------------------------------------------------------------------------
// Public API — wasm32 builds talk to `window.honbaCharts`; host builds degrade.

/// `mountStatic(element)` — plain engine chart (no widget chrome) for
/// host-supplied series. Returns the chart id.
#[cfg(target_arch = "wasm32")]
pub fn mount_static(element: &web_sys::HtmlElement) -> Result<f64, JsValue> {
    let result = call("mountStatic", &[element.into()])?;
    result
        .as_f64()
        .ok_or_else(|| JsValue::from_str("mountStatic did not return a chart id"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn mount_static(_element: &web_sys::HtmlElement) -> Result<f64, JsValue> {
    Err(JsValue::NULL)
}

/// `setSeries(id, kind, style_json, bars_json)` — replace a series' data.
#[cfg(target_arch = "wasm32")]
pub fn set_series(
    id: f64,
    kind: &str,
    style_json: &str,
    bars_json: &str,
) -> Result<bool, JsValue> {
    let args = [
        JsValue::from_f64(id),
        JsValue::from_str(kind),
        JsValue::from_str(style_json),
        JsValue::from_str(bars_json),
    ];
    let result = call("setSeries", &args)?;
    result
        .as_bool()
        .ok_or_else(|| JsValue::from_str("setSeries did not return a boolean"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_series(
    _id: f64,
    _kind: &str,
    _style_json: &str,
    _bars_json: &str,
) -> Result<bool, JsValue> {
    Err(JsValue::NULL)
}

/// `updateLast(id, bar_json)` — update/append the last bar (live streaming).
#[cfg(target_arch = "wasm32")]
pub fn update_last(id: f64, bar_json: &str) -> Result<bool, JsValue> {
    let args = [JsValue::from_f64(id), JsValue::from_str(bar_json)];
    let result = call("updateLast", &args)?;
    result
        .as_bool()
        .ok_or_else(|| JsValue::from_str("updateLast did not return a boolean"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn update_last(_id: f64, _bar_json: &str) -> Result<bool, JsValue> {
    Err(JsValue::NULL)
}

/// `addVolumeOverlay(id)` — attach a volume histogram to the primary pane.
#[cfg(target_arch = "wasm32")]
pub fn add_volume_overlay(id: f64) -> Result<bool, JsValue> {
    let args = [JsValue::from_f64(id)];
    let result = call("addVolumeOverlay", &args)?;
    result
        .as_bool()
        .ok_or_else(|| JsValue::from_str("addVolumeOverlay did not return a boolean"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn add_volume_overlay(_id: f64) -> Result<bool, JsValue> {
    Err(JsValue::NULL)
}

/// `destroy(id)` — tear down a chart and release its canvases.
#[cfg(target_arch = "wasm32")]
pub fn destroy(id: f64) -> Result<bool, JsValue> {
    let args = [JsValue::from_f64(id)];
    let result = call("destroy", &args)?;
    result
        .as_bool()
        .ok_or_else(|| JsValue::from_str("destroy did not return a boolean"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn destroy(_id: f64) -> Result<bool, JsValue> {
    Err(JsValue::NULL)
}

/// `refreshThemeAll()` — restyle every mounted chart from the active skin.
#[cfg(target_arch = "wasm32")]
pub fn refresh_theme_all() -> Result<(), JsValue> {
    call("refreshThemeAll", &[])?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn refresh_theme_all() -> Result<(), JsValue> {
    Err(JsValue::NULL)
}
