//! The render scene (HBD-2026-DES-006 P2) — pure, no DOM.
//!
//! The design doc's render rule: one `Memo` over graph + positions +
//! selection produces everything the SVG draws, so a 12-node graph is one
//! memo pass, never a per-node or per-port effect. [`build_scene`] is that
//! pass; the editor wraps it in a single `Memo<Scene>`.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::features::simulator::kernel::selection::{StrategyGraphNode, StrategyGraphShape};
use crate::features::strategy_studio::canvas::geometry::{
    auto_layout, input_anchor, node_height, output_anchor, smoothstep_path, Position, NODE_WIDTH,
};
use crate::features::strategy_studio::kernel::graph::{
    node_inputs, node_outputs, NodeCategory, PortKind, ValidationIssue,
};

/// What the editor has selected. Multi-select (rubber band) is out of
/// scope — fe never shipped it either.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Selection {
    /// Nothing: the inspector shows the strategy-level settings.
    #[default]
    None,
    /// One node, by id.
    Node(String),
    /// One edge, by id.
    Edge(String),
}

/// One port row on a card.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderPort {
    /// Port id (`input`, `left`, `value`, …).
    pub id: &'static str,
    /// Data kind (drives the dot colour and connect compatibility).
    pub kind: PortKind,
    /// Absolute anchor x.
    pub x: f64,
    /// Absolute anchor y.
    pub y: f64,
    /// Whether any edge uses this port.
    pub connected: bool,
}

/// One node card, fully placed.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderNode {
    /// Node id.
    pub id: String,
    /// Card title (label, else id).
    pub title: String,
    /// `type · indicator` subtitle.
    pub subtitle: String,
    /// Category (top-border accent).
    pub category: NodeCategory,
    /// Top-left x.
    pub x: f64,
    /// Top-left y.
    pub y: f64,
    /// Card width.
    pub width: f64,
    /// Card height.
    pub height: f64,
    /// Per-kind body chips.
    pub chips: Vec<String>,
    /// Input port rows.
    pub inputs: Vec<RenderPort>,
    /// Output port rows.
    pub outputs: Vec<RenderPort>,
    /// Validation issues naming this node.
    pub issues: Vec<String>,
    /// Whether this card is selected.
    pub selected: bool,
}

/// One edge, fully routed.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderEdge {
    /// Edge id.
    pub id: String,
    /// SVG path `d`.
    pub d: String,
    /// Splice-control x.
    pub mid_x: f64,
    /// Splice-control y.
    pub mid_y: f64,
    /// The data kind it carries (source port kind; `None` when the source
    /// port does not exist, which validation reports as `port_mismatch`).
    pub kind: Option<PortKind>,
    /// Whether this edge is selected.
    pub selected: bool,
}

/// Everything the canvas draws in one pass.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    /// Cards, in graph order (later cards paint on top).
    pub nodes: Vec<RenderNode>,
    /// Edges, in graph order.
    pub edges: Vec<RenderEdge>,
}

impl Scene {
    /// Card heights by node id (the fit-to-view input).
    pub fn heights(&self) -> BTreeMap<String, f64> {
        self.nodes.iter().map(|n| (n.id.clone(), n.height)).collect()
    }

    /// Card positions by node id.
    pub fn positions(&self) -> BTreeMap<String, Position> {
        self.nodes
            .iter()
            .map(|n| (n.id.clone(), Position::new(n.x, n.y)))
            .collect()
    }
}

fn value_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

fn op_symbol(op: &str) -> &str {
    match op {
        "crosses_above" => "crosses ↑",
        "crosses_below" => "crosses ↓",
        "gt" => ">",
        "gte" => "≥",
        "lt" => "<",
        "lte" => "≤",
        "eq" => "=",
        other => other,
    }
}

