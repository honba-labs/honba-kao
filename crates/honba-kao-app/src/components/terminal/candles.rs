//! Candle chart rendered as inline SVG.
//!
//! The reference app vendors TradingView's charting library; honba-kao keeps
//! the chart Rust-native (no JS charting dependency) — a plain SVG built from
//! the price series, which is enough for the scaffold and keeps the bundle
//! small.

use leptos::prelude::*;

use crate::data::demo_data::demo_price_series;

#[component]
pub fn Candles(#[prop(default = 48usize)] count: usize) -> impl IntoView {
    let series = demo_price_series(2884.0, count);
    let (min, max) = series
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), v| (lo.min(*v), hi.max(*v)));
    let span = (max - min).max(1e-9);

    view! {
        <svg
            class="candles"
            viewBox=format!("0 0 {} 100", count * 4)
            preserveAspectRatio="none"
            role="img"
            aria-label="Price candles"
        >
            {series
                .iter()
                .enumerate()
                .map(|(i, value)| {
                    let h = ((value - min) / span) * 90.0 + 5.0;
                    let up = i == 0 || *value >= series[i - 1];
                    let color = if up { "var(--pnl-gain)" } else { "var(--pnl-loss)" };
                    view! {
                        <rect
                            x=(i * 4 + 1) as f64
                            y=100.0 - h
                            width=2.0
                            height=h
                            fill=color
                            rx=0.6
                        />
                    }
                })
                .collect_view()}
        </svg>
    }
}
