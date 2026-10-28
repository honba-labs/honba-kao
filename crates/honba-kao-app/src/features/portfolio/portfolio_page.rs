use honba_kao_ui_core::molecules::crypto::HfLiquidationProgressBar;
use honba_kao_ui_core::organisms::{CollateralAssetRow, HfCollateralOrganism};
use honba_kao_ui_core::templates::{HfPage, HfTwoColumnSplitLayout};
use leptos::prelude::*;
use rust_decimal_macros::dec;

#[component]
pub fn PortfolioPage() -> impl IntoView {
    let assets = RwSignal::new(vec![
        CollateralAssetRow {
            symbol: "INR".to_string(),
            name: "Indian Rupee (Cash)".to_string(),
            balance: dec!(250000.0),
            price: dec!(1.0),
            haircut_pct: dec!(0.0),
            collateral_value: dec!(250000.0),
        },
        CollateralAssetRow {
            symbol: "RELIANCE".to_string(),
            name: "Reliance Industries".to_string(),
            balance: dec!(100.0),
            price: dec!(2950.0),
            haircut_pct: dec!(15.0),
            collateral_value: dec!(250750.0),
        },
    ]);

    let primary = view! {
        <HfCollateralOrganism
            assets=Signal::derive(move || assets.get())
            total_collateral_value=Signal::derive(|| dec!(500750.0))
            margin_used=Signal::derive(|| dec!(125000.0))
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
