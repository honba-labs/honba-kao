use crate::shared::contexts::HonbaClientContext;
use honba_kao_ui_core::molecules::fno::OptionLegQuote;
use honba_kao_ui_core::organisms::{
    HfMarketTopBar, HfOptionChain, HfOrderTicket, OptionStrikePair, OrderIntent,
};
use honba_kao_ui_core::templates::HfThreeColumnTradingLayout;
use leptos::prelude::*;
use rust_decimal_macros::dec;

#[component]
pub fn OptionChainPage() -> impl IntoView {
    let client = use_context::<HonbaClientContext>().unwrap_or_else(fallback_client_context);
    let account = client.account;

    let spot = RwSignal::new(dec!(24500.0));
    let ltp = RwSignal::new(dec!(24500.0));
    let pairs = RwSignal::new(vec![
        OptionStrikePair {
            strike: dec!(24400.0),
            call: OptionLegQuote {
                ltp: dec!(165.0),
                change_pct: dec!(12.5),
                volume: 50000,
                open_interest: 120000,
                oi_change: 15000,
                iv: dec!(0.142),
                delta: dec!(0.65),
                gamma: dec!(0.0004),
            },
            put: OptionLegQuote {
                ltp: dec!(42.0),
                change_pct: dec!(-18.2),
                volume: 32000,
                open_interest: 85000,
                oi_change: -8000,
                iv: dec!(0.151),
                delta: dec!(-0.35),
                gamma: dec!(0.0004),
            },
        },
        OptionStrikePair {
            strike: dec!(24500.0),
            call: OptionLegQuote {
                ltp: dec!(98.0),
                change_pct: dec!(5.4),
                volume: 120000,
                open_interest: 250000,
                oi_change: 32000,
                iv: dec!(0.138),
                delta: dec!(0.50),
                gamma: dec!(0.0005),
            },
            put: OptionLegQuote {
                ltp: dec!(92.0),
                change_pct: dec!(-6.8),
                volume: 110000,
                open_interest: 240000,
                oi_change: 28000,
                iv: dec!(0.140),
                delta: dec!(-0.50),
                gamma: dec!(0.0005),
            },
        },
        OptionStrikePair {
            strike: dec!(24600.0),
            call: OptionLegQuote {
                ltp: dec!(48.0),
                change_pct: dec!(-15.0),
                volume: 45000,
                open_interest: 90000,
                oi_change: -12000,
                iv: dec!(0.145),
                delta: dec!(0.32),
                gamma: dec!(0.0004),
            },
            put: OptionLegQuote {
                ltp: dec!(172.0),
                change_pct: dec!(14.1),
                volume: 62000,
                open_interest: 140000,
                oi_change: 19000,
                iv: dec!(0.148),
                delta: dec!(-0.68),
                gamma: dec!(0.0004),
            },
        },
    ]);

    let top_bar = view! {
        <HfMarketTopBar
            symbol=Signal::derive(|| "NIFTY50".to_string())
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
        <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
            "Underlying spot chart and payoff curve preview for the selected straddle."
        </div>
    }
    .into_any();

    let market_depth = view! {
        <HfOptionChain
            spot_price=Signal::derive(move || spot.get())
            pairs=Signal::derive(move || pairs.get())
            on_select_leg=Callback::new(|_| {})
        />
    }
    .into_any();

    let order_ticket = view! {
        <HfOrderTicket
            symbol=Signal::derive(|| "NIFTY50".to_string())
            mark_price=Signal::derive(move || ltp.get())
            best_bid=Signal::derive(|| Some(dec!(24498.0)))
            best_ask=Signal::derive(|| Some(dec!(24502.0)))
            available_capital=Signal::derive(move || {
                account
                    .get()
                    .map(|a| a.available_margin)
                    .unwrap_or(dec!(500000.0))
            })
            margin_requirement_ratio=Signal::derive(|| dec!(0.15))
            on_submit_order=Callback::new(move |intent: OrderIntent| {
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

    let dock = view! {
        <div class="p-2 rounded border border-border-soft bg-surface-base text-xs text-text-muted overflow-y-auto">
            "F&O active positions and multi-leg order status dock."
        </div>
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
