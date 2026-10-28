use honba_kao_ui_core::templates::HfMosaicWorkspace;
use leptos::prelude::*;

#[component]
pub fn LaunchpadPage() -> impl IntoView {
    view! {
        <HfMosaicWorkspace>
            <div class="col-span-12 laptop:col-span-6 p-4 rounded border border-border-soft bg-surface-base">
                <span class="text-xs font-bold text-text-high font-mono">"Tile 1: Execution Station"</span>
            </div>
            <div class="col-span-12 laptop:col-span-6 p-4 rounded border border-border-soft bg-surface-base">
                <span class="text-xs font-bold text-text-high font-mono">"Tile 2: Risk Telemetry"</span>
            </div>
        </HfMosaicWorkspace>
    }
}
