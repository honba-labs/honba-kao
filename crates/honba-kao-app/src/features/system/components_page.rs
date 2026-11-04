//! `ComponentsPage` — embeds and renders the design system showcase (`components.html`).

use leptos::prelude::*;

#[component]
pub fn ComponentsPage() -> impl IntoView {
    view! {
        <div class="components-showcase-page flex flex-col h-full w-full bg-surface-base overflow-hidden">
            <div class="flex items-center justify-between px-4 py-2 border-b border-border-soft bg-surface-subtle text-xs text-text-muted flex-shrink-0">
                <div class="flex items-center gap-2">
                    <span class="font-semibold text-text-high">"Design System Component Showcase"</span>
                    <span class="px-2 py-0.5 rounded-full text-[10px] bg-emerald-500/15 text-emerald-400 font-mono">
                        "honba-kao-ui-core"
                    </span>
                </div>
                <div class="flex items-center gap-3">
                    <a
                        href="/components.html"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="text-emerald-400 hover:underline flex items-center gap-1"
                    >
                        <span>"Open in new tab ↗"</span>
                    </a>
                </div>
            </div>
            <iframe
                src="/components.html"
                class="flex-1 w-full h-full border-0 bg-transparent"
                title="honba-kao Design System Component Showcase"
            />
        </div>
    }
}
