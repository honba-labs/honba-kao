//! The screen catalog modal — the Leptos port of the labs app's
//! `open-screen-modal.tsx`: the "Open screen" dialog with search, category
//! tabs (All / Popular / Fundamentals / Technicals / Valuation / My Saved)
//! and the custom-screen badge, listing the built-in screens of the active
//! asset class plus the user's saved screens; picking one publishes it
//! through `store.select_screen`.

use leptos::prelude::*;

use honba_kao_ui_core::HfIconButton;
use honba_kao_ui_core::icons::outline::x_close;

use crate::features::screener::stores::screener_store::{ScreenDefinition, ScreenerStore, builtin_screens_registry};

/// The category tabs (`cat.id` + labels of the labs modal).
const CATEGORIES: [(&str, &str); 6] = [
    ("all", "All Screens"),
    ("popular", "Popular"),
    ("fundamental", "Fundamentals"),
    ("technical", "Technicals"),
    ("valuation", "Valuation"),
    ("custom", "My Saved"),
];

/// Category marker glyph (`getCategoryIcon` colors become token classes).
fn category_class(category: &str) -> &'static str {
    match category {
        "popular" => "sc-screen__icon--popular",
        "fundamental" => "sc-screen__icon--fundamental",
        "technical" => "sc-screen__icon--technical",
        "valuation" => "sc-screen__icon--valuation",
        "custom" => "sc-screen__icon--custom",
        _ => "sc-screen__icon--misc",
    }
}

fn category_glyph(category: &str) -> &'static str {
    match category {
        "popular" => "⚡",
        "Fundamentals" => "≡",
        "fundamental" => "≡",
        "technical" => "▲",
        "valuation" => "₹",
        "custom" => "★",
        _ => "◍",
    }
}

