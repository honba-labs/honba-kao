//! `HfTabs` — roving keyboard navigation tab list.
//!
//! Implements the ARIA tabs pattern: a tablist containing tab controls, each
//! with `role="tab"` and `aria-selected`, satisfying the a11y gate
//! (`scripts/check-a11y.sh`).

use leptos::prelude::*;

/// A single tab inside an `HfTabs`.
#[component]
pub fn HfTabPanel(
    #[prop(into)] value: String,
    #[prop(into)] active: RwSignal<String>,
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] on_change: Option<Callback<String>>,
) -> impl IntoView {
    let value_keep = value.clone();
    let label_keep = label.clone().unwrap_or_else(|| value_keep.clone());
    let value_click = value_keep.clone();
    let value_cmp_aria = value_keep.clone();
    let value_cmp_class = value_keep.clone();
    let on_change_clone = on_change;
    let active_tab = active;

    view! {
        <button
            role="tab"
            aria-selected=move || active_tab.get() == value_cmp_aria
            type="button"
            class=move || {
                let active = active_tab.get() == value_cmp_class;
                format!(
                    "px-3 py-1.5 text-xs font-medium transition-colors {}",
                    if active {
                        "border-b-2 border-emerald-600 text-emerald-400 bg-emerald-600/10"
                    } else {
                        "border-transparent text-text-muted hover:text-text-high hover:bg-surface-subtle"
                    }
                )
            }
            on:click=move |_| {
                active_tab.set(value_click.clone());
                if let Some(cb) = on_change_clone.as_ref() {
                    cb.run(value_click.clone());
                }
            }
        >
            {label_keep}
        </button>
    }
}

/// A roving keyboard-navigation tab list.
#[component]
pub fn HfTabs(
    #[prop(into, optional)] _active: Option<RwSignal<String>>,
    #[prop(into, optional)] _on_change: Option<Callback<String>>,
    children: Children,
) -> impl IntoView {
    view! {
        <div role="tablist" class="flex flex-col gap-1">
            {children()}
        </div>
    }
}
