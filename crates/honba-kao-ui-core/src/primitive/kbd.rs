//! `HfKbdBadge` — terminal hotkey tags (e.g. `[Ctrl+G]`, `[Esc]`, `[F1]`).

use leptos::prelude::*;

#[component]
pub fn HfKbdBadge(
    #[prop(into)] key: String,
    #[prop(into, optional)] size: Option<&'static str>,
) -> impl IntoView {
    let size = size.unwrap_or("px-1.5 py-0.5");
    view! {
        <span class=format!(
            "hf-kbd-badge inline-flex items-center {} rounded border border-border-soft bg-surface-subtle text-[10px] font-mono font-bold text-text-high select-none",
            size
        )>
            {key}
        </span>
    }
}
