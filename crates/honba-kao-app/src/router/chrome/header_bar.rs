use honba_kao_ui_core::primitive::{ButtonSize, ButtonVariant, HfButton};
use leptos::prelude::*;

#[component]
pub fn HeaderBar(on_toggle_gobar: Callback<()>, on_toggle_agent: Callback<()>) -> impl IntoView {
    view! {
        <div class="header-bar flex items-center justify-between px-4 py-2 bg-surface-base border-b border-border-soft">
            <div class="flex items-center gap-4">
                <a href="/" class="flex items-center gap-2 text-text-high font-bold tracking-tight">
                    <span class="text-accent-primary font-mono text-base">"本場・顔"</span>
                    <span class="text-xs text-text-muted hidden tablet:inline">"honba-kao"</span>
                </a>

                <HfButton
                    class="gobar-trigger-btn flex items-center gap-2 px-3 py-1 bg-surface-subtle text-xs text-text-muted hover:text-text-high rounded border border-border-soft"
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Sm
                    on_click=Callback::new(move |_| on_toggle_gobar.run(()))
                >
                    <span class="font-mono">"Go to..."</span>
                    <span class="text-[10px] font-mono border border-border-strong px-1 rounded">"Ctrl+G"</span>
                </HfButton>
            </div>

            <div class="flex items-center gap-2">
                <HfButton
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Sm
                    on_click=Callback::new(move |_| on_toggle_agent.run(()))
                >
                    <span class="text-xs">"Analyst (Ctrl+J)"</span>
                </HfButton>

                <HfButton
                    href="/account".to_string()
                    variant=ButtonVariant::Tertiary
                    size=ButtonSize::Sm
                >
                    <span class="text-xs">"Account"</span>
                </HfButton>
            </div>
        </div>
    }
}
