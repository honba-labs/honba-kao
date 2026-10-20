//! HfChartToolbar — symbol, timeframe, chart type, indicators, snapshot.
//! Mirrors bfx-hf-ui-core chart toolbar used in workbench.

use leptos::prelude::*;
use crate::{HfButton, HfIconButton, HfSelect, HfSelectItem, HfButtonVariant, HfButtonSize};
use crate::icons::outline::candlestick;

#[component]
pub fn HfChartToolbar(
    /// Current symbol.
    #[prop(optional, into)]
    symbol: Option<String>,
    /// Current timeframe.
    #[prop(optional, into)]
    timeframe: Option<String>,
    /// Whether to show chart type selector.
    #[prop(optional)]
    show_chart_type: Option<bool>,
    /// Whether to show indicators button.
    #[prop(optional)]
    show_indicators: Option<bool>,
    /// Whether to show snapshot button.
    #[prop(optional)]
    show_snapshot: Option<bool>,
    children: Children,
) -> impl IntoView {
    let show_chart_type = show_chart_type.unwrap_or(true);
    let show_indicators = show_indicators.unwrap_or(true);
    let show_snapshot = show_snapshot.unwrap_or(true);

    view! {
        <div class="hf-chart-toolbar">
            <div class="hf-chart-toolbar__symbol">
                <HfButton variant=HfButtonVariant::Ghost size=HfButtonSize::Small>
                    {move || symbol.as_deref().unwrap_or("RELIANCE").to_string()}
                    <svg
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        fill="none"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        class="hf-chart-toolbar__caret"
                        width="12"
                        height="12"
                    >
                        <polyline points="6 9 12 15 18 9" />
                    </svg>
                </HfButton>
            </div>

            <div class="hf-chart-toolbar__timeframe">
                <HfButton variant=HfButtonVariant::Ghost size=HfButtonSize::Small>
                    {move || timeframe.as_deref().unwrap_or("1d").to_string()}
                    <svg
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        fill="none"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        class="hf-chart-toolbar__caret"
                        width="12"
                        height="12"
                    >
                        <polyline points="6 9 12 15 18 9" />
                    </svg>
                </HfButton>
            </div>

            {move || if show_chart_type {
                view! {
                    <div class="hf-chart-toolbar__chart-type">
                        <HfSelect>
                            <HfSelectItem value="candlestick" selected=true>"Candles"</HfSelectItem>
                            <HfSelectItem value="line">"Line"</HfSelectItem>
                            <HfSelectItem value="area">"Area"</HfSelectItem>
                        </HfSelect>
                    </div>
                }.into_any()
            } else { ().into_any() }}

            {move || if show_indicators {
                view! { <HfIconButton icon=candlestick() size=16 /> }.into_any()
            } else { ().into_any() }}

            {move || if show_snapshot {
                view! { <HfIconButton icon=camera() size=16 /> }.into_any()
            } else { ().into_any() }}

            {children()}
        </div>
    }
}

/// Camera icon for snapshot.
pub fn camera() -> &'static str { "M23 19a2 2 0 01-2 2H3a2 2 0 01-2-2V8a2 2 0 012-2h4l2-3h6l2 3h4a2 2 0 012 2z" }