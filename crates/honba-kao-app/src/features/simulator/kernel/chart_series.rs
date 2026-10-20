//! Strategy-graph → chart-indicator bridge (fe
//! `core/simulator/chart-series.ts`, port phase P6).
//!
//! The Simulator chart must show *the strategy's* indicators with *the
//! strategy's* parameters, not a generic overlay set — otherwise the chart
//! disagrees with the run it is describing. Every indicator node of the
//! active [`StrategyGraphShape`] is mapped onto the `openalgo-charts`
//! indicator registry (the catalog the bridge exposes as `indicatorCatalog`,
//! cf. `shared/components/charts/openalgo_chart.rs`), with the graph's parameter
//! names translated to the registry's input keys.
//!
//! A kind the registry has no descriptor for (today: `rma`, `psar`,
//! `parabolic_sar`) is reported with `supported: false` and skipped by the
//! chart rather than silently dropped — the canvas still shows which node
//! it could not draw and why.
//!
//! Resolution happens against a caller-supplied catalog slice
//! ([`resolve_specs`]) rather than importing the bundle: wasm drives the
//! bridge's `indicatorCatalog`, native tests feed a hand-built catalog
//! ([`openalgo_charts::indicators::IndicatorDef`] is a plain serde struct).

use std::collections::BTreeMap;

use serde_json::Value;

use crate::features::simulator::kernel::catalogue::catalogue_lookup;
use crate::features::simulator::kernel::selection::{StrategyGraphShape, StrategyGraphNode};
use openalgo_charts::indicators::IndicatorDef;

/// One chart-drawable indicator spec (fe `ChartIndicatorSpec`).
#[derive(Clone, Debug, PartialEq)]
pub struct ChartIndicatorSpec {
    /// Instance key; the node id, so it is stable across edits — a parameter
    /// tweak mutates the instance instead of blinking a new series in.
    pub key: String,
    /// Graph node id ([`key`] verbatim in fe).
    pub node_id: String,
    /// Graph kind, e.g. `sma`.
    pub kind: String,
    /// Registry id to `addIndicator`, or `None` when the kind is known
    /// unsupported.
    pub indicator: Option<String>,
    /// The drawn series' label.
    pub label: String,
    /// Registry settings: descriptor defaults overridden by the graph's params.
    pub settings: BTreeMap<String, Value>,
    /// Whether the registry can draw this.
    pub supported: bool,
    /// Why it is unsupported, for the UI.
    pub reason: Option<String>,
}

/// Graph kind → registry id.
///
/// `None` means "no descriptor exists"; an unlisted kind falls through to
/// itself, so a graph kind the registry happens to provide works without a
/// table entry. (Re-probed against the vendored `openalgo-charts` bundle
/// 2.6.6: its lazy indicator chunk registers `sma`/`ema`/`wma`/`rsi`/`atr`/
/// `macd`/`bollinger`/`donchian`/`supertrend` — the kinds the built-in
/// graphs use — but has no `rma` (Wilder smoothing registers as `smma`) and
/// no `psar`/`parabolic_sar`, matching fe's map.)
const REGISTRY_ID: &[(&str, Option<&str>)] = &[
    ("rma", None),
    ("psar", None),
    ("parabolic_sar", None),
];

/// Graph parameter name → registry input key.
///
/// The graph catalogue names look-back windows `period` (SMA/EMA/RSI) while
/// the registry calls them `length`; MACD is `fast/slow/signal` on both
/// sides but the registry wants `fastPeriod/slowPeriod/signalPeriod`.
const PARAM_ALIAS: &[(&str, &[(&str, &str)])] = &[
    ("sma", &[("period", "length")]),
    ("ema", &[("period", "length")]),
    ("wma", &[("period", "length")]),
    ("rsi", &[("period", "length")]),
    (
        "macd",
        &[
            ("fast", "fastPeriod"),
            ("slow", "slowPeriod"),
            ("signal", "signalPeriod"),
        ],
    ),
    ("bollinger", &[("length", "length"), ("std", "stdDev")]),
    ("donchian", &[("period", "length")]),
    ("atr", &[("period", "period")]),
    ("supertrend", &[("period", "period")]),
];

/// Overlay colours cycled by node order (fe `OVERLAY_COLORS`) so two SMAs of
/// different periods are tellable apart on the price pane.
///
/// These are *canvas* colours handed to the chart engine, not page CSS — but
/// the P6 convention routes them through the `--chart-*` theme tokens: the
/// chart container reads the CSS variables and passes the resolved values
/// here, keeping this constant only as the engine-side fallback.
pub const OVERLAY_COLORS: &[&str] = &[
    "#4f8cff", "#f5a623", "#ab47bc", "#26a69a", "#ef5350", "#787b86",
];

