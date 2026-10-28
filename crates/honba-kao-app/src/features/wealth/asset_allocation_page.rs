use honba_kao_ui_core::molecules::wealth::{Allocation, HfAssetAllocationBar};
use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn AssetAllocationPage() -> impl IntoView {
    view! {
        <HfPage title="Strategic Asset Allocation" subtitle="Policy weights vs tactical deviations across Equity, Fixed Income, Gold, and Liquid Cash">
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
        </HfPage>
    }
}
