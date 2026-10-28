use honba_kao_ui_core::molecules::wealth::{Allocation, HfAssetAllocationBar};
use honba_kao_ui_core::templates::{HfPage, HfTwoColumnSplitLayout};
use leptos::prelude::*;

#[component]
pub fn AssetAllocationPage() -> impl IntoView {
    let primary = view! {
        <div class="max-w-xl mx-auto p-4 rounded border border-border-soft bg-surface-base">
            <span class="text-xs font-semibold text-text-muted mb-2 block">"Current Total Portfolio Allocation"</span>
            <HfAssetAllocationBar
                alloc=Signal::derive(|| Allocation {
                    equity: 60.0,
                    debt: 25.0,
                    cash: 10.0,
                    gold: 5.0,
                })
            />
        </div>
    }
    .into_any();

    let rail = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base flex flex-col gap-3">
            <h3 class="text-xs font-semibold text-text-primary mb-1">"Policy Weights"</h3>
            <span class="text-xs text-text-muted">"Equity, Fixed Income, Gold, Liquid Cash."</span>
        </div>
    }
    .into_any();

    view! {
        <HfPage
            title="Strategic Asset Allocation"
            subtitle="Policy weights vs tactical deviations across Equity, Fixed Income, Gold, and Liquid Cash"
        >
            <HfTwoColumnSplitLayout primary=primary rail=rail />
        </HfPage>
    }
}
