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

                // Optional Bulk Cancel Action
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

            <div class="hf-dock-content flex-grow overflow-y-auto p-2">
                {move || match active_tab.get().as_str() {
                    "positions" => positions_view(),
                    "orders" => open_orders_view(),
                    "history" => order_history_view(),
                    _ => fills_view(),
                }}
            </div>
        </div>
    }
}
