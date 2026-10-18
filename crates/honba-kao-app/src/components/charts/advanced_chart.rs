//! Full chart workspace with timeframe tabs. Mirrors
//! `web/src/components/charts/AdvancedChart.tsx`.

use leptos::prelude::*;

use crate::components::terminal::Candles;
use crate::components::ui::{HfPanel, HfPanelHeader};

const TIMEFRAMES: &[&str] = &["1m", "5m", "15m", "1h", "1D"];

#[component]
pub fn AdvancedChart() -> impl IntoView {
    let active = RwSignal::new("5m");

    view! {
        <HfPanel>
            <HfPanelHeader title="RELIANCE.NSE">
                <div class="tabs">
                    {TIMEFRAMES
                        .iter()
                        .map(|tf| {
                            let tf = *tf;
                            view! {
                                <button
                                    class=move || {
                                        if active.get() == tf { "tab tab--active" } else { "tab" }
                                    }
                                    on:click=move |_| active.set(tf)
                                >
                                    {tf}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </HfPanelHeader>

            <div class="hfui-panel__content chart chart--tall">
                <Candles count=72 />
            </div>
        </HfPanel>
    }
}
