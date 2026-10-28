//! Hero deck — header ribbon (title, meta, search, universe select, saved
//! views, run) and the horizontally scrolling preset shelf. Milestone 3.1 of
//! `plan-screener.md` ("Hero Header, Preset Shelf & Query Configuration
//! Deck"), `screener-design.md` §3.1.

use leptos::prelude::*;

use honba_kao_ui_core::{HfButton, HfButtonVariant, HfInput};

use crate::features::screener::stores::screener_store::{ScreenDefinition, ScreenerStore};

/// Resolve a universe id to its label (falls back to the raw id if the
/// catalog hasn't loaded it — it always has, but keeps this total).
fn universe_label(store: ScreenerStore, id: &str) -> String {
    store
        .universes
        .get()
        .into_iter()
        .find(|universe| universe.id == id)
        .map(|universe| universe.label)
        .unwrap_or_else(|| id.to_string())
}

/// Preset-shelf search predicate (name / category / description),
/// case-insensitive. `needle` is expected pre-lowercased.
fn preset_matches(screen: &ScreenDefinition, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    screen.name.to_lowercase().contains(needle)
        || screen.category.to_lowercase().contains(needle)
        || screen.description.to_lowercase().contains(needle)
}

#[component]
pub fn HeroDeck() -> impl IntoView {
    let store = ScreenerStore::take();
    let preset_search = RwSignal::new(String::new());

    let sorted = store.sorted();
    let matches = Memo::new(move |_| sorted.get().len());
    let exec_ms = Memo::new(move |_| store.results.get().exec_ms);

    view! {
        <section class="sc-hero-deck">
            <div class="sc-hero-deck__header">
                <h1 class="sc-hero-deck__title">"Equity screener"</h1>
                <div class="sc-hero-deck__meta">
                    {move || {
                        format!(
                            "{} matches · {} · {} · {} ms",
                            matches.get(),
                            store.screen_title.get(),
                            universe_label(store, &store.selected_universe_id.get()),
                            exec_ms.get(),
                        )
                    }}
                </div>
            </div>

            <div class="sc-hero-deck__controls">
                <HfInput
                    class="sc-search-input"
                    r#type="text"
                    placeholder="Search screens…"
                    prop:value=move || preset_search.get()
                    on_input=Callback::new(move |ev| {
                        let input = event_target::<web_sys::HtmlInputElement>(&ev);
                        preset_search.set(input.value());
                    })
                />

                <select
                    class="sc-universe-select"
                    title="Target scan universe"
                    prop:value=move || store.selected_universe_id.get()
                    on:change=move |ev| {
                        let select = event_target::<web_sys::HtmlSelectElement>(&ev);
                        store.selected_universe_id.set(select.value());
                    }
                >
                    {move || {
                        store
                            .universes
                            .get()
                            .into_iter()
                            .map(|universe| {
                                view! {
                                    <option value=universe.id.clone()>{universe.label.clone()}</option>
                                }
                            })
                            .collect_view()
                    }}
                </select>

                <div class="sc-saved-views-control">
                    <HfButton
                        class="sc-saved-views-control__save"
                        title="Save the current filters, tab and view mode as a named view"
                        on_click=Callback::new(move |_| {
                            if let Some(window) = web_sys::window()
                                && let Ok(Some(name)) = window.prompt_with_message("Name this view:")
                            {
                                let trimmed = name.trim().to_string();
                                if !trimmed.is_empty() {
                                    store.save_current_view(&trimmed);
                                }
                            }
                        })
                    >
                        "Save view"
                    </HfButton>
                    <Show when=move || !store.saved_views.get().is_empty()>
                        <select
                            class="sc-saved-views-control__restore"
                            title="Restore a saved view"
                            prop:value=""
                            on:change=move |ev| {
                                let select = event_target::<web_sys::HtmlSelectElement>(&ev);
                                let id = select.value();
                                if let Some(view) = store
                                    .saved_views
                                    .get_untracked()
                                    .into_iter()
                                    .find(|view| view.id == id)
                                {
                                    store.restore_view(&view);
                                    store.run_screen();
                                }
                            }
                        >
                            <option value="" disabled=true>"Restore view…"</option>
                            {move || {
                                store
                                    .saved_views
                                    .get()
                                    .into_iter()
                                    .map(|view| {
                                        view! { <option value=view.id.clone()>{view.name.clone()}</option> }
                                    })
                                    .collect_view()
                            }}
                        </select>
                    </Show>
                </div>

                <HfButton
                    class="sc-hero-deck__run"
                    variant=HfButtonVariant::Green
                    title="Run the current screen"
                    on_click=Callback::new(move |_| store.run_screen())
                >
                    "▶ Run screen"
                </HfButton>
            </div>

            <div class="sc-preset-shelf">
                {move || {
                    let needle = preset_search.get().trim().to_lowercase();
                    let active_id = store.selected_preset_id.get();
                    store
                        .presets
                        .get()
                        .into_iter()
                        .filter(|screen| preset_matches(screen, &needle))
                        .map(|screen| {
                            let preset_id = screen.id.clone();
                            let is_active = active_id.as_deref() == Some(screen.id.as_str());
                            view! {
                                <button
                                    class="sc-preset-card"
                                    data-active=is_active.to_string()
                                    on:click=move |_| store.select_preset(&preset_id)
                                >
                                    <span class="sc-preset-card__name">{screen.name.clone()}</span>
                                    <span class="sc-preset-card__category">{screen.category.clone()}</span>
                                </button>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </section>
    }
}
