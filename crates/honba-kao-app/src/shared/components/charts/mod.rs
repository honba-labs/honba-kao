//! Chart components. Mirrors `web/src/components/charts/`.
//!
//! Price, equity and comparison charts render through the vendored
//! [openalgo-charts](https://github.com/marketcalls/openalgo-charts) engine via
//! [`openalgo_chart::OpenAlgoChart`]. Small inline sparklines stay Rust-native
//! SVG, where a canvas engine would cost more than it returns.

pub mod comparison_chart;
pub mod equity_chart;
pub mod openalgo_chart;

pub use comparison_chart::ComparisonChart;
pub use equity_chart::EquityChart;
pub use openalgo_chart::{ChartSeries, OpenAlgoChart};

/// Scale a series into an SVG `polyline`/`path` point string.
pub fn scale_points(values: &[f64], width: f64, height: f64) -> String {
    if values.is_empty() {
        return String::new();
    }
    let (min, max) = values
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), v| (lo.min(*v), hi.max(*v)));
    let span = (max - min).max(1e-9);
    let step = width / (values.len().max(2) - 1) as f64;

    values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = i as f64 * step;
            let y = height - ((v - min) / span) * height;
            format!("{:.2},{:.2}", x, y)
        })
        .collect::<Vec<_>>()
        .join(" ")
}
