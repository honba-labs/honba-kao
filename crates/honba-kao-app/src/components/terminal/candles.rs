//! Price candles on the openalgo-charts engine.
//!
//! The reference app vendors TradingView's charting library; honba-kao uses
//! [openalgo-charts](https://github.com/marketcalls/openalgo-charts) instead —
//! dependency-free, Apache-2.0, and it reads its palette from the app skin so
//! candles follow the active theme.

use leptos::prelude::*;

use crate::components::charts::{ChartSeries, OpenAlgoChart};
use crate::data::demo_data::demo_ohlc_bars;

#[component]
pub fn Candles(
    #[prop(default = 48usize)] count: usize,
    #[prop(default = 300i64)] interval_seconds: i64,
) -> impl IntoView {
    let bars = demo_ohlc_bars(2884.0, count, interval_seconds);

    view! {
        <OpenAlgoChart
            series=vec![ChartSeries::candles(bars)]
            class="openalgo-chart"
            label="Price candles"
        />
    }
}
