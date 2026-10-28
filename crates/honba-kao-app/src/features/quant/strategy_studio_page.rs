use honba_kao_ui_core::templates::HfMosaicWorkspace;
use leptos::prelude::*;

#[component]
pub fn StrategyStudioPage() -> impl IntoView {
    view! {
        <HfMosaicWorkspace>
            <div class="col-span-12 laptop:col-span-6 p-4 rounded border border-border-soft bg-surface-base">
                <h3 class="text-xs font-semibold text-text-primary mb-2">"Signal Rule Builder"</h3>
                <div class="text-xs text-text-muted">
                    "Rule-based event trigger composition, trailing stop definitions, and bracket order wiring."
                </div>
            </div>
            <div class="col-span-12 laptop:col-span-6 p-4 rounded border border-border-soft bg-surface-base">
                <h3 class="text-xs font-semibold text-text-primary mb-2">"Code Exporter"</h3>
                <div class="text-xs text-text-muted">
                    "Export compiled strategy to api-wasm replay engine for deterministic backtest."
                </div>
            </div>
        </HfMosaicWorkspace>
    }
}
