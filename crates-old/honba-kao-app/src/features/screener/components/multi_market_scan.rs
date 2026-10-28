//! Multi-market scan panel — a universe checklist over `store.universes`
//! that runs `store.run_multi_market_scan` against the checked ids.
//! Milestone 3.5 of `plan-screener.md`.

use leptos::prelude::*;

use honba_kao_ui_core::{HfButton, HfButtonVariant, HfCheckbox};

use crate::features::screener::stores::screener_store::ScreenerStore;

/// Toggle one universe id's membership in a selection list, returning the
/// updated list. Pure; unit-tested in `tests/screener_components_test.rs`.
pub fn toggle_universe_selection(selected: &[String], id: &str) -> Vec<String> {
    if selected.iter().any(|existing| existing == id) {
        selected
            .iter()
            .filter(|existing| *existing != id)
            .cloned()
            .collect()
    } else {
        let mut next = selected.to_vec();
        next.push(id.to_string());
        next
    }
}

#[component]
pub fn MultiMarketScanPanel() -> impl IntoView {
    let store = ScreenerStore::take();
    let selected = RwSignal::new(vec![store.selected_universe_id.get_untracked()]);

    view! {
        <div class="sc-multi-market-scan">
            <div class="sc-multi-market-scan__list">
                {move || {
                    let current = selected.get();
                    store
                        .universes
                        .get()
                        .into_iter()
                        .map(|universe| {
                            let id = universe.id.clone();
                            let is_checked = current.iter().any(|existing| existing == &id);
                            view! {
                                <HfCheckbox
                                    class="sc-multi-market-scan__item"
                                    checked=is_checked
                                    on_change=Callback::new(move |_: web_sys::Event| {
                                        selected.update(|list| *list = toggle_universe_selection(list, &id));
                                    })
                                >
                                    {universe.label.clone()}
                                </HfCheckbox>
                            }
                        })
                        .collect_view()
                }}
            </div>
            <HfButton
                class="sc-multi-market-scan__run"
                variant=HfButtonVariant::Green
                title="Run the scan across every checked universe"
                on_click=Callback::new(move |_| store.run_multi_market_scan(selected.get_untracked()))
            >
                "Run multi-market scan"
            </HfButton>
        </div>
    }
}
