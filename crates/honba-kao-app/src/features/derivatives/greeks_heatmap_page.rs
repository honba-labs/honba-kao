use honba_kao_ui_core::organisms::{HfMarketTopBar, HfOrderTicket};
use honba_kao_ui_core::templates::HfThreeColumnTradingLayout;
use leptos::prelude::*;
use rust_decimal_macros::dec;

#[component]
pub fn GreeksHeatmapPage() -> impl IntoView {
    let ltp = RwSignal::new(dec!(24500.0));

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
            "Volatility surface interpolation and charm/vanna cross-sensitivities."
        </div>
    }
    .into_any();

    let market_depth = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
            "Greeks surface: Delta, Gamma, Vega, Theta across strikes and tenors."
        </div>
    }
    .into_any();

    let order_ticket = view! {
        <HfOrderTicket
            symbol=Signal::derive(|| "NIFTY50".to_string())
            mark_price=Signal::derive(move || ltp.get())
            best_bid=Signal::derive(|| Some(dec!(24498.0)))
            best_ask=Signal::derive(|| Some(dec!(24502.0)))
            available_capital=Signal::derive(|| dec!(500000.0))
            margin_requirement_ratio=Signal::derive(|| dec!(0.15))
            on_submit_order=Callback::new(|_| {})
        />
    }
    .into_any();

    let dock = view! {
        <div class="p-2 rounded border border-border-soft bg-surface-base text-xs text-text-muted overflow-y-auto">
            "Strategy Greeks and margin heatmap dock."
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
