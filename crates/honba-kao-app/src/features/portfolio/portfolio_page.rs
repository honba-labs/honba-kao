use honba_kao_ui_core::molecules::crypto::HfLiquidationProgressBar;
use honba_kao_ui_core::organisms::{CollateralAssetRow, HfCollateralOrganism};
use honba_kao_ui_core::templates::{HfPage, HfTwoColumnSplitLayout};
use leptos::prelude::*;

#[component]
pub fn PortfolioPage() -> impl IntoView {
    let assets = RwSignal::new(vec![
        CollateralAssetRow {
            symbol: "INR".to_string(),
            name: "Indian Rupee (Cash)".to_string(),
            balance: 250_000.0,
            price: 1.0,
            haircut_pct: 0.0,
            collateral_value: 250_000.0,
        },
        CollateralAssetRow {
            symbol: "RELIANCE".to_string(),
            name: "Reliance Industries".to_string(),
            balance: 100.0,
            price: 2950.0,
            haircut_pct: 15.0,
            collateral_value: 250_750.0,
        },
    ]);

    let primary = view! {
        <HfCollateralOrganism
            assets=Signal::derive(move || assets.get())
            total_collateral_value=Signal::derive(|| 500_750.0)
            margin_used=Signal::derive(|| 125_000.0)
        />
    }
    .into_any();

    let rail = view! {
        <div class="flex flex-col gap-3">
            <HfLiquidationProgressBar
                current_utilization=Signal::derive(|| 24.9)
                liquidation_threshold=Signal::derive(|| 85.0)
                est_liquidation_price=Signal::derive(|| None)
            />
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
