//! HfTradingDrawer — tabs: Holdings/Positions/Orders/Trades/Analytics/Depth.
//! Mirrors bfx-hf-ui-core trading drawer at the bottom of the workbench.

use leptos::prelude::*;
use crate::HfTabs;

#[component]
pub fn HfTradingDrawer(
    /// Initially active tab.
    #[prop(optional)]
    default_tab: Option<&'static str>,
    #[prop(optional, into)]
    class: String,
    children: Children,
) -> impl IntoView {
    let default_tab = default_tab.unwrap_or("positions");
    let full_class = format!("hf-trading-drawer {class}").trim().to_string();

    view! {
        <div class=full_class>
            <div class="hf-trading-drawer__tabs">
                <HfTabs default_value=default_tab>
                    <div class="hf-tabs__list">
                        <button class="hf-tabs__trigger hf-tabs__trigger--active" data-value="holdings">"Holdings"</button>
                        <button class="hf-tabs__trigger" data-value="positions">"Positions"</button>
                        <button class="hf-tabs__trigger" data-value="orders">"Orders"</button>
                        <button class="hf-tabs__trigger" data-value="trades">"Trades"</button>
                        <button class="hf-tabs__trigger" data-value="analytics">"Analytics"</button>
                        <button class="hf-tabs__trigger" data-value="depth">"Market Depth"</button>
                    </div>

                    <div class="hf-tabs__content" data-value="holdings">{children()}</div>
                    <div class="hf-tabs__content hf-tabs__content--hidden" data-value="positions"></div>
                    <div class="hf-tabs__content hf-tabs__content--hidden" data-value="orders"></div>
                    <div class="hf-tabs__content hf-tabs__content--hidden" data-value="trades"></div>
                    <div class="hf-tabs__content hf-tabs__content--hidden" data-value="analytics"></div>
                    <div class="hf-tabs__content hf-tabs__content--hidden" data-value="depth"></div>
                </HfTabs>
            </div>
        </div>
    }
}