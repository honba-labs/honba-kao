//! Pure graph edits (HBD-2026-DES-006 P2 "store wiring").
//!
//! The Leptos port of the graph half of honba-fe algodesigner `store.ts`
//! (`addNode`, `deleteNode`, `deleteEdge`, `spliceEdge`, `onConnect`,
//! `updateNodeParams`, `updateNodeLabel`, `updateNodeIndicator`). The
//! store calls these and owns nothing but the signals; canvas positions are
//! deliberately *not* a parameter here — they are view state and stay out
//! of the graph (and so out of `to_manifest`).
//!
//! Deviations from fe, each a fix of an fe hole:
//! - [`connect`] refuses kind mismatches, self-loops and cycles up front
//!   (fe accepted the edge and let validation flag it), and a second edge
//!   into an already-fed input *replaces* the first (one source per input).
//! - [`splice_edge`] refuses a splice whose ports do not type-check on both
//!   sides (fe offered "Buy Order" on every bool edge, which orphaned the
//!   downstream input).
//! - [`set_indicator`] drops edges leaving output ports the new kind no
//!   longer has (`macd.histogram` → `sma` has no `histogram`).

use crate::features::simulator::kernel::selection::{
    StrategyGraphEdge, StrategyGraphNode, StrategyGraphShape,
};
use crate::features::strategy_studio::kernel::graph::{
    PortKind, catalogue_entry, edge_id, first_cycle_node, node_inputs, node_outputs,
};

/// Why an edit was refused. Messages are rendered verbatim in the editor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditError {
    /// A node id the edit names does not exist.
    UnknownNode(String),
    /// An edge id the edit names does not exist.
    UnknownEdge(String),
    /// A port the edit names does not exist on that node.
    UnknownPort { node: String, port: String },
    /// The two ports carry different data kinds.
    KindMismatch { from: PortKind, to: PortKind },
    /// An edge from a node to itself.
    SelfLoop,
    /// The edge would close a cycle.
    Cycle,
}

impl EditError {
    /// Human message for the editor's status line.
    pub fn message(&self) -> String {
        match self {
            EditError::UnknownNode(id) => format!("unknown node `{id}`"),
            EditError::UnknownEdge(id) => format!("unknown edge `{id}`"),
            EditError::UnknownPort { node, port } => format!("`{node}` has no port `{port}`"),
            EditError::KindMismatch { from, to } => format!(
                "cannot connect a {} output to a {} input",
                from.as_str(),
                to.as_str()
            ),
            EditError::SelfLoop => "a node cannot feed itself".to_string(),
            EditError::Cycle => "that connection would create a cycle".to_string(),
        }
    }
}

fn find_node<'a>(graph: &'a StrategyGraphShape, id: &str) -> Option<&'a StrategyGraphNode> {
    graph.nodes.iter().find(|n| n.id == id)
}

/// The kind of a node's output port, when it exists.
pub fn output_kind(graph: &StrategyGraphShape, node_id: &str, port: &str) -> Option<PortKind> {
    let node = find_node(graph, node_id)?;
    node_outputs(&node.node_type, Some(node))
        .iter()
        .find(|p| p.id == port)
        .map(|p| p.kind)
}

/// The kind of a node's input port, when it exists.
pub fn input_kind(graph: &StrategyGraphShape, node_id: &str, port: &str) -> Option<PortKind> {
    let node = find_node(graph, node_id)?;
    node_inputs(&node.node_type)
        .iter()
        .find(|p| p.id == port)
        .map(|p| p.kind)
}

/// fe's unique-id rule: `base`, then `base_1`, `base_2`, … until free.
pub fn unique_node_id(graph: &StrategyGraphShape, base: &str) -> String {
    let base = if base.trim().is_empty() {
        "node"
    } else {
        base.trim()
    };
    if find_node(graph, base).is_none() {
        return base.to_string();
    }
    let mut counter = 1usize;
    loop {
        let candidate = format!("{base}_{counter}");
        if find_node(graph, &candidate).is_none() {
            return candidate;
        }
        counter += 1;
    }
}

