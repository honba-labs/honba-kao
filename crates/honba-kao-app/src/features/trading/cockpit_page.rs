use crate::shared::contexts::HonbaClientContext;
use honba_kao_ui_core::organisms::{
    BookLevel, HfMarketTopBar, HfOrderBook, HfOrderTicket, HfTradingDock, OrderIntent,
};
use honba_kao_ui_core::templates::HfThreeColumnTradingLayout;
use leptos::prelude::*;
use openalgo_charts::OpenalgoChart;
use rust_decimal_macros::dec;
use std::sync::Arc;

#[component]
pub fn CockpitPage() -> impl IntoView {
    let client = use_context::<HonbaClientContext>().unwrap_or_else(fallback_client_context);
    // Copy out the Copy `RwSignal` handles so `client` stays available for the
    // order-submission callback.
    let account = client.account;
    let positions = client.positions;
    let orders = client.orders;

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
        BookLevel {
            price: dec!(24490.0),
            size: dec!(500.0),
            cumulative_size: dec!(950.0),
            order_count: 12,
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
        BookLevel {
            price: dec!(24510.0),
            size: dec!(450.0),
            cumulative_size: dec!(850.0),
            order_count: 10,
        },
    ]);

    let spread = RwSignal::new(Some((dec!(4.0), dec!(1.63))));

    let available_capital = Signal::derive(move || {
        account
            .get()
            .map(|a| a.available_margin)
            .unwrap_or(dec!(500000.0))
    });

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
        <OpenalgoChart data=Signal::derive(Vec::new) />
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
            available_capital=available_capital
            margin_requirement_ratio=Signal::derive(|| dec!(0.15))
            on_submit_order=Callback::new(move |intent: OrderIntent| {
                // Route live order submission through the OMS client.
                client.submit_order.run(honba_kao_client::dto::order::OrderIntent {
                    symbol: intent.symbol,
                    side: intent.side,
                    order_type: intent.order_type,
                    price: Some(intent.price),
                    trigger_price: None,
                    quantity: intent.quantity,
                });
            })
        />
    }
    .into_any();

    let open_orders_count = Signal::derive(move || orders.get().len());
    let positions_count = Signal::derive(move || positions.get().len());

    let dock = view! {
        <HfTradingDock
            positions_view=Arc::new(|| view! { <div class="text-xs text-text-muted p-3">"No open positions"</div> }.into_any())
            positions_count=positions_count
            open_orders_view=Arc::new(|| view! { <div class="text-xs text-text-muted p-3">"No active working orders"</div> }.into_any())
            open_orders_count=open_orders_count
            order_history_view=Arc::new(|| view! { <div class="text-xs text-text-muted p-3">"Order history clear"</div> }.into_any())
            fills_view=Arc::new(|| view! { <div class="text-xs text-text-muted p-3">"No trade executions today"</div> }.into_any())
        />
    }.into_any();

    view! {
        <div class="h-full w-full">
            <HfThreeColumnTradingLayout
                top_bar=top_bar
                visualizer=visualizer
                market_depth=market_depth
                order_ticket=order_ticket
                dock=dock
            />
        </div>
    }
}

/// Fallback client context when `HonbaClientContext` has not been provided.
fn fallback_client_context() -> HonbaClientContext {
    HonbaClientContext {
        oms_connected: RwSignal::new(false),
        feed_connected: RwSignal::new(false),
        latency_ms: RwSignal::new(None),
        account: RwSignal::new(None),
        positions: RwSignal::new(vec![]),
        holdings: RwSignal::new(vec![]),
        brokers: RwSignal::new(vec![]),
        orders: RwSignal::new(vec![]),
        liquidation_warnings: RwSignal::new(vec![]),
        submit_order: Callback::new(|_| {}),
        cancel_order: Callback::new(|_| {}),
        _order_event_source: None,
        _risk_event_source: None,
        _agent_event_source: None,
    }
}
