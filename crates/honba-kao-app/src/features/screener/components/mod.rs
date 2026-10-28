use honba_kao_ui_core::primitive::{ButtonSize, ButtonVariant, HfButton};
use leptos::prelude::*;

#[component]
pub fn ScreenerHeroDeck() -> impl IntoView {
    view! {
        <div class="screener-hero-deck grid grid-cols-1 laptop:grid-cols-4 gap-3 mb-4">
            <div class="p-3 rounded border border-border-soft bg-surface-base">
                <span class="text-[10px] text-text-muted uppercase">"Total Filtered"</span>
                <div class="text-base font-bold font-mono text-text-high">"142 / 2,150"</div>
            </div>
            <div class="p-3 rounded border border-border-soft bg-surface-base">
                <span class="text-[10px] text-text-muted uppercase">"Top Sector"</span>
                <div class="text-base font-bold font-mono text-emerald-400">"IT & Software (+2.4%)"</div>
            </div>
            <div class="p-3 rounded border border-border-soft bg-surface-base">
                <span class="text-[10px] text-text-muted uppercase">"Median P/E"</span>
                <div class="text-base font-bold font-mono text-text-high">"22.8"</div>
            </div>
            <div class="p-3 rounded border border-border-soft bg-surface-base">
                <span class="text-[10px] text-text-muted uppercase">"Active Presets"</span>
                <div class="flex items-center gap-1 mt-1">
                    <HfButton
                        variant=ButtonVariant::Tertiary
                        size=ButtonSize::Xxs
                    >
                        "High Growth"
                    </HfButton>
                </div>
            </div>
        </div>
    }
}
