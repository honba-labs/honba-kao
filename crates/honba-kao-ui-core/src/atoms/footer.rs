//! `HfFooterBar` — terminal footer status bar with latency, block height, and protocol status.
//! Ported from `secured-finance-app` atom `Footer`.

use leptos::prelude::*;

#[component]
pub fn HfFooterBar(
    #[prop(into)] items: Vec<(String, Signal<String>)>, // (label, value signal)
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let base = "hf-footer-bar flex items-center justify-between px-3 py-1.5 border-t border-border-soft bg-surface-subtle text-[10px] font-mono text-text-muted";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <div class=classes>
            <div class="hf-footer-left flex gap-4">
                {items.into_iter().map(|(label, value)| {
                    view! {
                        <div class="hf-footer-item flex flex-col gap-0.5">
                            <span class="uppercase tracking-wider">{label}</span>
                            <span class="text-text-high">{move || value.get()}</span>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
