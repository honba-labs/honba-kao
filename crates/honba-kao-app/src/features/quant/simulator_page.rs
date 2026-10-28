use honba_api_wasm::replay::ReplayClock;
use honba_kao_ui_core::primitive::{ButtonSize, ButtonVariant, HfButton};
use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn SimulatorPage() -> impl IntoView {
    let clock = RwSignal::new(ReplayClock::new(1700000000, 1.0));

    view! {
        <HfPage title="Tick Replay & Simulation Engine" subtitle="Deterministic tick player driven by honba-kao-api-wasm">
            <div class="flex flex-col gap-4 p-4 rounded border border-border-soft bg-surface-base">
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
            </div>
        </HfPage>
    }
}