/// The id base a fresh node takes: the indicator kind, else the node type.
pub fn id_base(node: &StrategyGraphNode) -> String {
    node.indicator
        .clone()
        .unwrap_or_else(|| node.node_type.clone())
}

/// Appends `node` under a unique id (its own id when free, else the
/// [`id_base`] with a counter) and returns the id it landed under.
pub fn add_node(graph: &mut StrategyGraphShape, mut node: StrategyGraphNode) -> String {
    let base = if node.id.trim().is_empty() {
        id_base(&node)
    } else {
        node.id.clone()
    };
    let id = unique_node_id(graph, &base);
    node.id = id.clone();
    graph.nodes.push(node);
    id
}

/// Removes a node and every edge touching it. `false` when absent.
pub fn remove_node(graph: &mut StrategyGraphShape, node_id: &str) -> bool {
    let before = graph.nodes.len();
    graph.nodes.retain(|n| n.id != node_id);
    if graph.nodes.len() == before {
        return false;
    }
    graph
        .edges
        .retain(|e| e.from_node != node_id && e.to_node != node_id);
    true
}

/// Removes one edge. `false` when absent.
pub fn remove_edge(graph: &mut StrategyGraphShape, edge_id: &str) -> bool {
    let before = graph.edges.len();
    graph.edges.retain(|e| e.id != edge_id);
    graph.edges.len() != before
}

/// Connects `from.from_port → to.to_port` (fe `onConnect`), returning the
/// edge id. The graph is unchanged on error. Re-connecting an identical
/// edge is a no-op that returns the existing id; a different source into an
/// already-fed input replaces the old edge.
pub fn connect(
    graph: &mut StrategyGraphShape,
    from: &str,
    from_port: &str,
    to: &str,
    to_port: &str,
) -> Result<String, EditError> {
    if find_node(graph, from).is_none() {
        return Err(EditError::UnknownNode(from.to_string()));
    }
    if find_node(graph, to).is_none() {
        return Err(EditError::UnknownNode(to.to_string()));
    }
    if from == to {
        return Err(EditError::SelfLoop);
    }
    let out_kind = output_kind(graph, from, from_port).ok_or_else(|| EditError::UnknownPort {
        node: from.to_string(),
        port: from_port.to_string(),
    })?;
    let in_kind = input_kind(graph, to, to_port).ok_or_else(|| EditError::UnknownPort {
        node: to.to_string(),
        port: to_port.to_string(),
    })?;
    if out_kind != in_kind {
        return Err(EditError::KindMismatch {
            from: out_kind,
            to: in_kind,
        });
    }
    if let Some(existing) = graph.edges.iter().find(|e| {
        e.from_node == from && e.from_port == from_port && e.to_node == to && e.to_port == to_port
    }) {
        return Ok(existing.id.clone());
    }

    let mut next = graph.clone();
    next.edges
        .retain(|e| !(e.to_node == to && e.to_port == to_port));
    let id = edge_id(from, from_port, to, to_port);
    next.edges.push(StrategyGraphEdge {
        id: id.clone(),
        from_node: from.to_string(),
        from_port: from_port.to_string(),
        to_node: to.to_string(),
        to_port: to_port.to_string(),
    });
    if first_cycle_node(&next).is_some() {
        return Err(EditError::Cycle);
    }
    *graph = next;
    Ok(id)
}

/// Whether a node of this shape can be spliced into an edge carrying
/// `edge_kind` that lands on an input of `target_kind`: its `in_port` must
/// accept the edge's kind and its `out_port` must feed the target's input.
pub fn splice_fits(
    node: &StrategyGraphNode,
    in_port: &str,
    out_port: &str,
    edge_kind: PortKind,
    target_kind: PortKind,
) -> bool {
    let input_ok = node_inputs(&node.node_type)
        .iter()
        .any(|p| p.id == in_port && p.kind == edge_kind);
    let output_ok = node_outputs(&node.node_type, Some(node))
        .iter()
        .any(|p| p.id == out_port && p.kind == target_kind);
    input_ok && output_ok
}

