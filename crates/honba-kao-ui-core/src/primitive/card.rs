//! `HfCard` — visual container with token-based gradient border for highlighted cards.
//! Ported from `secured-finance-app` atom `GradientBox`.

use leptos::prelude::*;

#[component]
pub fn HfCard(
    #[prop(into, optional)] class: Option<String>,
    #[prop(default = false)] outlined: bool,
    children: Children,
) -> impl IntoView {
    let base = if outlined {
        "hf-card rounded-lg border border-border-soft bg-surface-base"
    } else {
        "hf-card rounded-lg border border-border-soft bg-surface-base shadow-sm"
    };
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <div class=classes>{children()}</div>
    }
}
