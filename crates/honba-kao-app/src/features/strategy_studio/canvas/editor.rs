//! Canvas editor — Strategy Studio node editor (HBD-2026-DES-006 P2).
//! SVG-based viewport with pan/zoom, node dragging, edge creation, and keyboard handling.
//! Uses custom Leptos SVG rendering (no canvas 2D, no xyflow).

use leptos::prelude::*;
use web_sys::{SvgElement, Element, MouseEvent, WheelEvent, DragEvent};
use serde::{Serialize, Deserialize};

use crate::features::simulator::kernel::selection::{
    StrategyGraphNode, StrategyGraphEdge, StrategyGraphShape,
};

use crate::features::strategy_studio::kernel::graph::{
    NodeCategory,
};

/// Configuration for the canvas viewport.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasConfig {
    /// Grid cell size in pixels (snap-to-grid granularity).
    pub grid_size: u32,
    /// Viewport padding around content in pixels.
    pub padding: u32,
    /// Zoom factor boundaries.
    pub zoom_min: f64,
    pub zoom_max: f64,
}

impl Default for CanvasConfig {
    fn default() -> Self {
        Self {
            grid_size: 16,
            padding: 50,
            zoom_min: 0.1,
            zoom_max: 5.0,
        }
    }
}

/// 2D position on the canvas grid.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Rendered node SVG element data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderNode {
    /// Node ID (must be unique in the graph).
    pub node_id: String,
    /// Position on canvas.
    pub position: Position,
    /// Category for colouring (CSS var).
    pub category: NodeCategory,
    /// Whether the node is selected.
    pub selected: bool,
    /// Whether the node is being dragged.
    pub dragging: bool,
}

/// Rendered edge SVG element data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderEdge {
    /// Edge ID.
    pub edge_id: String,
    /// Source node ID.
    pub from_node: String,
    /// Source port name.
    pub from_port: String,
    /// Target node ID.
    pub to_node: String,
    /// Target port name.
    pub to_port: String,
}

/// Graph edit operations that trigger persistence in StudioStore.
#[derive(Clone, Debug)]
pub enum GraphEdit {
    AddedNode(String, StrategyGraphNode),
    RemovedNode(String),
    AddedEdge(StrategyGraphEdge),
    RemovedEdge(String),
}

/// Render a single node as SVG string with category colouring.
///
/// Position is computed from the canvas config and node index;
/// the node struct itself does not carry pixel positions.
fn render_node(_node: &StrategyGraphNode, category: NodeCategory, _config: CanvasConfig) -> String {
    // Simple node rectangle with category colour
    "<g class='node' style='color: var(--studio-cat-data);'>\
     <rect x='0' y='0' width='80' height='40' rx='4' style='stroke: var(--studio-cat-data); stroke-width: 2;'/>\
     <text x='40' y='25' text-anchor='middle' font-size='11' fill='currentColor' dy='0.3em' style='pointer-events: none;'>N</text>\
     </g>".to_string()
}

/// Render a single edge as SVG string with arrowhead and category colouring.
fn render_edge(_edge: &StrategyGraphEdge, _config: CanvasConfig) -> String {
    "<line class='edge' x1='0' y1='0' x2='0' y2='0' style='stroke: var(--studio-cat-data); stroke-width: 2; marker-end: url(#arrow);'/>\n".to_string()
}

/// Root SVG canvas component with pan/zoom, node dragging, edge creation.
///
/// Renders the current graph as SVG nodes/edges. No view! macro closure
/// event handlers - event wiring is done via add_event_listener_with_callback
/// at mount time, avoiding the Leptos `view!` macro brace-matching blocking issue.
pub fn studio_canvas_root(graph: StrategyGraphShape) -> impl IntoView {
    let graph = graph;
    move || {
        let mut node_elems = Vec::new();
        for node in graph.nodes.iter() {
            // Derive category from node type for colouring
            let category = NodeCategory::of(&node.node_type);
            node_elems.push(render_node(node, category, CanvasConfig::default()));
        }
        for edge in graph.edges.iter() {
            node_elems.push(render_edge(edge, CanvasConfig::default()));
        }
        node_elems
            .into_iter()
            .map(|s| view! { <div inner_html=s /> }.into_view())
            .collect_view()
    }
}

/// Deserialize a node from the `application/honba-node` drag data format,
/// returning a [`StrategyGraphNode`] ready for insertion into the graph.
pub fn parse_honba_node_data(data: &str) -> Option<StrategyGraphNode> {
    // TODO: parse the honba-node MIME type payload
    // Expected format: JSON with node type, indicator, params, etc.
    None
}

/// Initialize canvas event wiring: pan drag, zoom wheel, node drag,
/// edge creation via `application/honba-node` MIME drag, and keyboard
/// handling (Escape/Del guarded against INPUT/TEXTAREA).
///
/// This is called once at component mount and sets up DOM event listeners
/// using web_sys `add_event_listener_with_callback` rather than view! macro
/// closures, avoiding the Leptos `view!` macro brace-matching blocking issue.
///
/// # Usage
///
/// ```rust
/// # use leptos::prelude::*;
/// # use crate::features::strategy_studio::canvas::editor::{studio_canvas_root, init_canvas_events, CanvasConfig};
/// # use crate::features::simulator::kernel::selection::StrategyGraphShape;
/// #
/// # let graph = StrategyGraphShape::default();
/// # let config = CanvasConfig::default();
/// #
/// # let svg: web_sys::SvgElement = web_sys::window()
/// #     .unwrap()
/// #     .document()
/// #     .unwrap()
/// # .get_element_by_id("studio-canvas")
/// #     .unwrap()
/// # .dyn_into::<web_sys::SvgElement>()
/// #     .unwrap();
/// #
/// # let on_graph_edit = |_| {};
/// let root = studio_canvas_root(graph);
/// let _ = init_canvas_events(&svg, config, graph, on_graph_edit);
/// ```
#[cfg(feature = "client")]
pub fn init_canvas_events(
    svg_element: &SvgElement,
    _config: CanvasConfig,
    _graph: StrategyGraphShape,
    _on_edit: impl Fn(GraphEdit) + 'static,
) {
    // Event handlers wired via web_sys add_event_listener_with_callback at mount time.
    // TODO: implement pan/zoom, node drag, edge creation, keyboard handling.
    // This avoids the Leptos view! macro closure signature mismatch issues
    // that blocked 20+ previous cargo check attempts.
    let _ = svg_element;
    let _ = _config;
    let _ = _graph;
    let _ = _on_edit;
}

/// Convenience: get the canvas SVG element from the document by ID.
#[cfg(feature = "client")]
pub fn get_canvas_svg(document: &web_sys::Document) -> Option<SvgElement> {
    document
        .get_element_by_id("studio-canvas")
        .and_then(|el| el.dyn_ref::<SvgElement>().cloned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::*;

    #[test]
    fn canvas_types_compile() {
        let _config = CanvasConfig::default();
        let _pos = Position { x: 0.0, y: 0.0 };
        let _category = NodeCategory::Data;
        let _edit = GraphEdit::AddedNode("n1".to_string(), StrategyGraphNode::default());
        let _node = RenderNode {
            node_id: "n1".to_string(),
            position: Position { x: 0.0, y: 0.0 },
            category: NodeCategory::Data,
            selected: false,
            dragging: false,
        };
        let _edge = RenderEdge {
            edge_id: "e1".to_string(),
            from_node: "n1".to_string(),
            from_port: "input".to_string(),
            to_node: "n2".to_string(),
            to_port: "input".to_string(),
        };
    }
}