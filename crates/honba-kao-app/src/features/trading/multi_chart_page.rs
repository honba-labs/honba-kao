use honba_kao_ui_core::organisms::{BookLevel, HfMarketTopBar, HfOrderBook, HfOrderTicket, HfTradingDock, OrderIntent};
use honba_kao_ui_core::templates::HfThreeColumnTradingLayout;
use leptos::prelude::*;
use openalgo_charts::OpenalgoChart;
use rust_decimal_macros::dec;
use std::sync::Arc;

#[component]
pub fn MultiChartPage() -> impl IntoView {
    let symbol = RwSignal::new("NIFTY50".to_string());
    let ltp = RwSignal::new(dec!(24500.0));
    let change_abs = RwSignal::new(dec!(125.5));
    let change_pct = RwSignal::new(dec!(0.52));

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
            change_abs=Signal::derive(move || change_abs.get())
            change_pct=Signal::derive(move || change_pct.get())
            high_24h=Signal::derive(|| dec!(24550.0))
            low_24h=Signal::derive(|| dec!(24380.0))
            volume_24h=Signal::derive(|| dec!(15200000.0))
        />
    }
    .into_any();

    let visualizer = view! {
        <div class="grid grid-cols-1 laptop:grid-cols-2 gap-3 h-full">
            <div class="rounded border border-border-soft bg-surface-base p-2 flex flex-col">
                <span class="text-xs font-mono font-bold text-text-high mb-1">"NIFTY50 (1m)"</span>
                <OpenalgoChart data=Signal::derive(Vec::new) />
            </div>
            <div class="rounded border border-border-soft bg-surface-base p-2 flex flex-col">
                <span class="text-xs font-mono font-bold text-text-high mb-1">"BANKNIFTY (5m)"</span>
                <OpenalgoChart data=Signal::derive(Vec::new) />
            </div>
            <div class="rounded border border-border-soft bg-surface-base p-2 flex flex-col">
                <span class="text-xs font-mono font-bold text-text-high mb-1">"FINNIFTY (15m)"</span>
                <OpenalgoChart data=Signal::derive(Vec::new) />
            </div>
            <div class="rounded border border-border-soft bg-surface-base p-2 flex flex-col">
                <span class="text-xs font-mono font-bold text-text-high mb-1">"MIDCPNIFTY (1h)"</span>
                <OpenalgoChart data=Signal::derive(Vec::new) />
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
