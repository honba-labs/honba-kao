//! `HfSortArrows` — tri-state table sorting indicator (Neutral, Ascending, Descending).
//! Ported from `secured-finance-app` atom `SortArrows`.

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortDirection {
    #[default]
    Neutral,
    Ascending,
    Descending,
}

fn sort_symbol(direction: SortDirection) -> String {
    match direction {
        SortDirection::Neutral => "⇅".to_string(),
        SortDirection::Ascending => "↑".to_string(),
        SortDirection::Descending => "↓".to_string(),
    }
}

fn sort_color(direction: SortDirection) -> &'static str {
    match direction {
        SortDirection::Neutral => "text-text-muted",
        SortDirection::Ascending => "text-emerald-400",
        SortDirection::Descending => "text-rose-400",
    }
}

#[component]
pub fn HfSortArrows(
    #[prop(into)] direction: RwSignal<SortDirection>,
    #[prop(into, optional)] on_cycle: Option<Callback<SortDirection>>,
) -> impl IntoView {
    let symbol = Signal::derive(move || sort_symbol(direction.get()));
    let dir_class = Signal::derive(move || sort_color(direction.get()));

    view! {
        <button
            type="button"
            class="hf-sort-arrows inline-flex items-center gap-1 px-1 py-0.5 rounded border border-transparent text-xs font-mono focus:outline-none hover:bg-surface-subtle hover:border-border-soft transition-colors"
            on:click=move |_| {
                let next = match direction.get() {
                    SortDirection::Neutral => SortDirection::Ascending,
                    SortDirection::Ascending => SortDirection::Descending,
                    SortDirection::Descending => SortDirection::Neutral,
                };
                direction.set(next);
                if let Some(cb) = on_cycle.as_ref() {
                    cb.run(next);
                }
            }
        >
            <span
                class=move || format!("hf-icon hf-icon-sort {}", dir_class.get())
                aria_hidden="true"
            >
                {move || symbol.get()}
            </span>
        </button>
    }
}
