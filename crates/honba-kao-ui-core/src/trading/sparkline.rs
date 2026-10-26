//! Mini sparkline: a single polyline scaled into a fixed box.
//!
//! OpenTerminalUI's `common/SparkLine.tsx` is a scaffold; this is the real
//! widget it stands in for. Values are display-only `f64`s (the zero-floats
//! policy covers domain money paths, not chart geometry).

use leptos::prelude::*;

/// Builds the SVG `points` attribute for `values` inside a `width` x `height`
/// box with a 1px inset. Empty input yields an empty string; a flat series
/// draws a horizontal line in the middle.
pub fn sparkline_points(values: &[f64], width: f64, height: f64) -> String {
    if values.is_empty() {
        return String::new();
    }
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = max - min;
    let n = values.len();
    let step = if n > 1 { (width - 2.0) / (n as f64 - 1.0) } else { 0.0 };
    values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = 1.0 + step * i as f64;
            let y = if span.abs() < f64::EPSILON {
                height / 2.0
            } else {
                1.0 + (height - 2.0) * (1.0 - (v - min) / span)
            };
            format!("{x:.2},{y:.2}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[component]
pub fn HfMiniSparkline(
    values: Vec<f64>,
    #[prop(default = 80.0)] width: f64,
    #[prop(default = 20.0)] height: f64,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let points = sparkline_points(&values, width, height);
    let rising = values.len() >= 2 && values.last() >= values.first();
    let tone = if rising { "hf-sparkline--up" } else { "hf-sparkline--down" };
    view! {
        <svg
            class=format!("hf-sparkline {tone} {class}").trim().to_string()
            viewBox=format!("0 0 {width} {height}")
            width=width
            height=height
            role="img"
            aria-label="price sparkline"
        >
            <polyline points=points fill="none" stroke="currentColor" stroke-width="1.5" />
        </svg>
    }
}
