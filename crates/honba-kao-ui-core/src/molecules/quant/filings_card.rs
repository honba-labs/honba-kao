//! `HfFilingsCard` — corporate filing extract with highlighted entity mentions.

use leptos::prelude::*;

#[component]
pub fn HfFilingsCard(
    #[prop(into)] title: String,
    #[prop(into)] excerpt: String,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let base = "hf-filings-card rounded border border-border-soft bg-surface-base p-3 text-xs";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <div class=classes>
            <span class="text-xs font-semibold text-text-high block">{title}</span>
            <div class="hf-filings-excerpt mt-1 whitespace-pre-wrap break-words text-text-high">{excerpt}</div>
        </div>
    }
}
