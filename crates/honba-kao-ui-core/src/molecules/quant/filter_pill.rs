//! `HfFilterPill` — multi-select factor filter pill for screeners.

use leptos::prelude::*;

#[component]
pub fn HfFilterPill(
    #[prop(into)] label: String,
    #[prop(into)] active: RwSignal<bool>,
    #[prop(into, optional)] on_toggle: Option<Callback<()>>,
) -> impl IntoView {
    let on_toggle = on_toggle.unwrap_or_else(|| Callback::new(move |_| {}));
    view! {
        <button
            type="button"
            class=move || format!(
                "hf-filter-pill inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-md text-xs font-medium border transition-colors {}",
                if active.get() { "bg-emerald-600 text-white border-emerald-600" } else { "bg-surface-subtle text-text-muted border-border-soft hover:text-text-high" }
            )
            on:click=move |_| {
                active.set(!active.get());
                on_toggle.run(());
            }
        >
            {label}
        </button>
    }
}
