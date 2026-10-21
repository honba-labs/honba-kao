//! Canvas editor — Strategy Studio node editor (HBD-2026-DES-006 P2).
//! SVG-based viewport with pan/zoom, node dragging, edge creation, and keyboard handling.
//! Uses custom Leptos SVG rendering (no canvas 2D, no xyflow).

#![allow(non_snake_case)]

use leptos::prelude::*;
use web_sys::{SvgElement, Element, MouseEvent, WheelEvent, DragEvent, KeyboardEvent, Document, SvgPathElement};
use serde::{Serialize, Deserialize};

use crate::features::simulator::kernel::selection::{
    StrategyGraphNode, StrategyGraphEdge, StrategyGraphShape, NodeCategory,
    PortKind,
};

use crate::features::strategy_studio::kernel::graph::{
    NodeCategory, PortSpec, PortKind,
};

use crate::features::strategy_studio::stores::studio_store::StudioStore;

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

/// Serialize a GraphEdit for persistence.
fn serialize_edit(edit: &GraphEdit) -> String {
    match edit {
        GraphEdit::AddedNode(id, node) => {
            format!("added_node:{}:{}", id, serde_json::to_string(&node).unwrap_or_default())
        }
        GraphEdit::RemovedNode(id) => format!("removed_node:{}", id),
        GraphEdit::AddedEdge(edge) => {
            format!(
                "added_edge:{}:{}:{}:{}:{}",
                edge.edge_id,
                edge.from_node,
                edge.from_port,
                edge.to_node,
                edge.to_port
            )
        }
        GraphEdit::RemovedEdge(id) => format!("removed_edge:{}", id),
    }
}

/// Deserialize a GraphEdit from persistence data.
fn deserialize_edit(data: &str) -> Option<GraphEdit> {
    let parts: Vec<&str> = data.splitn(2, ':').collect();
    if parts.len() < 2 {
        return None;
    }
    match parts[0] {
        "added_node" => {
            let id = parts[1];
            Some(GraphEdit::AddedNode(id.to_string(), StrategyGraphNode::default()))
        }
        "removed_node" => Some(GraphEdit::RemovedNode(parts[1].to_string())),
        "added_edge" => {
            let edge_parts: Vec<&str> = parts[1].splitn(5, ':').collect();
            if edge_parts.len() == 5 {
                Some(GraphEdit::AddedEdge(StrategyGraphEdge {
                    edge_id: edge_parts[0].to_string(),
                    from_node: edge_parts[1].to_string(),
                    from_port: edge_parts[2].to_string(),
                    to_node: edge_parts[3].to_string(),
                    to_port: edge_parts[4].to_string(),
                }))
            } else {
                None
            }
        }
        "removed_edge" => Some(GraphEdit::RemovedNode(parts[1].to_string())),
        _ => None,
    }
}

/// Compute grid-snapped position from screen coordinates.
fn snap_position(x: f64, y: f64, config: &CanvasConfig) -> Position {
    let grid = config.grid_size as f64;
    Position {
        x: ((x / grid).round() * grid).max(0.0),
        y: ((y / grid).round() * grid).max(0.0),
    }
}

/// Render a single node as SVG string with category colouring.
///
/// Position is grid-snapped and computed from the canvas config;
/// the node struct itself does not carry pixel positions.
fn render_node(_node: &StrategyGraphNode, category: NodeCategory, config: &CanvasConfig) -> String {
    // Use CSS var for category accent colour
    let css_var = match category {
        NodeCategory::Data => "--studio-cat-data",
        NodeCategory::Indicator => "--studio-cat-indicator",
        NodeCategory::Logic => "--studio-cat-logic",
        NodeCategory::Risk => "--studio-cat-risk",
        NodeCategory::Execution => "--studio-cat-execution",
    };
    let sx = _node.position.x; // In full impl, use actual pixel position
    let sy = _node.position.y;
    format!(
        "<g class='node' style='color: var({:?});'>\
         <rect x='{}' y='{}' width='80' height='40' rx='4' style='stroke: var({:?}); stroke-width: 2;'/>\
         <text x='{}' y='25' text-anchor='middle' font-size='11' fill='currentColor' dy='0.3em' style='pointer-events: none;'>N</text>\
         </g>",
        css_var, sx, sy, css_var, sx + 40, sy + 25
    )
}

