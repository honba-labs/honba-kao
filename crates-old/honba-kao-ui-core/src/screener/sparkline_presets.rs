//! Screener sparkline size presets — thin wrappers over
//! `crate::home::sparkline_cell::SparklineCell` for the two sizes called for
//! by the OpenTerminalUI screener spec: table cell (80x24) and drawer
//! (240x60, interactive). No sparkline SVG math is reimplemented here.

use leptos::prelude::*;

use crate::home::sparkline_cell::SparklineCell;

#[component]
pub fn TableSparkline(points: Vec<f64>, #[prop(optional, into)] class: String) -> impl IntoView {
    view! { <SparklineCell points=points width=80.0 height=24.0 class=class /> }
}

#[component]
pub fn DrawerSparkline(points: Vec<f64>, #[prop(optional, into)] class: String) -> impl IntoView {
    view! { <SparklineCell points=points width=240.0 height=60.0 show_tooltip=true class=class /> }
}
