//! The indicator-picker dialog — the Leptos port of the labs app's
//! `indicator-picker-dialog.tsx`: search + category tabs (All / Trend /
//! Volume / Volatility / Momentum / Other / Active / Favourites), favourites
//! (★ backed by localStorage) and an "Active on chart" count.
//!
//! Adding only records the pick in the store under the pane id; the pane
//! chart effect performs the engine `addIndicator` (and re-plays it after
//! every remount), so the dialog never touches chart handles.

use std::sync::atomic::{AtomicU64, Ordering};

use leptos::prelude::*;

use crate::features::workbench::stores::layout_store::AppliedIndicator;
use crate::features::workbench::stores::workbench_store::WorkbenchStore;
use crate::shared::components::charts::openalgo_chart::IndicatorDef;

/// Minted instance ids so the engine can own per-instance state and the
/// chips keep working across pane remounts.
static INSTANCE_SEQ: AtomicU64 = AtomicU64::new(1);

fn next_instance_id(pane_id: &str, def_id: &str) -> String {
    format!(
        "{pane_id}:{def_id}:{}",
        INSTANCE_SEQ.fetch_add(1, Ordering::Relaxed)
    )
}

/// The canonical category tabs, in the labs dialog's order (`Other` bins
/// everything the five R&D groups don't name).
const CATEGORIES: [&str; 5] = ["Trend", "Volume", "Volatility", "Momentum", "Other"];

/// Tabs beyond "All": derived from the catalog, plus per-marker views.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    All,
    Category(usize),
    Active,
    Favourites,
}

impl Tab {
    const LIST: [Tab; 8] = [
        Tab::All,
        Tab::Category(0),
        Tab::Category(1),
        Tab::Category(2),
        Tab::Category(3),
        Tab::Category(4),
        Tab::Active,
        Tab::Favourites,
    ];

    pub fn label(self) -> String {
        match self {
            Tab::All => "All".to_string(),
            Tab::Category(index) => CATEGORIES
                .get(index)
                .map_or_else(|| "Other".to_string(), |c| (*c).to_string()),
            Tab::Active => "Active".to_string(),
            Tab::Favourites => "Favourites".to_string(),
        }
    }

    pub fn as_id(self) -> String {
        match self {
            Tab::All => "all".to_string(),
            Tab::Category(index) => CATEGORIES
                .get(index)
                .unwrap_or(&"Other")
                .to_lowercase()
                .replace(' ', "-"),
            Tab::Active => "active".to_string(),
            Tab::Favourites => "favourites".to_string(),
        }
    }
}

