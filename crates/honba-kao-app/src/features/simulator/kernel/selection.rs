//! Simulator strategy selection (HBD-2026-DES-005 P1, UI wiring deferred to
//! P8; graph type defined here in P4's shape per §6 risk register).
//!
//! The Leptos port of honba-fe `core/simulator/select.ts`, minus the parts
//! that depend on the strategy-studio port (library save format, the
//! `?strategy=` slug resolution against saved manifests). honba-kao's
//! `/strategy` page is a placeholder today, so the graph shape is a **minimal
//! local struct** carrying exactly what the run needs, reading the same
//! JSON field names fe's hand-off writes. Until the studio lands, the
//! handoff key is read best-effort and its absence means "template mode".
//!
//! Precedence (fe): the Designer hand-off stashed in `sessionStorage`, then a
//! library strategy named by the `?strategy=` slug, then the first saved
//! strategy, then a built-in template.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Where the selected strategy came from (`SelectionOrigin`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionOrigin {
    /// Came from the strategy studio hand-off.
    Designer,
    /// Came from the saved-strategy library.
    Library,
    /// Came from the built-in template.
    Template,
}

impl SelectionOrigin {
    /// Short badge label.
    pub const fn label(self) -> &'static str {
        match self {
            SelectionOrigin::Designer => "workspace",
            SelectionOrigin::Library => "library",
            SelectionOrigin::Template => "template",
        }
    }

    /// Wire/JSON spelling (stable, matches fe).
    pub const fn id(self) -> &'static str {
        match self {
            SelectionOrigin::Designer => "designer",
            SelectionOrigin::Library => "library",
            SelectionOrigin::Template => "template",
        }
    }

    /// Parses the JSON spelling; unknown ⇒ template (the safe default).
    pub fn from_id(id: &str) -> Self {
        match id {
            "designer" => SelectionOrigin::Designer,
            "library" => SelectionOrigin::Library,
            _ => SelectionOrigin::Template,
        }
    }
}

/// One strategy-graph node (fe `GraphNode`, algodesigner `nodes/types.ts`).
///
/// Carried from P6 onward: the chart bridge maps indicator nodes onto the
/// chart's indicator registry (`kernel/chart_series.rs`), so the shape now
/// reads the same `nodes`/`edges` arrays fe's hand-off JSON writes. P7's
/// replay evaluator consumes condition/logic nodes, and P8's studio port
/// writes the same document without touching this struct.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StrategyGraphNode {
    /// Stable node id (`f`, `s`, …) — also the chart indicator instance key.
    pub id: String,
    /// Node family (fe `NodeType`): `universe | indicator | transform |
    /// condition | logic | sizing | risk | order | schedule | benchmark`.
    #[serde(rename = "type")]
    pub node_type: String,
    /// Optional presentation label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Node parameters, keyed by the graph catalogue's spelling (`period`, …).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, serde_json::Value>,
    /// For indicator nodes: the graph catalogue kind (`sma`, `rsi`, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indicator: Option<String>,
}

/// One strategy-graph edge (fe `GraphEdge`, field names already snake_case).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StrategyGraphEdge {
    pub id: String,
    pub from_node: String,
    pub from_port: String,
    pub to_node: String,
    pub to_port: String,
}

/// The minimal strategy graph shape the simulator needs before the strategy
/// studio exists on kao (§6 risk row "strategy graph source absent").
///
/// Field names match fe's `StrategyGraphShape` hand-off JSON so a future
/// studio port writes the same document without touching this struct. The
/// `nodes`/`edges` arrays are carried since P6 (chart indicator bridge);
/// `schema_version` stays omitted — the sim's manifest pins
/// [`crate::features::simulator::kernel::manifest::STRATEGY_API_VERSION`] instead — and the
/// verifier behind [`graph_is_runnable`]'s fe counterpart stays deferred
/// with the studio port.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StrategyGraphShape {
    /// Node name (presentation only today).
    pub name: String,
    /// Named universe the graph subscribes to (`nifty50`, …).
    pub universe_named: String,
    /// Explicit per-instrument subscriptions `(symbol, exchange)`.
    pub universe_explicit: Vec<(String, String)>,
    /// The driving timeframe as `(interval, aggregation-unit-id)`, e.g.
    /// `(5, "minute")` → `5m`.
    pub driving_interval: u32,
    /// The driving timeframe's aggregation unit spelling.
    pub driving_aggregation: String,
    /// Bars required before the first decision (fe `warmup_bars`), echoed
    /// verbatim into the compile manifest.
    pub warmup_bars: u32,
    /// Graph nodes (fe `nodes`), same JSON field names as the hand-off.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<StrategyGraphNode>,
    /// Graph edges (fe `edges`), same JSON field names as the hand-off.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<StrategyGraphEdge>,
}