/// fe `spliceEdge`: replaces `a → b` with `a → node → b`, returning the new
/// node's id. Refused (graph unchanged) when the node's ports do not
/// type-check on both sides.
pub fn splice_edge(
    graph: &mut StrategyGraphShape,
    edge_id_to_split: &str,
    node: StrategyGraphNode,
    in_port: &str,
    out_port: &str,
) -> Result<String, EditError> {
    let edge = graph
        .edges
        .iter()
        .find(|e| e.id == edge_id_to_split)
        .cloned()
        .ok_or_else(|| EditError::UnknownEdge(edge_id_to_split.to_string()))?;
    let edge_kind = output_kind(graph, &edge.from_node, &edge.from_port).ok_or_else(|| {
        EditError::UnknownPort {
            node: edge.from_node.clone(),
            port: edge.from_port.clone(),
        }
    })?;
    let target_kind =
        input_kind(graph, &edge.to_node, &edge.to_port).ok_or_else(|| EditError::UnknownPort {
            node: edge.to_node.clone(),
            port: edge.to_port.clone(),
        })?;
    if !splice_fits(&node, in_port, out_port, edge_kind, target_kind) {
        let node_in = node_inputs(&node.node_type)
            .iter()
            .find(|p| p.id == in_port)
            .map(|p| p.kind)
            .unwrap_or(edge_kind);
        return Err(EditError::KindMismatch {
            from: edge_kind,
            to: node_in,
        });
    }

    let mut next = graph.clone();
    let new_id = add_node(&mut next, node);
    next.edges.retain(|e| e.id != edge.id);
    next.edges.push(StrategyGraphEdge {
        id: edge_id(&edge.from_node, &edge.from_port, &new_id, in_port),
        from_node: edge.from_node.clone(),
        from_port: edge.from_port.clone(),
        to_node: new_id.clone(),
        to_port: in_port.to_string(),
    });
    next.edges.push(StrategyGraphEdge {
        id: edge_id(&new_id, out_port, &edge.to_node, &edge.to_port),
        from_node: new_id.clone(),
        from_port: out_port.to_string(),
        to_node: edge.to_node.clone(),
        to_port: edge.to_port.clone(),
    });
    *graph = next;
    Ok(new_id)
}

/// Writes one parameter (fe `updateNodeParams` for a single key).
pub fn set_param(
    graph: &mut StrategyGraphShape,
    node_id: &str,
    name: &str,
    value: serde_json::Value,
) -> bool {
    match graph.nodes.iter_mut().find(|n| n.id == node_id) {
        Some(node) => {
            node.params.insert(name.to_string(), value);
            true
        }
        None => false,
    }
}

/// Removes one parameter (e.g. clearing an optional `threshold`).
pub fn remove_param(graph: &mut StrategyGraphShape, node_id: &str, name: &str) -> bool {
    graph
        .nodes
        .iter_mut()
        .find(|n| n.id == node_id)
        .is_some_and(|node| node.params.remove(name).is_some())
}

/// fe `updateNodeLabel`; an empty label clears it.
pub fn set_label(graph: &mut StrategyGraphShape, node_id: &str, label: &str) -> bool {
    match graph.nodes.iter_mut().find(|n| n.id == node_id) {
        Some(node) => {
            let trimmed = label.trim();
            node.label = (!trimmed.is_empty()).then(|| trimmed.to_string());
            true
        }
        None => false,
    }
}

/// fe `updateNodeIndicator`: switches the kind, resets params to the
/// catalogue defaults and relabels. Edges leaving output ports the new kind
/// lacks are dropped so no edge dangles from a vanished handle.
pub fn set_indicator(graph: &mut StrategyGraphShape, node_id: &str, kind: &str) -> bool {
    let Some(node) = graph.nodes.iter_mut().find(|n| n.id == node_id) else {
        return false;
    };
    node.indicator = Some(kind.to_string());
    node.params.clear();
    if let Some(entry) = catalogue_entry(kind) {
        for param in entry.params {
            node.params
                .insert(param.name.to_string(), param.json_default());
        }
    }
    node.label = Some(kind.to_uppercase());
    let outputs: Vec<&'static str> = node_outputs(&node.node_type, Some(&*node))
        .iter()
        .map(|p| p.id)
        .collect();
    graph
        .edges
        .retain(|e| e.from_node != node_id || outputs.contains(&e.from_port.as_str()));
    true
}
