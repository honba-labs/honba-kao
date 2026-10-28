use honba_kao_ui_core::primitive::{ButtonVariant, HfButton};
use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <HfPage
            title="Institutional Financial Workstation"
            subtitle="Executive summary, market indices, portfolio overview, and direct workstation entry"
        >
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div class="p-6 rounded-lg border border-border-soft bg-surface-base flex flex-col justify-between">
                    <div>
                        <h3 class="text-base font-semibold text-text-primary mb-2">"Trading Cockpit"</h3>
                        <p class="text-xs text-text-muted mb-4">
                            "Ultra-low latency institutional execution dock, DOM ladder, multi-chart canvas, and Level 2 depth."
                        </p>
                    </div>
                    <a href="/trading/cockpit">
                        <HfButton variant=ButtonVariant::Primary class="w-full">
                            "Open Cockpit"
                        </HfButton>
                    </a>
                </div>

                <div class="p-6 rounded-lg border border-border-soft bg-surface-base flex flex-col justify-between">
                    <div>
                        <h3 class="text-base font-semibold text-text-primary mb-2">"Derivatives & F&O"</h3>
                        <p class="text-xs text-text-muted mb-4">
                            "Option chain analytics with real-time Black-Scholes Greeks, OI heatmaps, and multi-leg payoff graph builder."
                        </p>
                    </div>
                    <a href="/derivatives/options">
                        <HfButton variant=ButtonVariant::Secondary class="w-full">
                            "Analyze Options"
                        </HfButton>
                    </a>
                </div>

                <div class="p-6 rounded-lg border border-border-soft bg-surface-base flex flex-col justify-between">
                    <div>
                        <h3 class="text-base font-semibold text-text-primary mb-2">"Quantitative Lab"</h3>
                        <p class="text-xs text-text-muted mb-4">
                            "Deterministic historical replay engine, statistical arbitrage studio, and factor cointegration research."
                        </p>
                    </div>
                    <a href="/quant/simulator">
                        <HfButton variant=ButtonVariant::Secondary class="w-full">
                            "Launch Lab"
                        </HfButton>
                    </a>
                </div>
            </div>
        </HfPage>
    }
}
