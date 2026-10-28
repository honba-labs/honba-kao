use honba_kao_ui_core::organisms::{HfMarketTape, HfMarketTopBar, HfOrderTicket, HfTradingDock, OrderIntent, TapeTrade};
use honba_kao_ui_core::templates::HfThreeColumnTradingLayout;
use leptos::prelude::*;
use rust_decimal_macros::dec;
use std::sync::Arc;

#[component]
pub fn TapePage() -> impl IntoView {
    let symbol = RwSignal::new("NIFTY50".to_string());
    let ltp = RwSignal::new(dec!(24500.0));

    let trades = RwSignal::new(vec![
        TapeTrade {
            price: dec!(24500.5),
            size: dec!(50.0),
            time: "09:15:02".to_string(),
            is_buyer_maker: false,
        },
        TapeTrade {
            price: dec!(24500.0),
            size: dec!(100.0),
            time: "09:15:01".to_string(),
            is_buyer_maker: true,
        },
        TapeTrade {
            price: dec!(24501.0),
            size: dec!(25.0),
            time: "09:15:00".to_string(),
            is_buyer_maker: false,
        },
    ]);

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
                "Time & Sales — real-time trade print stream"
            </div>
            <div class="flex-grow overflow-y-auto p-2">
                <span class="text-xs text-text-muted">"Market tape feeds into Column 2 depth and Column 3 ticket."</span>
            </div>
        </div>
    }
    .into_any();

    let market_depth = view! {
        <div class="h-full flex flex-col">
            <HfMarketTape trades=Signal::derive(move || trades.get()) />
        </div>
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