#[component]
pub fn IndicatorPickerDialog() -> impl IntoView {
    let store = WorkbenchStore::take();

    let query = RwSignal::new(String::new());
    let active_tab = RwSignal::new(Tab::All);

    // The catalog the bridge returned (fetched when the dialog opened).
    let catalog = store.layout.picker_catalog;

    // Registry rows filtered by the tab + query.
    let visible = Memo::new(move |_| {
        let Some(defs) = catalog.get() else {
            return Vec::with_capacity(0);
        };
        let tab = active_tab.get();
        let query = query.get().trim().to_lowercase();
        let wants = match tab {
            Tab::All | Tab::Active | Tab::Favourites => None,
            Tab::Category(index) => CATEGORIES.get(index).map(|c| c.to_string()),
        };

        defs.iter()
            .filter(|definition| match &wants {
                Some(category) => {
                    CATEGORIES.contains(&definition.category.as_str())
                        && &definition.category == category
                }
                None => true,
            })
            .filter(|definition| {
                query.is_empty()
                    || definition.name.to_lowercase().contains(&query)
                    || definition.id.contains(&query)
            })
            .filter(|definition| match tab {
                Tab::Favourites => store
                    .layout
                    .fav_indicators
                    .get()
                    .iter()
                    .any(|id| *id == definition.id),
                _ => true,
            })
            .cloned()
            .collect::<Vec<IndicatorDef>>()
    });

    // The pane the picker serves and its live applied list (both track the
    // `picker_target` signal, so the header count stays honest).
    let applied = Memo::new(move |_| {
        let pane_id = format!("pane-{}", store.layout.picker_target.get());
        store
            .layout
            .applied_indicators
            .get()
            .iter()
            .find(|(id, _)| *id == pane_id)
            .map_or_else(Vec::new, |(_, list)| list.clone())
    });
    let applied_def_ids = Memo::new(move |_| {
        applied
            .get()
            .iter()
            .map(|item| item.indicator_id.clone())
            .collect::<Vec<String>>()
    });

    let add_indicator = move |definition: IndicatorDef| {
        let pane_id = format!("pane-{}", store.layout.picker_target.get_untracked());
        let def_id = definition.id.clone();
        store.layout.add_pane_indicator(
            &pane_id,
            AppliedIndicator {
                instance_id: next_instance_id(&pane_id, &def_id),
                indicator_id: def_id,
                name: definition.name.clone(),
                placement: definition.placement.clone(),
            },
        );
    };

    view! {
        <Show when=move || store.layout.picker_open.get()>
            <div class="hfui-modal__backdrop" on:click=move |_| store.layout.close_indicator_picker()>
                <div
                    class="hfui-modal__container wb-picker"
                    on:click=move |event| event.stop_propagation()
                >
                    <div class="hfui-modal__header">
                        <div class="wb-picker__title">
                            <span>"Indicators"</span>
                            <span class="wb-picker__hint">
                                {move || format!("pane {}", pane_index_label(store.layout.picker_target.get()))}
                                {" · "}
                                {move || format!("{} active", applied.get().len())}
                            </span>
                        </div>
                        <button
                            class="hfui-panel__icon-button"
                            title="Close"
                            on:click=move |_| store.layout.close_indicator_picker()
                        >
                            "×"
                        </button>
                    </div>

                    <div class="hfui-modal__body">
                        <input
                            class="wb-picker__input"
                            type="text"
                            placeholder="Search indicators (RSI, MACD, Supertrend…)"
                            bind:value=query
                        />

                        <div class="wb-picker__tabs" role="tablist">
                            {Tab::LIST
                                .iter()
                                .map(|tab| {
                                    let tab = *tab;
                                    let current = active_tab;
                                    view! {
                                        <button
                                            class=move || {
                                                if current.get() == tab {
                                                    "wb-pill wb-pill--on"
                                                } else {
                                                    "wb-pill"
                                                }
                                            }
                                            role="tab"
                                            data-category=tab.as_id()
                                            on:click=move |_| current.set(tab)
                                        >
                                            {tab.label()}
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>

                        <div class="wb-picker__results">
                            {move || {
                                let def_ids = applied_def_ids.get();
                                visible.get().into_iter().map(|definition| {
                                    let def_id = definition.id.clone();
                                    let def_id_for_fav = def_id.clone();
                                    let def_id_for_click = def_id.clone();
                                    let name = definition.name.clone();
                                    let category = definition_category(&definition);
                                    let on_chart = def_ids.iter().any(|id| *id == def_id);

                                    view! {
                                        <div class="wb-picker__row" class:wb-picker__row--on=on_chart>
                                            <button
                                                class="wb-picker__fav"
                                                title="Favourite"
                                                on:click=move |_| store.layout.toggle_fav_indicator(&def_id_for_click)
                                            >
                                                {move || {
                                                    if store.layout.fav_indicators.get().iter().any(|id| *id == def_id_for_fav) {
                                                        "★"
                                                    } else {
                                                        "☆"
                                                    }
                                                }}
                                            </button>
                                            <button
                                                class="wb-picker__add"
                                                title="Add to the active chart"
                                                disabled=definition.id.is_empty().then_some(true)
                                                on:click=move |_| add_indicator(definition.clone())
                                            >
                                                <span class="wb-picker__name">{name.clone()}</span>
                                                <span class="wb-picker__category">{category}</span>
                                            </button>
                                        </div>
                                    }
                                }).collect_view()
                            }}
                        </div>

                        <div class="wb-picker__recents">
                            <span class="wb-picker__eyebrow">"Recents"</span>
                            <div class="wb-picker__recent-chips">
                                {move || {
                                    store.layout.recent_indicators.get().iter().map(|id| {
                                        let id = id.clone();
                                        view! { <span class="wb-badge">{id}</span> }
                                    }).collect_view()
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}

/// One-based pane number for the header hint.
fn pane_index_label(target: usize) -> usize {
    target + 1
}

/// The catalog category, normalised to the five tabs (else "Other").
fn definition_category(definition: &IndicatorDef) -> String {
    if CATEGORIES.contains(&definition.category.as_str()) {
        definition.category.clone()
    } else {
        "Other".to_string()
    }
}
