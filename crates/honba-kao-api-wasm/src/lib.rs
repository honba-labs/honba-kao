#![allow(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(unsafe_code)]

//! WASM compute surface for Honba frontend.
//!
//! Pure compute only: indicators (close-only and OHLC), screener evaluation, Journal v1
//! backtest replay, fill → round-trip pairing and run analytics. No tokio, no filesystem, no
//! network. Depends only on pure L0-L4 crates.
//!
//! The logic lives in plain Rust modules ([`indicators`], [`screener`], [`replay`],
//! [`pairing`], [`analytics`], [`calendar`]) that are tested natively and expose **typed** Rust
//! APIs: same-workspace Rust callers (the Leptos app) call those directly with plain structs, no
//! JSON round trip. With the default `js` feature, this file adds the `wasm-bindgen` layer for
//! external JS callers on top: series cross that boundary as `Float64Array`; a screen result and a
//! journal replay cross as JSON strings rendered from the typed results.

#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;

pub mod analytics;
pub mod calendar;
pub mod indicators;
pub mod pairing;
pub mod replay;
pub mod screener;

/// Renders an indicator failure as a `reason: message` pair (ADR 0012 rule 4) for the
/// wasm boundary, which can only throw the message string.
#[cfg(feature = "js")]
fn indicator_error(error: &indicators::IndicatorError) -> JsError {
    JsError::new(&format!("{}: {error}", error.reason()))
}

/// Full indicator series over `closes`, same length as the input, `NaN` during warm-up.
///
/// `params_json` is a JSON object such as `{"period":14}`. Throws a JavaScript `Error` (a `JsError`
/// carrying a `reason: message` pair) on an unknown indicator, bad params, non-finite input, or
/// finite input whose result overflows `f64` (e.g. an SMA of values near `1e308`); it never
/// returns infinity. The `reason` is the stable [`IndicatorError::reason`] string.
#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn indicator_series(
    name: &str,
    params_json: &str,
    closes: &[f64],
) -> Result<Vec<f64>, JsError> {
    indicators::indicator_series(name, params_json, closes)
        .map_err(|error| indicator_error(&error))
}

/// Full OHLC-input indicator series (currently `atr`) over equal-length `high`, `low` and
/// `close` arrays, same length as the input, `NaN` during warm-up (`period - 1` values).
///
/// Additive companion to [`indicator_series`]: `params_json` is a JSON object such as
/// `{"period":14}`. Throws a JavaScript `Error` (a `JsError` carrying a `reason: message` pair)
/// on an unknown (or close-only) indicator, bad params (period must be an integer in
/// `1..=1000000`), unequal array lengths, non-finite input, or finite input whose result
/// overflows `f64`; it never returns infinity. Empty arrays give an empty result. The `reason` is
/// the stable [`IndicatorError::reason`] string.
#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn ohlc_indicator_series(
    name: &str,
    params_json: &str,
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, JsError> {
    indicators::ohlc_indicator_series(name, params_json, high, low, close)
        .map_err(|error| indicator_error(&error))
}

/// JSON catalog of the available indicators: names, params, outputs and warm-up length.
#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn list_indicators() -> String {
    indicators::list_indicators_json()
}

/// Evaluates one screener predicate over column-oriented bar arrays and reports the outcome.
///
/// `predicate_json` is a JSON `ScreenerFilterPredicate` (e.g.
/// `{"key":"close","op":"gt","value":10}`). The five arrays are equal-length chronological bars.
/// Returns the JSON string `{"result": bool, "metrics": {key: number|null}}` (`null` is a metric
/// still in warm-up). Throws a JavaScript `Error` on malformed predicate JSON, unequal lengths,
/// non-finite input, an inconsistent OHLC row, an unsupported metric, or an operator/operand
/// mismatch.
#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn screener_scan_predicate(
    predicate_json: &str,
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<String, JsError> {
    screener::scan_predicate_json(predicate_json, open, high, low, close, volume)
        .map_err(|e| JsError::new(&e))
}

/// Evaluates an AND/OR (possibly nested) screener filter group over column-oriented bar arrays.
///
/// `group_json` is a JSON `ScreenerFilterGroup`. Returns the same JSON shape and throws the same
/// way as [`screener_scan_predicate`], plus on group shape/size/operator errors.
#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn screener_scan_group(
    group_json: &str,
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<String, JsError> {
    screener::scan_group_json(group_json, open, high, low, close, volume)
        .map_err(|e| JsError::new(&e))
}

/// Parses a Journal v1 NDJSON stream and returns its provenance header as JSON.
///
/// `journal_ndjson` is the whole journal in memory (the first non-empty line is the header,
/// `type: "header"`; the rest are audit records). Returns the `JournalHeader` object as JSON.
/// Throws a JavaScript `Error` on an empty stream, a missing or malformed header, a malformed
/// record line, or an unsupported journal version.
#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn journal_header(journal_ndjson: &str) -> Result<String, JsError> {
    replay::journal_header_json(journal_ndjson).map_err(|e| JsError::new(&e))
}

/// Replays a Journal v1 NDJSON stream and returns the reconstructed state as compact JSON.
///
/// `journal_ndjson` is the whole journal in memory. Returns the stable shape
/// `{"header": <JournalHeader>, "replay": {"trading_state", "dispatches", "positions", "orders",
/// "rejections"}}`, with `positions`/`orders` sorted and `rejections` keyed by order id, so the
/// frontend can show a backtest journal offline. Throws a JavaScript `Error` on the same inputs
/// as [`journal_header`].
#[cfg(feature = "js")]
#[wasm_bindgen]
pub fn replay_journal(journal_ndjson: &str) -> Result<String, JsError> {
    replay::replay_journal_json(journal_ndjson).map_err(|e| JsError::new(&e))
}

#[cfg(test)]
mod tests;
