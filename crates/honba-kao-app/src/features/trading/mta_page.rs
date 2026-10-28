use honba_kao_ui_core::organisms::{BookLevel, HfMarketTopBar, HfOrderBook, HfOrderTicket, HfTradingDock, OrderIntent};
use honba_kao_ui_core::templates::HfThreeColumnTradingLayout;
use leptos::prelude::*;
use std::sync::Arc;

#[component]
pub fn MtaPage() -> impl IntoView {
    let symbol = RwSignal::new("NIFTY50".to_string());
    let ltp = RwSignal::new(24500.0);

    let bids = RwSignal::new(vec![
        BookLevel {
            price: 24498.0,
            size: 150.0,
            cumulative_size: 150.0,
            order_count: 5,
        },
        BookLevel {
            price: 24495.0,
            size: 300.0,
            cumulative_size: 450.0,
            order_count: 8,
        },
    ]);
    let asks = RwSignal::new(vec![
        BookLevel {
            price: 24502.0,
            size: 120.0,
            cumulative_size: 120.0,
            order_count: 4,
        },
        BookLevel {
            price: 24505.0,
            size: 280.0,
            cumulative_size: 400.0,
            order_count: 7,
        },
    ]);
    let spread = RwSignal::new(Some((4.0, 1.63)));

    let top_bar = view! {
        <HfMarketTopBar
            symbol=Signal::derive(move || symbol.get())
            ltp=Signal::derive(move || ltp.get())
            change_abs=Signal::derive(|| 0.0)
            change_pct=Signal::derive(|| 0.0)
            high_24h=Signal::derive(|| 0.0)
            low_24h=Signal::derive(|| 0.0)
            volume_24h=Signal::derive(|| 0.0)
        />
    }
    .into_any();

    let visualizer = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
            "Multi-timeframe trend alignment, VWAP bands, and volume profiles."
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
            best_bid=Signal::derive(|| Some(24498.0))
            best_ask=Signal::derive(|| Some(24502.0))
            available_capital=Signal::derive(|| 500_000.0)
            margin_requirement_ratio=Signal::derive(|| 0.15)
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