/// Registry id a graph kind resolves to, or `None` when the registry has
/// none (fe `registryIdFor`). Unlisted kinds pass through unchanged —
/// possibly their own id — so the lifetime follows the caller's spelling.
pub fn registry_id_for(kind: &str) -> Option<&str> {
    match REGISTRY_ID.iter().find(|(k, _)| *k == kind) {
        Some((_, mapped)) => *mapped,
        None => Some(kind),
    }
}

/// The `kind`'s alias table slice from [`PARAM_ALIAS`] (empty when the kind
/// has none).
fn param_alias(kind: &str) -> &'static [(&'static str, &'static str)] {
    PARAM_ALIAS
        .iter()
        .find(|(k, _)| *k == kind)
        .map_or(&[], |(_, pairs)| *pairs)
}

/// Translate the graph's parameter names to the registry's input keys (fe
/// `settingsFor`).
///
/// fe'd rules kept: `undefined`/`null` values, non-finite numbers and values
/// that are neither number, string nor boolean are dropped; unaliased keys
/// pass through.
pub fn settings_for(kind: &str, params: &BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    let mut out = BTreeMap::new();
    let aliases = param_alias(kind);
    for (name, value) in params {
        let value = match value {
            Value::Null => continue,
            Value::Number(n) if !n.as_f64().is_some_and(f64::is_finite) => continue,
            Value::Number(_) | Value::String(_) | Value::Bool(_) => value,
            _ => continue,
        };
        let key = aliases
            .iter()
            .find(|(from, _)| *from == name)
            .map_or_else(|| name.clone(), |(_, to)| (*to).to_string());
        out.insert(key, value.clone());
    }
    out
}

