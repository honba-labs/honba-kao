//! Canvas geometry (HBD-2026-DES-006 P2) — pure, no DOM.
//!
//! Everything the editor needs to place pixels: the 16px snap grid, the
//! node-card box model and port anchors, fe's smoothstep edge path (the
//! `getSmoothStepPath(borderRadius: 8)` shape xyflow drew), the layered
//! auto-layout used when a graph arrives without positions, and the
//! pan/zoom [`Viewport`] (a plain SVG `translate(..) scale(..)` in user
//! units — no device-pixel-ratio handling, per the design doc).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::features::simulator::kernel::selection::StrategyGraphShape;
use crate::features::strategy_studio::kernel::graph::{node_inputs, node_outputs};

/// Snap-to-grid granularity, in canvas user units (= CSS px at zoom 1).
pub const GRID: f64 = 16.0;
/// Node card width (the design doc's 200–300px band; a grid multiple).
pub const NODE_WIDTH: f64 = 224.0;
/// Card header: title + subtitle lines.
pub const HEADER_HEIGHT: f64 = 40.0;
/// The per-kind parameter chip row.
pub const CHIPS_HEIGHT: f64 = 22.0;
/// One named port row.
pub const PORT_ROW_HEIGHT: f64 = 20.0;
/// Bottom padding under the last port row.
pub const CARD_PADDING: f64 = 8.0;
/// Horizontal gap between auto-layout columns.
pub const COLUMN_GAP: f64 = 80.0;
/// Vertical gap between stacked auto-layout cards.
pub const ROW_GAP: f64 = 32.0;
/// Auto-layout origin margin.
pub const LAYOUT_MARGIN: f64 = 48.0;
/// Zoom clamp.
pub const ZOOM_MIN: f64 = 0.25;
/// Zoom clamp.
pub const ZOOM_MAX: f64 = 2.5;

/// A node's top-left corner on the canvas, in user units.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
}

impl Position {
    /// A position.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Rounds one coordinate to the 16px grid.
pub fn snap(value: f64) -> f64 {
    (value / GRID).round() * GRID
}

/// Rounds a position to the 16px grid.
pub fn snap_position(position: Position) -> Position {
    Position::new(snap(position.x), snap(position.y))
}

/// Card height for a node with `inputs`/`outputs` port rows.
pub fn node_height(inputs: usize, outputs: usize) -> f64 {
    let rows = inputs.max(outputs).max(1) as f64;
    HEADER_HEIGHT + CHIPS_HEIGHT + rows * PORT_ROW_HEIGHT + CARD_PADDING
}

/// The vertical centre of port row `index`, relative to the card top.
pub fn port_row_y(index: usize) -> f64 {
    HEADER_HEIGHT + CHIPS_HEIGHT + index as f64 * PORT_ROW_HEIGHT + PORT_ROW_HEIGHT / 2.0
}

/// Absolute anchor of input port `index` (left card edge).
pub fn input_anchor(position: Position, index: usize) -> (f64, f64) {
    (position.x, position.y + port_row_y(index))
}

/// Absolute anchor of output port `index` (right card edge).
pub fn output_anchor(position: Position, index: usize) -> (f64, f64) {
    (position.x + NODE_WIDTH, position.y + port_row_y(index))
}

/// Stub length an edge leaves a port before turning (xyflow's `offset`).
const STEP_OFFSET: f64 = 20.0;
/// Corner radius (fe passed `borderRadius: 8`).
const CORNER_RADIUS: f64 = 8.0;

/// fe's smoothstep edge between an output anchor and an input anchor:
/// the SVG path `d` plus the label point the "+" splice control sits on.
pub fn smoothstep_path(source: (f64, f64), target: (f64, f64)) -> (String, f64, f64) {
    let (sx, sy) = source;
    let (tx, ty) = target;
    if (ty - sy).abs() < 0.5 && tx >= sx {
        return (format!("M {sx:.1},{sy:.1} L {tx:.1},{ty:.1}"), (sx + tx) / 2.0, sy);
    }
    if tx >= sx + 2.0 * STEP_OFFSET {
        // Forward: out, one vertical run at the horizontal midpoint, in.
        let mid_x = (sx + tx) / 2.0;
        let points = [(sx, sy), (mid_x, sy), (mid_x, ty), (tx, ty)];
        (rounded_polyline(&points, CORNER_RADIUS), mid_x, (sy + ty) / 2.0)
    } else {
        // Backward: step out right, cross at the vertical midpoint, step in
        // from the left — never through either card.
        let mid_y = (sy + ty) / 2.0;
        let out_x = sx + STEP_OFFSET;
        let in_x = tx - STEP_OFFSET;
        let points = [(sx, sy), (out_x, sy), (out_x, mid_y), (in_x, mid_y), (in_x, ty), (tx, ty)];
        (rounded_polyline(&points, CORNER_RADIUS), (out_x + in_x) / 2.0, mid_y)
    }
}

/// An orthogonal polyline with quadratic corners of radius `radius`,
/// clipped to half of each adjacent segment so short runs never overshoot.
pub fn rounded_polyline(points: &[(f64, f64)], radius: f64) -> String {
    let Some(first) = points.first() else {
        return String::new();
    };
    let mut d = format!("M {:.1},{:.1}", first.0, first.1);
    for i in 1..points.len() {
        let (px, py) = points[i - 1];
        let (cx, cy) = points[i];
        if i + 1 == points.len() {
            d.push_str(&format!(" L {cx:.1},{cy:.1}"));
            break;
        }
        let (nx, ny) = points[i + 1];
        let len_in = ((cx - px).powi(2) + (cy - py).powi(2)).sqrt();
        let len_out = ((nx - cx).powi(2) + (ny - cy).powi(2)).sqrt();
        let r = radius.min(len_in / 2.0).min(len_out / 2.0);
        if r <= 0.01 {
            d.push_str(&format!(" L {cx:.1},{cy:.1}"));
            continue;
        }
        let (ux, uy) = ((cx - px) / len_in, (cy - py) / len_in);
        let (vx, vy) = ((nx - cx) / len_out, (ny - cy) / len_out);
        d.push_str(&format!(
            " L {:.1},{:.1} Q {cx:.1},{cy:.1} {:.1},{:.1}",
            cx - ux * r,
            cy - uy * r,
            cx + vx * r,
            cy + vy * r
        ));
    }
    d
}

/// Layered auto-layout: column = longest path from a source node, rows
/// stacked in graph order, everything on the 16px grid. Used when a graph
/// arrives without positions (templates, library opens). Cycles (only
/// possible in an imported, invalid graph) are cut, never looped on.
pub fn auto_layout(graph: &StrategyGraphShape) -> BTreeMap<String, Position> {
    let mut depth: BTreeMap<&str, usize> = graph.nodes.iter().map(|n| (n.id.as_str(), 0)).collect();
    // Bellman-Ford-style relaxation bounded by the node count: a DAG
    // settles within n rounds; a cycle simply stops relaxing at the bound.
    for _ in 0..graph.nodes.len() {
        let mut changed = false;
        for edge in &graph.edges {
            let (Some(&from), Some(&to)) =
                (depth.get(edge.from_node.as_str()), depth.get(edge.to_node.as_str()))
            else {
                continue;
            };
            if to <= from && from < graph.nodes.len() {
                depth.insert(edge.to_node.as_str(), from + 1);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut column_bottom: BTreeMap<usize, f64> = BTreeMap::new();
    let mut positions = BTreeMap::new();
    for node in &graph.nodes {
        let column = depth.get(node.id.as_str()).copied().unwrap_or(0);
        let x = LAYOUT_MARGIN + column as f64 * (NODE_WIDTH + COLUMN_GAP);
        let y = *column_bottom.get(&column).unwrap_or(&LAYOUT_MARGIN);
        let height = node_height(
            node_inputs(&node.node_type).len(),
            node_outputs(&node.node_type, Some(node)).len(),
        );
        column_bottom.insert(column, snap(y + height + ROW_GAP));
        positions.insert(node.id.clone(), snap_position(Position::new(x, y)));
    }
    positions
}

/// The pan/zoom state: screen = world × zoom + (x, y).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    /// Horizontal pan, in screen px.
    pub x: f64,
    /// Vertical pan, in screen px.
    pub y: f64,
    /// Scale factor.
    pub zoom: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, zoom: 1.0 }
    }
}

impl Viewport {
    /// Screen (canvas-local px) → world (user units).
    pub fn to_world(self, local_x: f64, local_y: f64) -> (f64, f64) {
        ((local_x - self.x) / self.zoom, (local_y - self.y) / self.zoom)
    }