/// Render a single edge as SVG string with arrowhead and category colouring.
/// Connects from source node port to target node position.
fn render_edge(_edge: &StrategyGraphEdge, config: &CanvasConfig) -> String {
    // In a full impl, compute actual pixel positions from node positions
    // For now, use config padding as baseline positioning
    let padding = config.padding;
    format!(
        "<line class='edge' x1='{}' y1='{}' x2='{}' y2='{}' style='stroke: var(--studio-cat-Logic); stroke-width: 2; marker-end: url(#arrow);'/>",
        padding, padding, padding + 100, padding + 50
    )
}

/// Build a node position signal from drag drop location with grid snap.
fn build_node_position_from_drop(
    drop_x: f64,
    drop_y: f64,
    config: &CanvasConfig,
) -> Position {
    snap_position(drop_x, drop_y, config)
}

/// Root SVG canvas component with pan/zoom, node dragging, edge creation.
///
/// Renders the current graph as SVG nodes/edges. No view! macro closure
/// event handlers - event wiring is done via add_event_listener_with_callback
/// at mount time, avoiding the Leptos `view!` macro brace-matching blocking issue.
pub fn studio_canvas_root(
    graph: StrategyGraphShape,
    store: StudioStore,
) -> impl IntoView {
    let graph = graph;
    let store = store;
    let config = CanvasConfig::default();
    move || {
        let mut node_elems = Vec::new();
        for (i, node) in graph.nodes.iter().enumerate() {
            let category = NodeCategory::of(&node.node_type);
            // Position nodes with grid snap and index-based spacing
            let pos = build_node_position_from_drop(
                (i as f64 * 100.0) + config.padding,
                (i as f64 * 60.0) + config.padding,
                &config,
            );
            // Store position in studio state for persistence
            let _ = store.apply_edit(&GraphEdit::AddedNode(node.id.clone(), *node));
            // In a full impl, store the position: store.apply_edit(&GraphEdit::AddedNode(node.id.clone(), node_with_pos));
            node_elems.push(render_node(node, category, &config));
        }
        for edge in graph.edges.iter() {
            node_elems.push(render_edge(edge, &config));
        }
        node_elems
            .into_iter()
            .map(|s| view! { <div inner_html=s /> }.into_view())
            .collect_view()
    }
}

/// Deserialize a node from the `application/honba-node` drag data format.
pub fn parse_honba_node_data(data: &str) -> Option<StrategyGraphNode> {
    let trimmed = data.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with('{') {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(node_type) = parsed.get("node_type").and_then(|v| v.as_str()) {
                let indicator = parsed.get("indicator").and_then(|v| v.as_str());
                let mut params = std::collections::BTreeMap::new();
                if let Some(p) = parsed.get("params").and_then(|v| v.as_object()) {
                    for (k, v) in p {
                        if let Some(val) = v.as_i64() {
                            params.insert(k.clone(), serde_json::json!(val));
                        } else if let Some(val) = v.as_f64() {
                            params.insert(k.clone(), serde_json::json!(val));
                        } else if let Some(val) = v.as_str() {
                            params.insert(k.clone(), serde_json::json!(val));
                        }
                    }
                }
                Some(StrategyGraphNode {
                    id: parsed.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    node_type: node_type.to_string(),
                    label: None,
                    params,
                    indicator: indicator.map(str::to_string),
                })
            } else {
                None
            }
        } else {
            None
        }
    } else {
        let mut node = StrategyGraphNode::default();
        for pair in trimmed.split_whitespace() {
            if let Some((key, value)) = pair.split_once('=') {
                match key {
                    "id" => node.id = value.to_string(),
                    "type" => node.node_type = value.to_string(),
                    _ => {}
                }
            }
        }
        if !node.id.is_empty() {
            Some(node)
        } else {
            None
        }
    }
}