/// `String(value)` the way JS spells it — fe `describeKind` — so a JSON
/// `2.0` renders `2`, not `2.0`.
fn js_value_string(value: &Value) -> String {
    match value {
        Value::Number(n) => n
            .as_f64()
            .map(|f| format!("{f}"))
            .unwrap_or_else(|| n.to_string()),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// `SMA 3`, `MACD 12/26/9`, `Bollinger 20/2` — the graph's parameters, in
/// catalogue order (fe `describeKind`).
pub fn describe_kind(kind: &str, params: &BTreeMap<String, Value>) -> String {
    let ordered: Vec<&str> = catalogue_lookup(kind)
        .map(|entry| entry.params.iter().map(|p| p.name.as_str()).collect())
        .unwrap_or_else(|| params.keys().map(String::as_str).collect());
    let values: Vec<String> = ordered
        .into_iter()
        .filter_map(|name| params.get(name).map(js_value_string))
        .collect();
    let base = kind.to_uppercase();
    if values.is_empty() {
        base
    } else {
        format!("{base} {}", values.join("/"))
    }
}

/// The graph's indicator nodes — `type: 'indicator'` or any node that
/// carries an `indicator` spelling (fe `indicatorNodes`).
fn indicator_nodes(graph: &StrategyGraphShape) -> Vec<&StrategyGraphNode> {
    graph
        .nodes
        .iter()
        .filter(|node| node.node_type == "indicator" || node.indicator.is_some())
        .collect()
}

/// Graph indicator nodes → chart specs (pure; `supported` is completed by
/// [`resolve_specs`] once the indicator catalog is loaded — fe
/// `graphSpecs`).
///
/// Overlay colours are cycled by node order so two SMAs of different
/// periods are tellable apart on the price pane; the caller supplies the
/// resolved palette (theme tokens, [`OVERLAY_COLORS`] as fallback).
pub fn graph_specs(
    graph: Option<&StrategyGraphShape>,
    overlay_colors: &[&str],
) -> Vec<ChartIndicatorSpec> {
    let Some(graph) = graph else { return Vec::new() };
    let mut slot_per_kind: BTreeMap<String, usize> = BTreeMap::new();

    indicator_nodes(graph)
        .into_iter()
        .map(|node| {
            let kind = node.indicator.clone().unwrap_or_default();
            let registry_id = if kind.is_empty() { None } else { registry_id_for(&kind) };
            let indicator = registry_id.map(String::from);
            let supported = indicator.is_some();
            let reason = if supported {
                None
            } else {
                Some(format!("no chart indicator registered for '{kind}'"))
            };
            let is_band = registry_id == Some("bollinger") || registry_id == Some("donchian");
            let mut settings = settings_for(&kind, &node.params);
            let slot = slot_per_kind.entry(kind.clone()).or_insert(0);
            let color = overlay_colors[*slot % overlay_colors.len()];
            *slot += 1;
            if is_band {
                settings.insert("basisColor".to_string(), color.into());
                settings.insert("bandColor".to_string(), color.into());
            } else {
                settings.insert("color".to_string(), color.into());
            }
            let label = node
                .label
                .as_ref()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .unwrap_or_else(|| describe_kind(&kind, &node.params));
            ChartIndicatorSpec {
                key: node.id.clone(),
                node_id: node.id.clone(),
                kind,
                indicator,
                label,
                settings,
                supported,
                reason,
            }
        })
        .collect()
}

/// Validate specs against the loaded catalog and complete their settings
/// with the descriptor's own defaults (fe `resolveSpecs`, await-free: the
/// caller passes the catalog the wasm container fetched from the bridge's
/// `indicatorCatalog`).
pub fn resolve_specs(
    specs: Vec<ChartIndicatorSpec>,
    catalog: &[IndicatorDef],
) -> Vec<ChartIndicatorSpec> {
    specs
        .into_iter()
        .map(|spec| {
            let Some(indicator) = spec.indicator.as_deref() else {
                return spec;
            };
            let Some(descriptor) = catalog.iter().find(|def| def.id == indicator) else {
                let mut spec = spec;
                spec.supported = false;
                spec.reason = Some(format!("no chart indicator registered for '{}'", spec.kind));
                return spec;
            };
            let mut settings: BTreeMap<String, Value> = descriptor
                .inputs
                .iter()
                .filter(|input| !input.default.is_null())
                .map(|input| (input.key.clone(), input.default.clone()))
                .collect();
            settings.extend(spec.settings.clone());
            ChartIndicatorSpec { settings, ..spec }
        })
        .collect()
}

/// Specs the chart can actually draw, in graph order (fe `drawableSpecs`).
pub fn drawable_specs(specs: &[ChartIndicatorSpec]) -> Vec<&ChartIndicatorSpec> {
    specs
        .iter()
        .filter(|spec| spec.supported && spec.indicator.is_some())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use openalgo_charts::indicators::IndicatorInput;
    use serde_json::json;

    fn indicator_node(id: &str, indicator: &str, period: Option<i64>) -> StrategyGraphNode {
        let mut node = StrategyGraphNode {
            id: id.to_string(),
            node_type: "indicator".to_string(),
            label: None,
            params: BTreeMap::new(),
            indicator: Some(indicator.to_string()),
        };
        if let Some(period) = period {
            node.params.insert("period".to_string(), json!(period));
        }
        node
    }

    fn graph(nodes: Vec<StrategyGraphNode>) -> StrategyGraphShape {
        StrategyGraphShape {
            nodes,
            ..Default::default()
        }
    }

    #[test]
    fn maps_graph_kinds_to_registry_ids_and_flags_the_kinds_the_registry_lacks() {
        assert_eq!(registry_id_for("sma"), Some("sma"));
        assert_eq!(registry_id_for("rsi"), Some("rsi"));
        assert_eq!(registry_id_for("rma"), None);
        assert_eq!(registry_id_for("psar"), None);
        assert_eq!(registry_id_for("parabolic_sar"), None);
        assert_eq!(registry_id_for("supertrend"), Some("supertrend"));
    }

    #[test]
    fn renames_graph_parameters_to_the_registry_input_keys() {
        let params = json!({"period": 3}).as_object().unwrap().clone()
            .into_iter().collect();
        assert_eq!(settings_for("sma", &params), BTreeMap::from([("length".to_string(), json!(3))]));
        let macd = json!({"fast": 5, "slow": 13, "signal": 4});
        let macd_params = macd.as_object().unwrap().clone().into_iter().collect();
        assert_eq!(
            settings_for("macd", &macd_params),
            BTreeMap::from([
                ("fastPeriod".to_string(), json!(5)),
                ("slowPeriod".to_string(), json!(13)),
                ("signalPeriod".to_string(), json!(4)),
            ]),
        );
        let boll = json!({"length": 10, "std": 2.5}).as_object().unwrap().clone().into_iter().collect();
        assert_eq!(
            settings_for("bollinger", &boll),
            BTreeMap::from([("length".to_string(), json!(10)), ("stdDev".to_string(), json!(2.5))]),
        );
        let atr = json!({"period": 14}).as_object().unwrap().clone().into_iter().collect();
        assert_eq!(settings_for("atr", &atr), BTreeMap::from([("period".to_string(), json!(14))]));
    }

    #[test]
    fn passes_unaliased_keys_through_and_drops_empty_or_non_finite_values() {
        // A non-finite number is unrepresentable in the JSON the settings
        // arrive as; fe dropped `Number.POSITIVE_INFINITY`, so a `null`
        // value and a nested object stand in for the same drop rules.
        let params = json!({"unknown": "x", "nul": null, "deep": {}}).as_object().unwrap().clone().into_iter().collect();
        assert_eq!(
            settings_for("sma", &params),
            BTreeMap::from([("unknown".to_string(), json!("x"))]),
        );
        assert!(settings_for("sma", &BTreeMap::new()).is_empty());
    }

    #[test]
    fn describe_kind_reads_the_graph_parameters_in_catalogue_order() {
        let macd = json!({"slow": 26, "fast": 12, "signal": 9}).as_object().unwrap().clone().into_iter().collect();
        assert_eq!(describe_kind("macd", &macd), "MACD 12/26/9");
        let sma = json!({"period": 3}).as_object().unwrap().clone().into_iter().collect();
        assert_eq!(describe_kind("sma", &sma), "SMA 3");
        let boll = json!({"length": 20, "std": 2.0}).as_object().unwrap().clone().into_iter().collect();
        assert_eq!(describe_kind("bollinger", &boll), "BOLLINGER 20/2");
        assert_eq!(describe_kind("sma", &BTreeMap::new()), "SMA");
    }

    #[test]
    fn describes_kinds_without_a_catalogue_entry_from_the_param_names_it_has() {
        let params = json!({"zzz": 1}).as_object().unwrap().clone().into_iter().collect();
        assert_eq!(describe_kind("zzz_kind", &params), "ZZZ_KIND 1");
    }

    #[test]
    fn turns_every_indicator_node_into_a_spec_keeping_labels_and_ids() {
        let specs = graph_specs(
            Some(&graph(vec![
                StrategyGraphNode { id: "u".into(), node_type: "universe".into(), ..Default::default() },
                StrategyGraphNode { id: "f".into(), node_type: "indicator".into(), label: Some("Fast SMA".into()), indicator: Some("sma".into()), params: BTreeMap::from([("period".into(), json!(3))]) },
                StrategyGraphNode { id: "s".into(), node_type: "indicator".into(), indicator: Some("sma".into()), params: BTreeMap::from([("period".into(), json!(8))]), ..Default::default() },
                StrategyGraphNode { id: "r".into(), node_type: "indicator".into(), indicator: Some("rsi".into()), params: BTreeMap::from([("period".into(), json!(14))]), ..Default::default() },
            ])),
            OVERLAY_COLORS,
        );
        assert_eq!(specs.iter().map(|s| s.key.as_str()).collect::<Vec<_>>(), ["f", "s", "r"]);
        let first = &specs[0];
        assert_eq!(first.indicator.as_deref(), Some("sma"));
        assert_eq!(first.label, "Fast SMA");
        assert!(first.supported);
        assert_eq!(first.settings.get("length"), Some(&json!(3)));
        assert!(first.settings.contains_key("color"));
        // fe's label fallback: the catalogue-ordered description.
        assert_eq!(specs[2].label, "RSI 14");
    }

    #[test]
    fn cycles_overlay_colours_so_two_periods_of_one_kind_are_tellable_apart() {
        let specs = graph_specs(
            Some(&graph(vec![indicator_node("f", "sma", None), indicator_node("s", "sma", None)])),
            OVERLAY_COLORS,
        );
        assert_eq!(specs.len(), 2);
        let color_of = |spec: &ChartIndicatorSpec| spec.settings.get("color").cloned().unwrap();
        assert_ne!(color_of(&specs[0]), color_of(&specs[1]));
    }

    #[test]
    fn colours_band_indicators_by_basis_and_band_not_by_the_line_colour() {
        let specs = graph_specs(
            Some(&graph(vec![StrategyGraphNode {
                id: "bb".into(),
                node_type: "indicator".into(),
                indicator: Some("bollinger".into()),
                params: BTreeMap::from([("length".into(), json!(20)), ("std".into(), json!(2.0))]),
                ..Default::default()
            }])),
            OVERLAY_COLORS,
        );
        assert!(!specs[0].settings.contains_key("color"));
        assert_eq!(specs[0].settings.get("basisColor"), Some(&json!(OVERLAY_COLORS[0])));
        assert_eq!(specs[0].settings.get("bandColor"), Some(&json!(OVERLAY_COLORS[0])));
    }

    #[test]
    fn marks_a_kind_the_registry_has_no_descriptor_for_as_unsupported_with_a_reason() {
        let specs = graph_specs(
            Some(&graph(vec![indicator_node("r", "rma", None)])),
            OVERLAY_COLORS,
        );
        assert!(!specs[0].supported);
        assert_eq!(specs[0].indicator, None);
        assert!(specs[0].reason.as_deref().unwrap_or("").contains("rma"));
    }

    #[test]
    fn is_empty_for_a_graph_with_no_indicator_nodes() {
        assert!(graph_specs(Some(&graph(Vec::new())), OVERLAY_COLORS).is_empty());
        assert!(graph_specs(None, OVERLAY_COLORS).is_empty());
    }

    fn catalog() -> Vec<IndicatorDef> {
        vec![
            IndicatorDef {
                id: "sma".to_string(),
                name: "SMA".to_string(),
                category: "moving_average".to_string(),
                placement: "overlay".to_string(),
                inputs: vec![
                    IndicatorInput {
                        key: "length".to_string(),
                        input_type: "number".to_string(),
                        label: "Length".to_string(),
                        default: json!(20),
                        min: Some(1.0),
                        max: None,
                        step: Some(1.0),
                        options: Vec::new(),
                        group: None,
                    },
                    IndicatorInput {
                        key: "source".to_string(),
                        input_type: "select".to_string(),
                        label: "Source".to_string(),
                        default: json!("close"),
                        min: None,
                        max: None,
                        step: None,
                        options: Vec::new(),
                        group: None,
                    },
                ],
            },
            IndicatorDef {
                id: "rsi".to_string(),
                name: "RSI".to_string(),
                category: String::new(),
                placement: "pane".to_string(),
                inputs: Vec::new(),
            },
        ]
    }

    #[test]
    fn resolve_completes_settings_with_descriptor_defaults_but_the_graph_period_drives_the_registry_length() {
        let specs = graph_specs(
            Some(&graph(vec![indicator_node("f", "sma", Some(3))])),
            OVERLAY_COLORS,
        );
        let resolved = resolve_specs(specs, &catalog());
        assert!(resolved[0].supported);
        // The graph's `period: 3` beats the descriptor default; the
        // descriptor defaults fill the rest (`source: close`).
        assert_eq!(resolved[0].settings.get("length"), Some(&json!(3)));
        assert_eq!(resolved[0].settings.get("source"), Some(&json!("close")));
    }

    #[test]
    fn resolve_reports_an_unknown_kind_as_unsupported_rather_than_attaching_nothing_silently() {
        let specs = graph_specs(
            Some(&graph(vec![StrategyGraphNode {
                id: "z".into(),
                node_type: "indicator".into(),
                indicator: Some("not_a_real_indicator".into()),
                ..Default::default()
            }])),
            OVERLAY_COLORS,
        );
        // The pure pass trusts the graph; only the catalog knows better —
        // exactly fe's `graphSpecs` then `resolveSpecs` split.
        assert!(specs[0].supported);
        let resolved = resolve_specs(specs, &catalog());
        assert!(!resolved[0].supported);
        assert!(resolved[0].reason.as_deref().unwrap_or("").contains("not_a_real_indicator"));
        // drawable_specs never hands such a spec to the bridge.
        assert!(drawable_specs(&resolved).is_empty());
    }

    #[test]
    fn drawable_specs_keep_only_supported_specs_in_graph_order() {
        let resolved = resolve_specs(
            graph_specs(
                Some(&graph(vec![indicator_node("f", "sma", Some(3)), indicator_node("r", "rma", None)])),
                OVERLAY_COLORS,
            )
            .into_iter()
            .map(|spec| {
                // `rma` has no registry id in the graph pass; a spec the
                // registry lacks must carry the unsupported verdict before
                // drawing (as unknown kinds do after `resolve_specs`).
                if spec.indicator.is_none() {
                    ChartIndicatorSpec { supported: false, ..spec }
                } else {
                    spec
                }
            })
            .collect(),
            &catalog(),
        );
        let drawable = drawable_specs(&resolved);
        assert_eq!(drawable.iter().map(|s| s.key.as_str()).collect::<Vec<_>>(), ["f"]);
    }
}
