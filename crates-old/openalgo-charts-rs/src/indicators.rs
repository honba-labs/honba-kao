//! Built-in indicator catalog, indicator definitions, and live indicator management.

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::sys;

/// One row of the built-in indicator registry (502 indicators).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub category: String,
    /// `onchart` overlays share the price scale; `pane` get their own pane.
    #[serde(default)]
    pub placement: String,
    #[serde(default)]
    pub inputs: Vec<IndicatorInput>,
}

/// One tunable input of an indicator (mirrors the engine's input spec).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorInput {
    pub key: String,
    #[serde(rename = "type")]
    pub input_type: String,
    pub label: String,
    /// Literal default (`f64` / `bool` / `string` depending on `input_type`).
    #[serde(default)]
    pub default: serde_json::Value,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub step: Option<f64>,
    #[serde(default)]
    pub options: Vec<SelectOption>,
    #[serde(default)]
    pub group: Option<String>,
}

/// Options of a `select` input.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SelectOption {
    pub label: String,
    pub value: serde_json::Value,
}

/// One live indicator instance running on a chart.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveIndicator {
    /// The instance id minted by the engine (also what `removeIndicator` expects).
    pub instance_id: String,
    /// The registry id (`sma`, `rsi`, …).
    #[serde(default)]
    pub indicator_id: String,
    /// Human name for the pane header chip.
    #[serde(default)]
    pub name: String,
}

/// Fetch the full registry of built-in indicators (502 available).
pub fn indicator_catalog() -> Result<Vec<IndicatorDef>, String> {
    let json = sys::bridge_indicator_catalog().map_err(|e| format!("{e:?}"))?;
    let raw = json.as_string().unwrap_or_default();
    serde_json::from_str::<Vec<IndicatorDef>>(&raw).map_err(|e| format!("indicator catalog: {e}"))
}

/// List live indicator instances running on a specific chart.
pub fn chart_indicators(id: f64) -> Result<Vec<LiveIndicator>, String> {
    let json = sys::bridge_chart_indicators(id).map_err(|e| format!("{e:?}"))?;
    let raw = json.as_string().unwrap_or_default();
    serde_json::from_str::<Vec<LiveIndicator>>(&raw).map_err(|e| format!("live indicators: {e}"))
}

/// Add one built-in indicator to a chart, returning the live instance descriptor.
pub fn add_indicator(
    id: f64,
    indicator_id: &str,
    settings: &serde_json::Value,
    instance_id: Option<&str>,
) -> Result<LiveIndicator, String> {
    let settings_json =
        if settings.is_object() && !settings.as_object().is_some_and(|map| map.is_empty()) {
            settings.to_string()
        } else {
            String::new()
        };

    let json =
        sys::bridge_add_indicator(id, indicator_id, &settings_json, instance_id.unwrap_or(""))
            .map_err(|e| format!("{e:?}"))?;

    let raw = json
        .as_string()
        .ok_or_else(|| "engine refused the indicator".to_string())?;

    serde_json::from_str::<LiveIndicator>(&raw).map_err(|e| format!("{e}"))
}

/// Remove one live indicator instance (idempotent).
pub fn remove_indicator(id: f64, instance_id: &str) {
    let _ = sys::bridge_remove_indicator(id, instance_id);
}

/// Register a custom indicator picker dialog (replaces the built-in one).
pub fn register_custom_indicator_picker(mount_fn: &JsValue) {
    let _ = sys::bridge_register_custom_indicator_picker(mount_fn);
}
