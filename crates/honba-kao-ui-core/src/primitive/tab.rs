//! `HfTab`, `HfSubTab` — primary / secondary navigation tabs.
//! Ported from `secured-finance-app` atoms `Tab`, `SubTab`.

use leptos::prelude::*;

#[component]
pub fn HfTab(
    #[prop(into)] value: String,
    #[prop(into)] active: RwSignal<String>,
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] on_change: Option<Callback<String>>,
) -> impl IntoView {
    let value_keep = value.clone();
    let label_keep = label.clone().unwrap_or_else(|| value_keep.clone());
    let value_for_active = value_keep.clone();
    let is_active = move || active.get() == value_for_active;
    let on_change_clone = on_change.clone();
    let on_click = move |_| {
        active.set(value_keep.clone());
        if let Some(cb) = on_change_clone.as_ref() {
            cb.run(value_keep.clone());
        }
    };
    view! {
        <button
            type="button"
            class=move || format!(
                "hf-tab px-3 py-1.5 text-xs font-medium border-b-2 transition-colors {}",
                if is_active() {
                    "border-emerald-600 text-emerald-400 bg-emerald-600/10"
                } else {
                    "border-transparent text-text-muted hover:text-text-high hover:bg-surface-subtle"
                }
            )
            on:click=on_click
        >
            {label_keep}
        </button>
    }
}

/// Secondary navigation tab for sub-views within complex panels.
#[component]
pub fn HfSubTab(
    #[prop(into)] value: String,
    #[prop(into)] active: RwSignal<String>,
    #[prop(into, optional)] label: Option<String>,
) -> impl IntoView {
    let value_keep = value.clone();
    let label_keep = label.clone().unwrap_or_else(|| value_keep.clone());
    let value_for_active = value_keep.clone();
    let is_active = move || active.get() == value_for_active;
    view! {
        <button
            type="button"
            class=move || format!(
                "hf-sub-tab px-2 py-1 rounded-md text-xs transition-colors {}",
                if is_active() {
                    "bg-surface-elevated text-text-high font-semibold"
                } else {
                    "text-text-muted hover:text-text-high hover:bg-surface-subtle"
                }
            )
            on:click=move |_| active.set(value_keep.clone())
        >
            {label_keep}
        </button>
    }
}
