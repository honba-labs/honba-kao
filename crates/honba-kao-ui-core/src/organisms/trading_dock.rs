use crate::primitive::{ButtonSize, ButtonVariant, HfButton};
use leptos::prelude::*;

#[component]
pub fn HfTradingDock(
    positions_view: ChildrenFn,
    #[prop(into)] positions_count: Signal<usize>,
    open_orders_view: ChildrenFn,
    #[prop(into)] open_orders_count: Signal<usize>,
    order_history_view: ChildrenFn,
    fills_view: ChildrenFn,
    #[prop(into, optional)] on_cancel_all_orders: Option<Callback<()>>,
    /// Aggregate unrealised PnL across open positions (issue #73).
    #[prop(into, optional)] positions_pnl: Option<Signal<f64>>,
) -> impl IntoView {
    let active_tab = RwSignal::new("positions".to_string());

    view! {
        <div class="hf-trading-dock flex flex-col h-full bg-surface-base border border-border-soft rounded overflow-hidden">
            <div class="hf-dock-tabbar flex items-center justify-between px-3 py-1.5 border-b border-border-soft bg-surface-subtle">
                <div class="flex items-center gap-1">
                    <HfButton
                        class=Signal::derive(move || format!("flex items-center gap-1.5 px-2.5 py-1 text-xs rounded transition-colors {}", if active_tab.get() == "positions" { "bg-surface-base text-text-high font-semibold shadow-sm" } else { "text-text-muted hover:text-text-high" }))
                        variant=ButtonVariant::Ghost
                        size=ButtonSize::Sm
                        on_click=Callback::new(move |_| active_tab.set("positions".to_string()))
                    >
                        <span>"Positions"</span>
                        <span class="px-1.5 py-0.2 rounded-full bg-accent-primary/20 text-accent-primary text-[10px] font-bold">
                            {move || positions_count.get()}
                        </span>
                    </HfButton>
                    <HfButton
                        class=Signal::derive(move || format!("flex items-center gap-1.5 px-2.5 py-1 text-xs rounded transition-colors {}", if active_tab.get() == "orders" { "bg-surface-base text-text-high font-semibold shadow-sm" } else { "text-text-muted hover:text-text-high" }))
                        variant=ButtonVariant::Ghost
                        size=ButtonSize::Sm
                        on_click=Callback::new(move |_| active_tab.set("orders".to_string()))
                    >
                        <span>"Open Orders"</span>
                        <span class="px-1.5 py-0.2 rounded-full bg-accent-primary/20 text-accent-primary text-[10px] font-bold">
                            {move || open_orders_count.get()}
                        </span>
                    </HfButton>
                    <HfButton
                        class=Signal::derive(move || format!("px-2.5 py-1 text-xs rounded transition-colors {}", if active_tab.get() == "history" { "bg-surface-base text-text-high font-semibold shadow-sm" } else { "text-text-muted hover:text-text-high" }))
                        variant=ButtonVariant::Ghost
                        size=ButtonSize::Sm
                        on_click=Callback::new(move |_| active_tab.set("history".to_string()))
                    >
                        "Order History"
                    </HfButton>
                    <HfButton
                        class=Signal::derive(move || format!("px-2.5 py-1 text-xs rounded transition-colors {}", if active_tab.get() == "fills" { "bg-surface-base text-text-high font-semibold shadow-sm" } else { "text-text-muted hover:text-text-high" }))
                        variant=ButtonVariant::Ghost
                        size=ButtonSize::Sm
                        on_click=Callback::new(move |_| active_tab.set("fills".to_string()))
                    >
                        "Trade Fills"
                    </HfButton>
                </div>

                <div class="flex items-center gap-2 ml-auto">
                    {move || {
                        if let Some(pnl) = positions_pnl.as_ref() {
                            let v = pnl.get();
                            let sign = if v >= 0.0 { "+" } else { "" };
                            let pnl_color = if v > 0.0 { "text-emerald-400" } else if v < 0.0 { "text-rose-400" } else { "text-text-muted" };
                            view! {
                                <span class=format!("px-2 py-0.5 rounded bg-surface-subtle text-[10px] font-mono font-bold {pnl_color}")>
                                    {move || format!("PnL {sign}₹{:.2}", v.abs())}
                                </span>
                            }.into_any()
                        } else {
                            ().into_any()
                        }
                    }}
                    {move || {
                        if active_tab.get() == "orders" && open_orders_count.get() > 0 {
                            if let Some(cancel_all) = on_cancel_all_orders {
                                view! {
                                    <HfButton
                                        class="text-xs px-2 py-0.5 rounded bg-rose-500/20 text-rose-400 hover:bg-rose-500/30 font-semibold"
                                        variant=ButtonVariant::Danger
                                        size=ButtonSize::Xxs
                                        on_click=Callback::new(move |_| cancel_all.run(()))
                                    >
                                        "Cancel All"
                                    </HfButton>
                                }.into_any()
                            } else {
                                ().into_any()
                            }
                        } else {
                            ().into_any()
                        }
                    }}
                </div>
            </div>

            // All four tab views stay mounted (toggled with `hidden`) so that
            // switching tabs never tears down and remounts DOM subtrees
            // (issue #73).
            <div class="hf-dock-content flex-grow overflow-y-auto p-2">
                <div class=move || format!("hf-dock-panel {}", if active_tab.get() == "positions" { "" } else { "hidden" })>
                    {positions_view()}
                </div>
                <div class=move || format!("hf-dock-panel {}", if active_tab.get() == "orders" { "" } else { "hidden" })>
                    {open_orders_view()}
                </div>
                <div class=move || format!("hf-dock-panel {}", if active_tab.get() == "history" { "" } else { "hidden" })>
                    {order_history_view()}
                </div>
                <div class=move || format!("hf-dock-panel {}", if active_tab.get() == "fills" { "" } else { "hidden" })>
                    {fills_view()}
                </div>
            </div>
        </div>
    }
}