/// The handoff record fe's algodesigner stashes for the simulator.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SimulatorHandoff {
    /// Strategy name.
    pub name: String,
    /// The graph itself (fe allows `graph` or `strategy` as the key).
    pub graph: Option<StrategyGraphShape>,
}

/// A graph is runnable when it is structurally usable at all (fe
/// `graphIsRunnable`, with the verifier-based checks deferred until the
/// studio port supplies them; an empty graph is never runnable).
pub fn graph_is_runnable(graph: Option<&StrategyGraphShape>) -> bool {
    graph.is_some()
}

/// The built-in template the simulator always falls back to: fe's first
/// `STRATEGY_TEMPLATES` entry (`sma_cross`, algodesigner
/// `strategy-templates.ts`) — single driving timeframe, TCS/NSE so the
/// default panel is immediately valid, plus the same `nodes`/`edges` the
/// template ships so the chart bridge has two SMA indicator nodes to draw.
/// The node-level verifier checks land with the studio port.
pub fn builtin_template() -> (StrategyGraphShape, &'static str) {
    let node = |id: &str, node_type: &str, label: &str| StrategyGraphNode {
        id: id.to_string(),
        node_type: node_type.to_string(),
        label: (!label.is_empty()).then(|| label.to_string()),
        params: BTreeMap::new(),
        indicator: None,
    };
    let mut fast = node("f", "indicator", "Fast SMA");
    fast.params.insert("period".to_string(), 5.into());
    fast.indicator = Some("sma".to_string());
    let mut slow = node("s", "indicator", "Slow SMA");
    slow.params.insert("period".to_string(), 20.into());
    slow.indicator = Some("sma".to_string());
    let mut cross = node("c", "condition", "Cross");
    cross
        .params
        .insert("op".to_string(), "crosses_above".to_string().into());
    let mut entry = node("o", "order", "Entry Long");
    let mut params: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    params.insert("side".to_string(), "buy".to_string().into());
    params.insert("quantity".to_string(), 1.into());
    entry.params = params;
    let edge = |id: &str, from_node: &str, from_port: &str, to_node: &str, to_port: &str| {
        StrategyGraphEdge {
            id: id.to_string(),
            from_node: from_node.to_string(),
            from_port: from_port.to_string(),
            to_node: to_node.to_string(),
            to_port: to_port.to_string(),
        }
    };
    let graph = StrategyGraphShape {
        name: "SMA Cross".to_string(),
        universe_named: String::new(),
        universe_explicit: vec![("TCS".to_string(), "NSE".to_string())],
        driving_interval: 5,
        driving_aggregation: "minute".to_string(),
        warmup_bars: 2,
        nodes: vec![node("n1", "universe", "Universe"), fast, slow, cross, entry],
        edges: vec![
            edge("e1", "n1", "bars", "f", "input"),
            edge("e2", "n1", "bars", "s", "input"),
            edge("e3", "f", "value", "c", "left"),
            edge("e4", "s", "value", "c", "right"),
            edge("e5", "c", "signal", "o", "signal"),
        ],
    };
    (graph, "SMA Cross")
}

/// fe's `AGGREGATION_UNIT`: the graph's aggregation-unit spelling → the
/// timeframe suffix; an unknown spelling falls to `d` like fe did.
fn aggregation_unit_suffix(aggregation: &str) -> &'static str {
    match aggregation {
        "second" => "s",
        "minute" => "m",
        "hour" => "h",
        "day" => "d",
        "week" => "w",
        "month" => "mo",
        _ => "d",
    }
}

