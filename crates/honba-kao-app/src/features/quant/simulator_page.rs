use honba_api_wasm::replay::ReplayClock;
use honba_kao_ui_core::primitive::{ButtonSize, ButtonVariant, HfButton};
use honba_kao_ui_core::templates::HfMosaicWorkspace;
use leptos::prelude::*;

#[component]
pub fn SimulatorPage() -> impl IntoView {
    let clock = RwSignal::new(ReplayClock::new(1700000000, 1.0));

    view! {
        <HfMosaicWorkspace>
            <div class="col-span-12 laptop:col-span-4 p-4 rounded border border-border-soft bg-surface-base">
                <div class="flex items-center gap-3">
                    <HfButton
                        variant=ButtonVariant::Primary
                        size=ButtonSize::Sm
                        on_click=Callback::new(move |_| {
                            clock.update(|c| c.play());
                        })
                    >
                        "Play"
                    </HfButton>
                    <HfButton
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Sm
                        on_click=Callback::new(move |_| {
                            clock.update(|c| c.pause());
                        })
                    >
                        "Pause"
                    </HfButton>
                    <span class="font-mono text-xs text-text-muted">
                        {move || format!("Speed: {:.1}x | Running: {}", clock.get().speed_multiplier, clock.get().is_running)}
                    </span>
                </div>
                <p class="text-xs text-text-muted mt-2">
                    "Deterministic tick player driven by honba-kao-api-wasm replay engine."
                </p>
            </div>
            <div class="col-span-12 laptop:col-span-8 p-4 rounded border border-border-soft bg-surface-base">
                <span class="text-xs font-bold text-text-high font-mono block mb-1">"Replay Tape"</span>
                <div class="h-40 overflow-y-auto font-mono text-xs text-text-muted border border-border-soft p-2 bg-surface-subtle">
                    <span>"Waiting for journal events from api-wasm::replay..."</span>
                </div>
            </div>
        </HfMosaicWorkspace>
    }
}
