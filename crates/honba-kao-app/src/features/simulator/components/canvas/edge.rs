//! Edge (connection between two nodes).
//! Smoothstep path; insertable edge: midpoint "+" → filter menu;
//! splice = replace edge with `a → new → b` positioned midway.

use leptos::prelude::*;

/// Port kind determines color dot category.
fn port_kind_to_category(kind: &str) -> &'static str {
    match kind {
        "series" => "Indicator",
        "bool" => "Condition",
        "size" => "Sizing",
        "order" => "Order",
        _ => "Condition",
    }
}

/// Smoothstep SVG path between two points.
fn smoothstep_path(x1: f64, y1: f64, x2: f64, y2: f64, tension: f64) -> String {
    // Simplified smoothstep using cubic Bezier
    let tension = tension.max(0.0).min(1.0);
    let cp1x = x1 + (x2 - x1) * (1.0 - tension) * 0.5;
    let cp1y = y1 + (y2 - y1) * (1.0 - tension) * 0.5;
    let cp2x = x2 - (x2 - x1) * (1.0 - tension) * 0.5;
    let cp2y = y2 - (y2 - y1) * (1.0 - tension) * 0.5;
    format!("M {x1} {y1} C {cp1x} {cp1y} {cp2x} {cp2y} {x2} {y2}")
}

/// Insertable edge: midpoint "+" button → filter menu.
/// Clicking "+" opens a small modal with port-kind checkboxes;
/// selecting one inserts a new edge `a → new → b` positioned midway.
fn insertable_edge_midpoint(x1: f64, y1: f64, x2: f64, y2: f64) -> IntoView {
    let midpoint_x = (x1 + x2) / 2.0;
    let midpoint_y = (y1 + y2) / 2.0;
    view! {
        <svg width="20" height="20" viewBox="0 0 24 24" class="edge-insert-btn">
          <circle cx="12" cy="12" r="8" fill="none" stroke="var(--c-indicator)" stroke-width="2"/>
          <line x1="8" y1="12" x2="16" y2="12" stroke="var(--c-indicator)" stroke-width="2"/>
          <line x1="12" y1="8" x2="12" y2="16" stroke="var(--c-indicator)" stroke-width="2"/>
        </svg>
    }
}
/// Splice: replace edge with `a → new → b` positioned midway.
fn splice_edges(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    let mx = (x1 + x2) / 2.0;
    let my = (y1 + y2) / 2.0;
    // Render two segments: a→mid and mid→b
    format!("M {x1} {y1} L {mx} {my} L {x2} {y2}")
}