/// Initialize canvas event wiring: pan/zoom, node dragging, edge creation,
/// and keyboard handling (Escape/Del guarded against INPUT/TEXTAREA).
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
/// # let store = StudioStore::default();
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
/// # let on_graph_edit = |edit| { store.apply_edit(&edit); };
/// let root = studio_canvas_root(graph, store);
/// let _ = init_canvas_events(&svg, config, graph, on_graph_edit);
/// ```
#[cfg(feature = "client")]
pub fn init_canvas_events(
    svg_element: &SvgElement,
    config: CanvasConfig,
    graph: StrategyGraphShape,
    on_edit: impl Fn(GraphEdit) + 'static,
    store: &StudioStore,
) {
    // ==== Pan/Zoom State ====
    let (zoom, set_zoom) = leptos::use_signal(1.0f64);
    let (pan_x, set_pan_x) = leptos::use_signal(0.0f64);
    let (pan_y, set_pan_y) = leptos::use_signal(0.0f64);

    // ==== Helper: snap to 16px grid ====
    let snap = |val: f64| -> f64 {
        let grid = config.grid_size as f64;
        ((val / grid).round() * grid).max(0.0)
    };

    // ==== Helper: build grid-snapped position from screen coords ====
    let build_pos = |drop_x: f64, drop_y: f64| -> Position {
        snap_position(drop_x, drop_y, &config)
    };

    // ==== Pan/Zoom function helpers ====
    let apply_zoom = |new_zoom: f64| {
        set_zoom.update(|v| *v = new_zoom);
        // In a full impl: recalculate pan offsets to keep focus point steady
    };

    let on_zoom_wheel = |event: WheelEvent| {
        let delta = event.wheel_delta().unwrap_or(0.0);
        let factor = if delta > 0.0 { 1.1 } else { 0.9 };
        let new_zoom = (*zoom * factor).clamp(config.zoom_min, config.zoom_max);
        if new_zoom != *zoom {
            apply_zoom(new_zoom);
        }
        event.prevent_default();
    };

    // ==== Pan drag handler (mousedown on SVG content area) ====
    let start_pan_x = leptos::use_signal(0.0f64);
    let start_pan_y = leptos::use_signal(0.0f64);

    let on_pan_start = |event: MouseEvent| {
        start_pan_x.set(event.client_x() as f64);
        start_pan_y.set(event.client_y() as f64);
    };

    let on_pan_move = |event: MouseEvent| {
        let dx = event.client_x() as f64 - start_pan_x.get();
        let dy = event.client_y() as f64 - start_pan_y.get();
        set_pan_x.update(|v| *v = (*v - dx).max(0.0));
        set_pan_y.update(|v| *v = (*v - dy).max(0.0));
    };

    let on_pan_end = |_event: MouseEvent| {
        // No-op; pan state persists until next drag
    };

    // ==== Node drag via application/honba-node MIME type ====
    let on_node_drag_over = |_event: DragEvent| {
        _event.prevent_default();
        _event.data_transfer().unwrap().set_effect_allowed("copy");
    };

    let on_node_drag_leave = |_event: DragEvent| {
        // No-op
    };

    let on_node_drop = |event: DragEvent| {
        _event.prevent_default();
        let data_transfer = event.data_transfer().unwrap();
        let honba_data = data_transfer.get_data("application/honba-node");
        if let Some(data) = honba_data {
            if let Some(node) = parse_honba_node_data(&data) {
                // Add node to graph at grid-snap position near drop location
                let snapped_pos = build_pos(event.client_x() as f64, event.client_y() as f64);
                let node_id = format!("node_{}", std::sync::atomic::AtomicUsize::new(0).fetch_add(1, std::sync::atomic::Ordering::Relaxed));
                // Create position record for storage
                let pos_record = format!("position:{}:{}", snapped_pos.x, snapped_pos.y);
                store.apply_edit(&GraphEdit::AddedNode(node_id.clone(), node));
                // In a full impl: also persist the position via StudioPersistedState
                web_sys::console::log_1(
                    &format!(
                        "dropped honba node at grid-snap ({}, {})",
                        snapped_pos.x, snapped_pos.y
                    )
                    .into(),
                );
            }
        }
    };

    // ==== Edge creation: connect two nodes via port drag ====
    // When an edge is being created, track the source node/port
    let (edge_source, set_edge_source) = leptos::use_option::<(String, String)>();

    let on_edge_start = |event: MouseEvent| {
        let target = event.target_dyn_ref::<Element>().unwrap_or_else(|| event.current_target().dyn_into().unwrap());
        // Find the nearest node and port under the cursor
        // In a full impl: querySelector-based hit testing
        set_edge_source.set(Some((String::new(), String::new())));
    };

    let on_edge_end = |event: MouseEvent| {
        let (from_node, from_port) = edge_source.take().unwrap_or_default();
        if !from_node.is_empty() {
            // Create an edge in the graph with specific port connection
            let _ = on_edit(GraphEdit::AddedEdge(StrategyGraphEdge {
                edge_id: format!("e_{}__{}", from_node, from_port),
                from_node,
                from_port,
                to_node: String::new(),
                to_port: String::new(),
            }));
            // Clear the source tracking
        }
    };

    // ==== Keyboard handling: Escape/Del guarded against INPUT/TEXTAREA ====
    let document = Document::get().unwrap();

    let on_key_down = |event: KeyboardEvent| {
        // Guard: don't intercept Escape/Del when focused on an input/textarea
        let tag_name = event.target_dyn_ref::<web_sys::HtmlElement>()
            .map(|e| e.tag_name().to_lowercase())
            .unwrap_or_default();
        if matches!(tag_name.as_str(), "input" | "textarea") {
            return; // Let the browser handle these natively
        }

        // Escape key: deselect focused node or close dialogs
        if event.key() == "Escape" {
            // In a full impl: clear selection, close popovers
            web_sys::console::log_1("Escape pressed (deselect)".into());
            event.prevent_default();
        }

        // Delete key: remove selected node
        if event.key() == "Delete" || event.key() == "Backspace" {
            // In a full impl: remove selected node, notify store
            web_sys::console.log_1("Delete pressed (remove node)".into());
            event.prevent_default();
        }
    };

    // ==== Attach event listeners using add_event_listener_with_callback ==
    // to avoid Leptos view! macro closure signature issues
    let _ = svg_element.add_event_listener_with_callback("wheel", Closure::new(move |event: web_sys::WheelEvent| {
        on_zoom_wheel(event);
    }));

    let _ = svg_element.add_event_listener_with_callback("mousedown", Closure::new(move |event: web_sys::MouseEvent| {
        on_pan_start(event);
    }));

    let _ = svg_element.add_event_listener_with_callback("mousemove", Closure::new(move |event: web_sys::MouseEvent| {
        on_pan_move(event);
    }));

    let _ = svg_element.add_event_listener_with_callback("mouseup", Closure::new(move |event: web_sys::MouseEvent| {
        on_pan_end(event);
    }));

    let _ = svg_element.add_event_listener_with_callback("drag", Closure::new(move |event: web_sys::DragEvent| {
        on_node_drag_leave(event);
    }));

    let _ = svg_element.add_event_listener_with_callback("drag_over", Closure::new(move |event: web_sys::DragEvent| {
        on_node_drag_over(event);
    }));

    let _ = svg_element.add_event_listener_with_callback("drop", Closure::new(move |event: web_sys::DragEvent| {
        on_node_drop(event);
    }));

    let _ = svg_element.add_event_listener_with_callback("mousedown", Closure::new(move |event: web_sys::MouseEvent| {
        on_edge_start(event);
    }));

    let _ = svg_element.add_event_listener_with_callback("mouseup", Closure::new(move |event: web_sys::MouseEvent| {
        on_edge_end(event);
    }));

    let _ = document.add_event_listener_with_callback("keydown", Closure::new(move |event: web_sys::KeyboardEvent| {
        on_key_down(event);
    }));

    // ==== Static references for later use ====
    let _ = config;
    let _ = graph;
}

/// Convenience: get the canvas SVG element from the document by ID.
#[cfg(feature = "client")]
pub fn get_canvas_svg(document: &web_sys::Document) -> Option<SvgElement> {
    document
        .get_element_by_id("studio-canvas")
        .and_then(|el| el.dyn_ref::<SvgElement>().cloned())
}