#[component]
pub fn OpenScreenModal() -> impl IntoView {
    let store = ScreenerStore::take();

    let search = RwSignal::new(String::new());
    let category = RwSignal::new("all".to_string());

    // Re-seed on open (labs useEffect: clear search + reset the tab).
    Effect::new(move |_| {
        if store.open_screen_modal.get() {
            search.set(String::new());
            category.set("all".to_string());
        }
    });

    // Builtin + custom screens of the active asset class.
    let asset_screens = Memo::new(move |_| -> Vec<ScreenDefinition> {
        let screener_type = store.screener_type.get();
        let mut screens: Vec<ScreenDefinition> = store
            .custom_screens
            .get()
            .into_iter()
            .filter(|screen| screen.screener_type == screener_type)
            .collect();
        screens.extend(
            builtin_screens_registry()
                .iter()
                .filter(|screen| screen.screener_type == screener_type)
                .cloned(),
        );
        screens
    });

    // Category + search filter.
    let filtered = Memo::new(move |_| {
        let needle = search.get().trim().to_lowercase();
        let tab = category.get();
        asset_screens
            .get()
            .into_iter()
            .filter(|screen| {
                if tab == "custom" && screen.is_builtin {
                    return false;
                }
                if tab != "all" && tab != "custom" && screen.category != tab {
                    return false;
                }
                if !needle.is_empty()
                    && !screen.name.to_lowercase().contains(&needle)
                    && !screen.description.to_lowercase().contains(&needle)
                {
                    return false;
                }
                true
            })
            .collect::<Vec<_>>()
    });

    let close = move |_| store.toggle_open_screen_modal(false);

    view! {
        <Show when=move || store.open_screen_modal.get()>
            <div class="hfui-modal__backdrop" on:click=close>
                <div
                    class="hfui-modal__container sc-screen-modal"
                    on:click=move |event| event.stop_propagation()
                >
                    // Header: title + asset-class tag + close.
                    <div class="hfui-modal__header">
                        <div class="sc-screen-modal__title">
                            <span>"Open screen"</span>
                            <span class="sc-screen-modal__kind">
                                {move || store.screener_type.get().default_title()}
                            </span>
                        </div>
                        <HfIconButton
                            icon=x_close()
                            size=14
                            unstyled=true
                            class="hfui-panel__icon-button"
                            title="Close menu (Esc)"
                            on_click=Callback::new(close)
                        />
                    </div>

                    // Search.
                    <div class="sc-screen-modal__search">
                        <input
                            class="sc-screen-modal__search-input"
                            type="text"
                            placeholder=move || {
                                format!("Search {} screens…", store.screener_type.get().default_title())
                            }
                            prop:value=move || search.get()
                            on:input=move |ev| {
                                let input = event_target::<web_sys::HtmlInputElement>(&ev);
                                search.set(input.value());
                            }
                        />
                        <Show when=move || !search.get().is_empty()>
                            <HfIconButton
                                icon=x_close()
                                size=14
                                unstyled=true
                                class="sc-screen-modal__search-clear"
                                title="Clear search"
                                on_click=Callback::new(move |_| search.set(String::new()))
                            />
                        </Show>
                    </div>

                    // Category tabs.
                    <div class="sc-screen-modal__tabs" role="tablist">
                        {CATEGORIES
                            .iter()
                            .map(|(tab_id, label)| {
                                let tab = *tab_id;
                                let active_tab = tab.to_string();
                                let label = label.to_string();
                                let active = move || category.get() == tab;
                                view! {
                                    <button
                                        class=move || {
                                            if active() {
                                                "sc-screen-modal__tab sc-screen-modal__tab--on"
                                            } else {
                                                "sc-screen-modal__tab"
                                            }
                                        }
                                        role="tab"
                                        on:click=move |_| category.set(active_tab.clone())
                                    >
                                        {label}
                                    </button>
                                }
                            })
                            .collect_view()}
                    </div>

                    // Screen list.
                    <div class="sc-screen-modal__list">
                        <Show when=move || filtered.get().is_empty()>
                            <div class="sc-screen-modal__empty">
                                {move || {
                                    let text = search.get();
                                    if text.trim().is_empty() {
                                        "No screens in this category yet".to_string()
                                    } else {
                                        format!("No screens found matching \"{text}\"")
                                    }
                                }}
                            </div>
                        </Show>
                        {move || {
                            let active_id = store.screen_id.get_untracked();
                            filtered.get().into_iter().map(|screen| {
                                let selected = screen.id == active_id;
                                let screen_for_click = screen.clone();
                                let base = format!(
                                    "sc-screen{}",
                                    if selected { " sc-screen--sel" } else { "" },
                                );
                                let glyph = category_glyph(&screen.category);
                                let icon_class = category_class(&screen.category);
                                let name_class = if selected {
                                    "sc-screen__name sc-screen__name--sel"
                                } else {
                                    "sc-screen__name"
                                };
                                view! {
                                    <button
                                        class=base
                                        title=screen.description.clone()
                                        on:click=move |_| {
                                            store.select_screen(&screen_for_click);
                                            search.set(String::new());
                                        }
                                    >
                                        <span class=format!("sc-screen__icon {icon_class}")>{glyph}</span>
                                        <span class="sc-screen__text">
                                            <span class="sc-screen__name-row">
                                                <span class=name_class>{screen.name.clone()}</span>
                                                <Show when=move || !screen.is_builtin>
                                                    <span class="sc-screen__custom">"Custom"</span>
                                                </Show>
                                            </span>
                                            <span class="sc-screen__desc">{screen.description.clone()}</span>
                                        </span>
                                        <Show when=move || selected>
                                            <span class="sc-screen__check">"✓"</span>
                                        </Show>
                                    </button>
                                }
                            }).collect_view()
                        }}
                    </div>

                    // Footer: count + hotkey hints.
                    <div class="hfui-modal__footer sc-screen-modal__footer">
                        <span class="sc-screen-modal__count">
                            {move || {
                                format!(
                                    "Showing {} screens for {}",
                                    filtered.get().len(),
                                    store.screener_type.get().default_title(),
                                )
                            }}
                        </span>
                        <span class="sc-screen-modal__hints">
                            <span>"Shortcut: "<kbd>"."</kbd>" to open"</span>
                            <span><kbd>"Esc"</kbd>" to close"</span>
                        </span>
                    </div>
                </div>
            </div>
        </Show>
    }
}
