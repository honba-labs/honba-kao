//! Price chart panel with indicator readouts.

use leptos::prelude::*;

use crate::components::terminal::Candles;
use crate::components::ui::{HfPanel, HfPanelHeader};
use crate::data::demo_data::demo_price_series;
use crate::utils::{ema, rsi, sma};

fn readout(name: &str, value: Option<f64>) -> impl IntoView + use<> {
    let text = match value {
        Some(v) => format!("{:.2}", v),
        None => "—".to_string(),
    };
    view! {
        <span class="indicator">
            <span class="indicator__name">{name.to_string()}</span>
            <span class="indicator__value tnum">{text}</span>
        </span>
    }
}

#[component]
pub fn KlineChart() -> impl IntoView {
    let series = demo_price_series(2884.0, 120);
    let sma20 = sma(&series, 20);
    let ema50 = ema(&series, 50);
    let rsi14 = rsi(&series, 14);

    view! {
        <HfPanel>
            <HfPanelHeader title="RELIANCE.NSE · 5m">
                <div class="indicator-row">
                    {readout("SMA 20", sma20)}
                    {readout("EMA 50", ema50)}
                    {readout("RSI 14", rsi14)}
                </div>
            </HfPanelHeader>

            <div class="hfui-panel__content chart">
                <Candles count=48 />
            </div>
        </HfPanel>
    }
}