/// The per-kind body chips (design doc: "per-kind body chips").
pub fn node_chips(graph: &StrategyGraphShape, node: &StrategyGraphNode) -> Vec<String> {
    let param = |name: &str| node.params.get(name).map(value_text);
    let mut chips: Vec<String> = Vec::new();
    match node.node_type.as_str() {
        "universe" => {
            let universe = if !graph.universe_named.is_empty() {
                graph.universe_named.clone()
            } else {
                graph
                    .universe_explicit
                    .iter()
                    .map(|(symbol, exchange)| format!("{symbol}.{exchange}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            if !universe.is_empty() {
                chips.push(universe);
            }
            chips.push(format!("{} {}", graph.driving_interval, graph.driving_aggregation));
        }
        "condition" => {
            chips.push(op_symbol(param("op").as_deref().unwrap_or("gt")).to_string());
            if let Some(threshold) = param("threshold") {
                chips.push(threshold);
            }
        }
        "logic" => chips.push(param("op").unwrap_or_else(|| "and".to_string()).to_uppercase()),
        "order" => {
            chips.push(param("side").unwrap_or_else(|| "buy".to_string()).to_uppercase());
            chips.push(format!("× {}", param("quantity").unwrap_or_else(|| "1".to_string())));
        }
        _ => {
            for (name, value) in node.params.iter().take(3) {
                chips.push(format!("{name} {}", value_text(value)));
            }
        }
    }
    chips
}

fn subtitle(node: &StrategyGraphNode) -> String {
    match &node.indicator {
        Some(kind) => format!("{} · {}", node.node_type, kind),
        None => node.node_type.clone(),
    }
}

/// The single render pass. Nodes with no stored position fall back to the
/// auto-layout slot so nothing ever renders at the origin by accident.
pub fn build_scene(
    graph: &StrategyGraphShape,
    positions: &BTreeMap<String, Position>,
    selection: &Selection,
    issues: &[ValidationIssue],
) -> Scene {
    let fallback = if graph.nodes.iter().all(|n| positions.contains_key(&n.id)) {
        BTreeMap::new()
    } else {
        auto_layout(graph)
    };
    let position_of = |id: &str| {
        positions
            .get(id)
            .or_else(|| fallback.get(id))
            .copied()
            .unwrap_or(Position::new(0.0, 0.0))
    };
    let uses_input = |id: &str, port: &str| {
        graph.edges.iter().any(|e| e.to_node == id && e.to_port == port)
    };
    let uses_output = |id: &str, port: &str| {
        graph.edges.iter().any(|e| e.from_node == id && e.from_port == port)
    };

    let nodes: Vec<RenderNode> = graph
        .nodes
        .iter()
        .map(|node| {
            let position = position_of(&node.id);
            let in_specs = node_inputs(&node.node_type);
            let out_specs = node_outputs(&node.node_type, Some(node));
            let inputs = in_specs
                .iter()
                .enumerate()
                .map(|(i, spec)| {
                    let (x, y) = input_anchor(position, i);
                    RenderPort { id: spec.id, kind: spec.kind, x, y, connected: uses_input(&node.id, spec.id) }
                })
                .collect();
            let outputs = out_specs
                .iter()
                .enumerate()
                .map(|(i, spec)| {
                    let (x, y) = output_anchor(position, i);
                    RenderPort { id: spec.id, kind: spec.kind, x, y, connected: uses_output(&node.id, spec.id) }
                })
                .collect();
            RenderNode {
                id: node.id.clone(),
                title: node.label.clone().unwrap_or_else(|| node.id.clone()),
                subtitle: subtitle(node),
                category: NodeCategory::of(&node.node_type),
                x: position.x,
                y: position.y,
                width: NODE_WIDTH,
                height: node_height(in_specs.len(), out_specs.len()),
                chips: node_chips(graph, node),
                inputs,
                outputs,
                issues: issues
                    .iter()
                    .filter(|issue| issue.node_id.as_deref() == Some(node.id.as_str()))
                    .map(|issue| issue.message.clone())
                    .collect(),
                selected: *selection == Selection::Node(node.id.clone()),
            }
        })
        .collect();

    let port_xy = |node_id: &str, port: &str, output: bool| -> Option<(f64, f64, PortKind)> {
        let node = nodes.iter().find(|n| n.id == node_id)?;
        let ports = if output { &node.outputs } else { &node.inputs };
        ports.iter().find(|p| p.id == port).map(|p| (p.x, p.y, p.kind))
    };

    let edges = graph
        .edges
        .iter()
        .filter_map(|edge| {
            // An edge whose endpoint node is missing has nothing to draw
            // between (validation reports it); a missing *port* still draws,
            // from the card edge, so the mismatch is visible and deletable.
            let from_node = nodes.iter().find(|n| n.id == edge.from_node)?;
            let to_node = nodes.iter().find(|n| n.id == edge.to_node)?;
            let source = port_xy(&edge.from_node, &edge.from_port, true);
            let target = port_xy(&edge.to_node, &edge.to_port, false);
            let (sx, sy) = source
                .map(|(x, y, _)| (x, y))
                .unwrap_or((from_node.x + from_node.width, from_node.y + from_node.height / 2.0));
            let (tx, ty) = target
                .map(|(x, y, _)| (x, y))
                .unwrap_or((to_node.x, to_node.y + to_node.height / 2.0));
            let (d, mid_x, mid_y) = smoothstep_path((sx, sy), (tx, ty));
            Some(RenderEdge {
                id: edge.id.clone(),
                d,
                mid_x,
                mid_y,
                kind: source.map(|(_, _, kind)| kind),
                selected: *selection == Selection::Edge(edge.id.clone()),
            })
        })
        .collect();

    Scene { nodes, edges }
}
