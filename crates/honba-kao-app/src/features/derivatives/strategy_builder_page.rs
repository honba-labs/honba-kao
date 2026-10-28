use honba_kao_ui_core::molecules::fno::StrategyLeg;
use honba_kao_ui_core::organisms::{HfMarketTopBar, HfOrderTicket, HfStrategyBuilder};
use honba_kao_ui_core::templates::HfThreeColumnTradingLayout;
use leptos::prelude::*;

#[component]
pub fn StrategyBuilderPage() -> impl IntoView {
    let legs = RwSignal::new(vec![
        StrategyLeg {
            side: "BUY".to_string(),
            strike: 24500.0,
            expiry: "28-OCT".to_string(),
            lots: 1.0,
        },
        StrategyLeg {
            side: "SELL".to_string(),
            strike: 24700.0,
            expiry: "28-OCT".to_string(),
            lots: 1.0,
        },
    ]);
    let ltp = RwSignal::new(24500.0);

    let top_bar = view! {
        <HfMarketTopBar
            symbol=Signal::derive(|| "NIFTY50".to_string())
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
        <HfStrategyBuilder
            legs=legs
            max_profit=Signal::derive(|| Some(8500.0))
            max_loss=Signal::derive(|| Some(3200.0))
            breakeven_points=Signal::derive(|| vec![24564.0])
            on_execute_strategy=Callback::new(|_| {})
        />
    }
    .into_any();

    let market_depth = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
            "Strike ladder with Call/Put moneyness tags and OI change bars."
        </div>
    }
    .into_any();

    let order_ticket = view! {
        <HfOrderTicket
            symbol=Signal::derive(|| "NIFTY50".to_string())
            mark_price=Signal::derive(move || ltp.get())
            best_bid=Signal::derive(|| Some(24498.0))
            best_ask=Signal::derive(|| Some(24502.0))
            available_capital=Signal::derive(|| 500_000.0)
            margin_requirement_ratio=Signal::derive(|| 0.15)
            on_submit_order=Callback::new(|_| {})
        />
    }
    .into_any();

    let dock = view! {
        <div class="p-2 rounded border border-border-soft bg-surface-base text-xs text-text-muted overflow-y-auto">
            "Multi-leg order status and option expiration journal."
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
