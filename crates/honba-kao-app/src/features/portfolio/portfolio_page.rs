use crate::shared::contexts::HonbaClientContext;
use honba_kao_ui_core::molecules::crypto::HfLiquidationProgressBar;
use honba_kao_ui_core::organisms::{CollateralAssetRow, HfCollateralOrganism};
use honba_kao_ui_core::templates::{HfPage, HfTwoColumnSplitLayout};
use leptos::prelude::*;
use num_traits::ToPrimitive;
use rust_decimal_macros::dec;

#[component]
pub fn PortfolioPage() -> impl IntoView {
    let client = use_context::<HonbaClientContext>().unwrap_or_else(fallback_client_context);

    // Collateral assets derived from live account/positions via the client.
    let assets = Signal::derive(move || {
        let positions = client.positions.get();
        if positions.is_empty() {
            vec![CollateralAssetRow {
                symbol: "INR".to_string(),
                name: "Indian Rupee (Cash)".to_string(),
                balance: client
                    .account
                    .get()
                    .map(|a| a.total_equity)
                    .unwrap_or(dec!(250000.0)),
                price: dec!(1.0),
                haircut_pct: dec!(0.0),
                collateral_value: client
                    .account
                    .get()
                    .map(|a| a.total_equity)
                    .unwrap_or(dec!(250000.0)),
            }]
        } else {
            positions
                .iter()
                .map(|p| CollateralAssetRow {
                    symbol: p.symbol.clone(),
                    name: p.symbol.clone(),
                    balance: p.quantity,
                    price: p.current_price,
                    haircut_pct: dec!(15.0),
                    collateral_value: p.quantity * p.current_price,
                })
                .collect()
        }
    });

    let total_collateral_value = Signal::derive(move || {
        client
            .account
            .get()
            .map(|a| a.total_equity)
            .unwrap_or_else(|| {
                assets
                    .get()
                    .iter()
                    .map(|a| a.collateral_value)
                    .fold(dec!(0.0), |acc, v| acc + v)
            })
    });

    let margin_used = Signal::derive(move || {
        client
            .account
            .get()
            .map(|a| a.used_margin)
            .unwrap_or(dec!(125000.0))
    });

    let current_utilization = Signal::derive(move || {
        let total = total_collateral_value.get();
        let used = margin_used.get();
        if total.is_zero() {
            dec!(0.0)
        } else {
            (used / total) * dec!(100.0)
        }
    });

    let primary = view! {
        <HfCollateralOrganism
            assets=assets
            total_collateral_value=total_collateral_value
            margin_used=margin_used
        />
    }
    .into_any();

    let rail = view! {
        <div class="flex flex-col gap-3">
            <HfLiquidationProgressBar
                current_utilization=Signal::derive(move || current_utilization.get().to_f64().unwrap_or(0.0))
                liquidation_threshold=Signal::derive(|| dec!(85.0).to_f64().unwrap_or(85.0))
                est_liquidation_price=Signal::derive(|| None)
            />
            <div class="text-xs text-text-muted mt-2">
                {move || format!("Broker connections: {}", client.brokers.get().len())}
                {move || {
                    let connected = client.brokers.get().iter().filter(|b| b.is_connected).count();
                    format!(" | Connected: {connected}")
                }}
            </div>
        </div>
    }
    .into_any();

    view! {
        <HfPage title="Portfolio & Collateral Hub" subtitle="Real-time multi-asset haircuts, SPAN margin utilization, and holdings">
            <HfTwoColumnSplitLayout
                primary=primary
                rail=rail
            />
        </HfPage>
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
