use honba_kao_ui_core::molecules::quant::HfCointegrationBand;
use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn PairTradingPage() -> impl IntoView {
    view! {
        <HfPage title="Pair Trading & Cointegration Workstation" subtitle="Live Z-score spread monitor and hedge ratio tracking">
            <div class="max-w-2xl mx-auto flex flex-col gap-4">
                <div class="p-4 rounded border border-border-soft bg-surface-base">
                    <div class="flex justify-between items-center mb-3">
                        <span class="font-bold text-sm text-text-primary">"INFY / TCS Spread"</span>
                        <span class="text-xs font-mono text-text-muted">"Hedge Ratio: 0.82 • Half-life: 14.2 bars"</span>
                    </div>
                    <HfCointegrationBand z_score=Signal::derive(|| 1.45) />
                </div>
            </div>
        </HfPage>
    }
}
