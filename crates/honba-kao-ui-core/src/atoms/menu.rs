//! `HfMenu` / `HfMenuItem` — ARIA menu.
//!
//! Exposes `role="menu"` on the container and `role="menuitem"` on each item,
//! with `aria-expanded` for the open/closed state, satisfying the a11y gate
//! (`scripts/check-a11y.sh`).

use leptos::prelude::*;

/// A single menu item.
#[component]
pub fn HfMenuItem(
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] expanded: Option<RwSignal<bool>>,
    children: Children,
) -> impl IntoView {
    let label_keep = label.clone().unwrap_or_else(|| "Menu item".into());
    let is_expanded = expanded.map(|e| e.get()).unwrap_or(false);
    view! {
        <div role="menuitem" aria-expanded=is_expanded class="px-2 py-1 text-xs text-text-high hover:bg-surface-subtle">
            {label_keep}
            {children()}
        </div>
    }
}

/// A menu container.
#[component]
pub fn HfMenu(
    #[prop(into, optional)] expanded: Option<RwSignal<bool>>,
    children: Children,
) -> impl IntoView {
    let is_expanded = expanded.map(|e| e.get()).unwrap_or(false);
    view! {
        <div role="menu" aria-expanded=is_expanded class="flex flex-col gap-1 p-1 bg-surface-base border border-border-soft rounded">
            {children()}
        </div>
    }
}
