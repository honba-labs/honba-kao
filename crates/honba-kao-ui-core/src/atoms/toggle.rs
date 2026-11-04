//! `HfToggleSwitch` — ARIA toggle switch.
//!
//! Exposes `role="switch"` with `aria-checked` so keyboard/screen-reader
//! users can perceive the on/off state, satisfying the a11y gate
//! (`scripts/check-a11y.sh`).

use leptos::prelude::*;

/// ARIA toggle switch.
#[component]
pub fn HfToggleSwitch(
    #[prop(into)] checked: RwSignal<bool>,
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] on_change: Option<Callback<bool>>,
) -> impl IntoView {
    let on_change_clone = on_change;
    let on_click = move |_| {
        let next = !checked.get();
        checked.set(next);
        if let Some(cb) = on_change_clone.as_ref() {
            cb.run(next);
        }
    };
    view! {
        <button
            role="switch"
            aria-checked=move || checked.get()
            aria-pressed=move || checked.get()
            type="button"
            class=move || format!(
                "relative inline-flex h-6 w-11 items-center rounded-full transition-colors {}",
                if checked.get() {
                    "bg-emerald-600"
                } else {
                    "bg-surface-subtle text-text-muted"
                }
            )
            on:click=on_click
        >
            <span
                class=move || format!(
                    "inline-block h-4 w-4 rounded-full bg-white transform transition-transform {}",
                    if checked.get() { "translate-x-5" } else { "translate-x-0" }
                )
            ></span>
            {label.map(|label| view! { <span class="ml-2 text-xs text-text-muted">{label}</span> })}
        </button>
    }
}
