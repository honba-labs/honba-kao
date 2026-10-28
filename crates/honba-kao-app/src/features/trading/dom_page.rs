use honba_kao_ui_core::organisms::{BookLevel, HfMarketTopBar, HfOrderBook, HfOrderTicket, HfTradingDock, OrderIntent};
use honba_kao_ui_core::templates::HfThreeColumnTradingLayout;
use leptos::prelude::*;
use rust_decimal_macros::dec;
use std::sync::Arc;

#[component]
pub fn DomPage() -> impl IntoView {
    let symbol = RwSignal::new("NIFTY50".to_string());
    let ltp = RwSignal::new(dec!(24500.0));

    let bids = RwSignal::new(vec![
        BookLevel {
            price: dec!(24498.0),
            size: dec!(150.0),
            cumulative_size: dec!(150.0),
            order_count: 5,
        },
        BookLevel {
            price: dec!(24495.0),
            size: dec!(300.0),
            cumulative_size: dec!(450.0),
            order_count: 8,
        },
    ]);
    let asks = RwSignal::new(vec![
        BookLevel {
            price: dec!(24502.0),
            size: dec!(120.0),
            cumulative_size: dec!(120.0),
            order_count: 4,
        },
        BookLevel {
            price: dec!(24505.0),
            size: dec!(280.0),
            cumulative_size: dec!(400.0),
            order_count: 7,
        },
    ]);
    let spread = RwSignal::new(Some((dec!(4.0), dec!(1.63))));

    let top_bar = view! {
        <HfMarketTopBar
            symbol=Signal::derive(move || symbol.get())
            ltp=Signal::derive(move || ltp.get())
            change_abs=Signal::derive(|| dec!(0.0))
            change_pct=Signal::derive(|| dec!(0.0))
            high_24h=Signal::derive(|| dec!(0.0))
            low_24h=Signal::derive(|| dec!(0.0))
            volume_24h=Signal::derive(|| dec!(0.0))
        />
    }
    .into_any();

    let visualizer = view! {
        <div class="h-full flex flex-col">
            <div class="p-2 border-b border-border-soft text-xs font-mono font-bold text-text-high">
                "Depth of Market (DOM) — Level 2 / Level 3 order book ladder"
            </div>
            <div class="flex-grow overflow-y-auto p-2">
                <span class="text-xs text-text-muted">"Chart visualizer placeholder — mount OpenalgoChart here."</span>
            </div>
        </div>
    }
    .into_any();

    let market_depth = view! {
        <HfOrderBook
            bids=Signal::derive(move || bids.get())
            asks=Signal::derive(move || asks.get())
            spread=Signal::derive(move || spread.get())
            on_level_click=Callback::new(move |(px, _sz, _is_bid)| {
                ltp.set(px);
            })
        />
    }
    .into_any();

    let order_ticket = view! {
        <HfOrderTicket
            symbol=Signal::derive(move || symbol.get())
            mark_price=Signal::derive(move || ltp.get())
            best_bid=Signal::derive(|| Some(dec!(24498.0)))
            best_ask=Signal::derive(|| Some(dec!(24502.0)))
            available_capital=Signal::derive(|| dec!(500000.0))
            margin_requirement_ratio=Signal::derive(|| dec!(0.15))
            on_submit_order=Callback::new(move |_intent: OrderIntent| {})
        />
    }
    .into_any();

    let dock = view! {
        <HfTradingDock
            positions_view=Arc::new(|| view! { <div class="text-xs text-text-muted p-3">"No open positions"</div> }.into_any())
            positions_count=Signal::derive(|| 0)
            open_orders_view=Arc::new(|| view! { <div class="text-xs text-text-muted p-3">"No active working orders"</div> }.into_any())
            open_orders_count=Signal::derive(|| 0)
            order_history_view=Arc::new(|| view! { <div class="text-xs text-text-muted p-3">"Order history clear"</div> }.into_any())
            fills_view=Arc::new(|| view! { <div class="text-xs text-text-muted p-3">"No trade executions today"</div> }.into_any())
        />
    }
    .into_any();

    view! {
        <HfThreeColumnTradingLayout
            top_bar=top_bar
            visualizer=visualizer
            market_depth=market_depth
            order_ticket=order_ticket
            dock=dock
        />
    }
}
