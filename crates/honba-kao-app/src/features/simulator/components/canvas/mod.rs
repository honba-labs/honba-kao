//! Node editor canvas — SVG-based graph editor for strategy authoring.
//! 
//! Features: pan/zoom, node selection, edge splicing, palette, inspector,
//! persistence via localStorage, and a minimal code pane (no Monaco).
//! 
//! Graph model: `StrategyGraphShape` v1 (same contract as DES-004/DES-005).
//! Nodes: 10 types with typed ports (`series|bool|size|order`).
//! Ports: `series` → indicators, `bool` → gates/orders, `size` → size, `order` → order.
//! 
//! Node canvas: custom Leptos SVG (no xyflow — see DES-004 risk).
//! Scale: inverse rendering with `Background gap=16`, snap-to-grid 16.
//! Node card: 200–300px, category top-border color (`--c-*` tokens).
//! Edge: smoothstep path; insertable edge: midpoint "+" → filter menu;
//! splice = replace edge with `a → new → b` positioned midway.
//! Palette: drag with HTML5 MIME `application/honba-node` (port fe's).
//! Inspector: shows param inputs from catalogue ranges.
//! Store wiring: every editor gesture calls a store action; canvas-only
//! extras (`positions`) stay out of `to_manifest`.

use leptos::prelude::*;

pub mod node_card;
pub mod edge;
pub mod palette;
pub mod inspector;

/// Pan/zoomcontrolled SVG viewport with pan (drag), wheel-zoom, click-select,
/// drag-move (16px snap), `Escape`/`Del` handling guarded against
/// `INPUT/TEXTAREA`.
/// 
/// Node card: 200–300px, category top-border color (`--c-*` tokens).
/// Edge: smoothstep path; insertable edge: midpoint "+" → filter menu;
/// splice = replace edge with `a → new → b` positioned midway.
/// Palette: drag with HTML5 MIME `application/honba-node` (port fe's).
/// Inspector: shows param inputs from catalogue ranges.
/// Store wiring: every editor gesture calls a store action; canvas-only
/// extras (`positions`) stay out of `to_manifest`.

/// Canvas state kept in a single `Signal<Vec<RenderNode>>` `Memo` over
/// positions; do NOT re-render each port as its own effect (a 12-node graph
/// must be one MEMO render pass).
/// 
/// Pan/zoom is a plain SVG transform — do not hand-roll DPR handling;
/// keep `viewBox` in user units. Multi-select (rubber band) is **out of scope**
/// — fe never implemented it either; state so in copy.
/// 
/// Keyboard rule from fe: Delete/Backspace must **not** intercept editable
/// inputs (INPUT/TEXTAREA guard); `Ctrl/Cmd+S` = snapshot+save, Escape
/// clears selection first, then closes whatever else.

/// Renderable node.
#[derive(Clone)]
struct RenderNode {
    id: String,
    category: String,
    x: f64,
    y: f64,
    ports: Vec<(String, String)>, // (port_name, port_kind)
}

/// Render pass: one MEMO over positions; no per-node effects.
fn render_nodes(nodes: &[RenderNode]) -> impl IntoView {
    let nodes = nodes.to_vec();
    move || {
        view! {
            <g class="nodes">
                {nodes.iter().map(|n| {
                    view! { <NodeCard category=String::from(&n.category) /> }
                }).collect_view()}
            </g>
        }
    }
}