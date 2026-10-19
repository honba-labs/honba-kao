//! The universal symbol-search modal — the Leptos port of the labs app's
//! `symbol-search-modal.tsx`, driven by `data/symbol_search.rs` (the pooled
//! universe of indices, instruments and bhavcopy rows).

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::data::symbol_search::{self, SymbolCategory};
use crate::stores::workbench::WorkbenchStore;
use crate::utils::format::percent;

/// The search modal: fixed overlay, the pooled results react to the query.
#[component]
pub fn SymbolSearchModal() -> impl IntoView {
    let store = WorkbenchStore::take();

    let query = RwSignal::new(String::new());
    let active_category = RwSignal::new(SymbolCategory::All);

    // Results recompute on every keystroke / tab flip (sync pool, ~4k rows).
    let results =
        Memo::new(move |_| symbol_search::search(&query.get(), active_category.get(), 50));
    let first_result = Memo::new(move |_| results.get().first().map(|item| item.symbol.clone()));

    // Esc closes; Enter picks the top row. The listener installs while open.
    Effect::new(move |_| {
        if !store.search_open.get() {
            return;
        }
        let close = move |first: Option<String>| {
            if let Some(symbol) = first {
                store.pick_symbol(&symbol);
            } else {
                store.close_symbol_search();
            }
        };

        let handle = window_event_listener_untyped("keydown", move |event: web_sys::Event| {
            let Some(key_event) = event.dyn_into::<web_sys::KeyboardEvent>().ok() else {
                return;
            };
            if key_event.default_prevented() {
                return;
            }
            match key_event.key().as_str() {
                "Escape" => {
                    key_event.prevent_default();
                    store.close_symbol_search();
                }
                "Enter" => {
                    key_event.prevent_default();
                    close(first_result.get_untracked());
                }
                _ => {}
            }
        });
        on_cleanup(move || handle.remove());
    });

    // Autofocus the field when the modal opens.
    let input_ref = NodeRef::<leptos::html::Input>::new();
    Effect::new(move |_| {
        if store.search_open.get()
            && let Some(input) = input_ref.get()
        {
            let _ = input.focus();
        }
    });

    view! {
        <Show when=move || store.search_open.get()>
            <div class="hfui-modal__backdrop" on:click=move |_| store.close_symbol_search()>
                <div
                    class="hfui-modal__container wb-search"
                    on:click=move |event| event.stop_propagation()
                >
                    <div class="hfui-modal__header">
                        <div class="wb-search__title">
                            <span>"Search Symbols"</span>
                            <span class="wb-search__hint">
                                "NSE · BSE · MCX · Crypto · 4,000+ symbols"
                            </span>
                        </div>
                        <button
                            class="hfui-panel__icon-button"
                            title="Close"
                            on:click=move |_| store.close_symbol_search()
                        >
                            "×"
                        </button>
                    </div>

                    <div class="hfui-modal__body">
                        <input
                            node_ref=input_ref
                            class="wb-search__input"
                            type="text"
                            placeholder="Search symbol or name — e.g. RELIANCE, NIFTY, GOLD…"
                            bind:value=query
                        />

                        <div class="wb-search__tabs" role="tablist">
                            {SymbolCategory::ALL
                                .iter()
                                .map(|category| {
                                    let category = *category;
                                    let current = active_category;
                                    view! {
                                        <button
                                            class=move || {
                                                if current.get() == category {
                                                    "wb-pill wb-pill--on"
                                                } else {
                                                    "wb-pill"
                                                }
                                            }
                                            role="tab"
                                            data-category=category.as_id()
                                            on:click=move |_| current.set(category)
                                        >
                                            {category.label()}
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>

                        <div class="wb-search__results" role="listbox">
                            {move || {
                                results.get().into_iter().map(|item| {
                                    let symbol = item.symbol.clone();
                                    let symbol_for_title = symbol.clone();
                                    let symbol_for_click = symbol.clone();
                                    let name = item.name.clone();
                                    let badge = item.type_badge.clone();
                                    let exchange = item.exchange.clone();
                                    let price = format!("₹{:.2}", item.price);
                                    let change_percent = item.change_percent;

                                    view! {
                                        <button
                                            class="wb-search__row"
                                            role="option"
                                            title="Open in this pane"
                                            on:click=move |_| store.pick_symbol(&symbol_for_click)
                                        >
                                            <span class="wb-search__symbol">
                                                <strong>{symbol_for_title}</strong>
                                                <span class="wb-badge">{badge}</span>
                                            </span>
                                            <span class="wb-search__name">{name}</span>
                                            <span class="wb-search__quote tnum">
                                                <span>{price}</span>
                                                <span class=move || {
                                                    if change_percent >= 0.0 {
                                                        "wb-delta wb-delta--up"
                                                    } else {
                                                        "wb-delta wb-delta--down"
                                                    }
                                                }>{percent(change_percent)}</span>
                                            </span>
                                            <span class="wb-search__exchange">{exchange}</span>
                                        </button>
                                    }
                                }).collect_view()
                            }}
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
