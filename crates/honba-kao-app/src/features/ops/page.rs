//! Operations Console page component.
use leptos::prelude::*;

/// Operations Console page component.
#[component]
pub fn OpsPage() -> impl IntoView {
    view! {
        <section class="p-6">
            <h2 class="text-2xl font-bold text-primary mb-6">Operations Console</h2>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Trader Desk</h3>
                    <p class="text-sm">Open orders, executed trades, position monitoring</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Quant Desk</h3>
                    <p class="text-sm">Model runs, backtest status, alpha factor performance</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">PM Desk</h3>
                    <p class="text-sm">Portfolio snapshot, PnL, risk alerts</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Risk Desk</h3>
                    <p class="text-sm">VaR, exposure heatmap, stress test results</p>
                </div>
                <div class="col-span-full p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">System Desk</h3>
                    <p class="text-sm">Feed health, WebSocket latency, data quality scoring</p>
                </div>
            </div>
            <div class="mt-6 p-4 border rounded elevated" style="background: var(--risk-critical); color: var(--text-inverse);">
                <h3 class="font-semibold text-warning mb-2">Emergency Kill Switches</h3>
                <div class="grid grid-cols-1 gap-2 md:grid-cols-4">
                    <button class="w-full py-2 px-4 border rounded elevated text-sm font-medium text-danger hover:bg-danger/10">
                        "Panic Sell All"
                    </button>
                    <button class="w-full py-2 px-4 border rounded elevated text-sm font-medium text-danger hover:bg-danger/10">
                        "Cancel All Pending"
                    </button>
                    <button class="w-full py-2 px-4 border rounded elevated text-sm font-medium text-danger hover:bg-danger/10">
                        "Halt Trading Engine"
                    </button>
                    <button class="w-full py-2 px-4 border rounded elevated text-sm font-medium text-danger hover:bg-danger/10">
                        "Safe Mode Toggle"
                    </button>
                </div>
            </div>
        </section>
    }
}