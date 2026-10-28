use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;
use openalgo_charts::OpenalgoChart;

#[component]
pub fn MultiChartPage() -> impl IntoView {
    view! {
        <HfPage title="Multi-Pane Trading Charts" subtitle="Synchronized crosshairs and multi-timeframe grid">
            <div class="grid grid-cols-1 laptop:grid-cols-2 gap-3 h-[750px]">
                <div class="rounded border border-border-soft bg-surface-base p-2 flex flex-col">
                    <span class="text-xs font-mono font-bold text-text-high mb-1">"NIFTY50 (1m)"</span>
                    <OpenalgoChart data=Signal::derive(Vec::new) />
                </div>
                <div class="rounded border border-border-soft bg-surface-base p-2 flex flex-col">
                    <span class="text-xs font-mono font-bold text-text-high mb-1">"BANKNIFTY (5m)"</span>
                    <OpenalgoChart data=Signal::derive(Vec::new) />
                </div>
                <div class="rounded border border-border-soft bg-surface-base p-2 flex flex-col">
                    <span class="text-xs font-mono font-bold text-text-high mb-1">"FINNIFTY (15m)"</span>
                    <OpenalgoChart data=Signal::derive(Vec::new) />
                </div>
                <div class="rounded border border-border-soft bg-surface-base p-2 flex flex-col">
                    <span class="text-xs font-mono font-bold text-text-high mb-1">"MIDCPNIFTY (1h)"</span>
                    <OpenalgoChart data=Signal::derive(Vec::new) />
                </div>
            </div>
        </HfPage>
    }
}