/// Pre-fills the controls from the selected graph's universe and driving
/// timeframe, over the base the caller supplies (fe `controlsForGraph`);
/// everything else keeps `base`. The graph's timeframes win; the panel keeps
/// whatever window/costs it already had — a strategy switch never resets the
/// operator's window.
pub fn controls_for_graph(
    graph: &StrategyGraphShape,
    base: &crate::features::simulator::kernel::controls::SimulatorControls,
) -> crate::features::simulator::kernel::controls::SimulatorControls {
    let first = graph.universe_explicit.first();
    // fe: `first?.symbol ?? graph.universe.named ?? base.symbol`.
    let symbol = match first {
        Some((symbol, _)) if !symbol.is_empty() => symbol.clone(),
        _ => {
            if !graph.universe_named.is_empty() {
                graph.universe_named.clone()
            } else {
                base.symbol.clone()
            }
        }
    };
    let exchange = first
        .map(|(_, exchange)| exchange.clone())
        .unwrap_or_else(|| base.exchange.clone());
    let timeframe = format!(
        "{}{}",
        graph.driving_interval,
        aggregation_unit_suffix(&graph.driving_aggregation)
    );
    let mut controls = base.clone();
    controls.symbol = symbol;
    controls.exchange = exchange;
    controls.timeframe = timeframe;
    controls
}

/// The universe string the run contract expects (fe `universeString`) — a
/// non-empty panel symbol wins, then the graph's named universe, then the
/// graph's explicit subscriptions joined with `,`.
pub fn universe_string(
    graph: &StrategyGraphShape,
    controls: &crate::features::simulator::kernel::controls::SimulatorControls,
) -> String {
    let symbol = controls.symbol.trim().to_uppercase();
    if !symbol.is_empty() {
        let exchange = if controls.exchange.trim().is_empty() {
            "NSE".to_string()
        } else {
            controls.exchange.trim().to_uppercase()
        };
        return format!("{}.{}", symbol, exchange);
    }
    if !graph.universe_named.is_empty() {
        return graph.universe_named.clone();
    }
    graph
        .universe_explicit
        .iter()
        .map(|(symbol, exchange)| {
            format!("{}.{}", symbol.trim().to_uppercase(), exchange.trim().to_uppercase())
        })
        .collect::<Vec<String>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::simulator::kernel::controls::default_controls;

    fn graph() -> StrategyGraphShape {
        builtin_template().0
    }

    #[test]
    fn a_template_graph_fills_the_panel_and_falls_back_all_the_way() {
        // fe's `controlsForGraph` over the template: the explicit universe's
        // first instrument and the driving timeframe pre-fill the panel.
        let controls = controls_for_graph(&graph(), &default_controls());
        assert_eq!(controls.symbol, "TCS");
        assert_eq!(controls.exchange, "NSE");
        assert_eq!(controls.timeframe, "5m");
        // The rest of the panel — window and costs — keeps its values.
        assert_eq!(controls.benchmark, default_controls().benchmark);
        assert_eq!(controls.capital, default_controls().capital);
    }

    #[test]
    fn a_named_universe_names_the_symbol_only_when_no_instrument_is_explicit() {
        let named = StrategyGraphShape {
            universe_named: "nifty50".to_string(),
            universe_explicit: Vec::new(),
            ..graph()
        };
        let controls = controls_for_graph(&named, &default_controls());
        assert_eq!(controls.symbol, "nifty50");
        // No explicit exchange keeps the panel's.
        assert_eq!(controls.exchange, default_controls().exchange);
    }

    #[test]
    fn unknown_aggregation_units_fall_to_the_daily_suffix() {
        let weekly = StrategyGraphShape {
            driving_interval: 1,
            driving_aggregation: "week".to_string(),
            ..graph()
        };
        assert_eq!(controls_for_graph(&weekly, &default_controls()).timeframe, "1w");
        let unusual = StrategyGraphShape {
            driving_aggregation: "quarter".to_string(),
            ..graph()
        };
        // fe's `AGGREGATION_UNIT … ?? 'd'`.
        assert_eq!(controls_for_graph(&unusual, &default_controls()).timeframe, "5d");
    }

    #[test]
    fn the_universe_string_prefers_the_panel_symbol_case_insensitively() {
        let controls = {
            let mut controls = default_controls();
            controls.symbol = " tcs ".to_string();
            controls.exchange = "nse".to_string();
            controls
        };
        assert_eq!(universe_string(&graph(), &controls), "TCS.NSE");
    }

    #[test]
    fn an_unfilled_symbol_panel_falls_back_to_the_graph_universe() {
        let named = StrategyGraphShape {
            universe_named: "nifty50".to_string(),
            universe_explicit: Vec::new(),
            ..graph()
        };
        assert_eq!(universe_string(&named, &default_controls()), "nifty50");

        let controls = {
            let mut controls = default_controls();
            controls.exchange = String::new();
            controls
        };
        // An empty exchange reads as NSE (fe's `?? 'NSE'`).
        assert_eq!(universe_string(&graph(), &controls), "TCS.NSE");
    }
}