    /// World → screen (canvas-local px).
    pub fn to_screen(self, world_x: f64, world_y: f64) -> (f64, f64) {
        (world_x * self.zoom + self.x, world_y * self.zoom + self.y)
    }

    /// Zooms by `factor` keeping the screen point `(local_x, local_y)`
    /// fixed under the cursor; the zoom is clamped to
    /// [`ZOOM_MIN`]..=[`ZOOM_MAX`].
    pub fn zoom_at(self, local_x: f64, local_y: f64, factor: f64) -> Viewport {
        let zoom = (self.zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX);
        let (wx, wy) = self.to_world(local_x, local_y);
        Viewport { x: local_x - wx * zoom, y: local_y - wy * zoom, zoom }
    }

    /// The `transform` attribute of the world group.
    pub fn transform(self) -> String {
        format!("translate({:.2},{:.2}) scale({:.4})", self.x, self.y, self.zoom)
    }

    /// A viewport that fits every card into a `width × height` canvas with
    /// a margin, never zooming in past 1:1.
    pub fn fit(
        positions: &BTreeMap<String, Position>,
        heights: &BTreeMap<String, f64>,
        width: f64,
        height: f64,
    ) -> Viewport {
        if positions.is_empty() || width <= 0.0 || height <= 0.0 {
            return Viewport::default();
        }
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for (id, p) in positions {
            let h = heights.get(id).copied().unwrap_or(HEADER_HEIGHT);
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x + NODE_WIDTH);
            max_y = max_y.max(p.y + h);
        }
        let margin = 32.0;
        let zoom = ((width - 2.0 * margin) / (max_x - min_x))
            .min((height - 2.0 * margin) / (max_y - min_y))
            .clamp(ZOOM_MIN, 1.0);
        let content_w = (max_x - min_x) * zoom;
        let content_h = (max_y - min_y) * zoom;
        Viewport {
            x: (width - content_w) / 2.0 - min_x * zoom,
            y: (height - content_h) / 2.0 - min_y * zoom,
            zoom,
        }
    }
}
