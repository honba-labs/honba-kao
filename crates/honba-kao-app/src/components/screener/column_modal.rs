//! The column configurator — the Leptos port of the labs app's
//! `column-modal.tsx`: "Customize Columns" modal with category filter tabs,
//! a two-column checkbox grid over every column of the active screener type
//! and apply/reset footers (`getPresetForScreenerTab` for the reset path).
//!
//! The `symbol` column is always kept (mirrors the labs `handleToggle`
//! guard); drafts live in a local signal merged in on open, and `Save`
//! publishes through `store.set_visible_columns`.

use leptos::prelude::*;

use crate::stores::screener::{
    ColumnCategory, ScreenerStore, ScreenerType, columns_for_screener, preset_for_tab,
};

/// Canonical category order for the tab strip (`ColumnCategory` display
/// order, matching the labs modal's grouping).
const CATEGORY_TABS: [ColumnCategory; 7] = [
    ColumnCategory::Overview,
    ColumnCategory::Performance,
    ColumnCategory::Valuation,
    ColumnCategory::Technicals,
    ColumnCategory::Fundamentals,
    ColumnCategory::Funds,
    ColumnCategory::Bonds,
];

/// Count of shown classes per tab id (all-tab included) for the tab badge.
fn screener_kind_label(screener_type: ScreenerType) -> &'static str {
    screener_type
        .stock_name()
        .split(' ')
        .next()
        .unwrap_or("Stocks")
}

fn screener_kind_id(screener_type: ScreenerType) -> &'static str {
    screener_type.as_id()
}

#[component]
pub fn ColumnModal() -> impl IntoView {
    let store = ScreenerStore::take();

    // `None` = the All tab, `Some(category)` — one category tab.
    let active_tab = RwSignal::new(None::<ColumnCategory>);
    let draft = RwSignal::new(Vec::<String>::new());

    // Every column of the current screener type, the checkbox grid rows.
    let all_columns = Memo::new(move |_| columns_for_screener(store.screener_type.get()));

    // Filtered rows for the active tab.
    let shown = Memo::new(move |_| {
        let tab = active_tab.get();
        all_columns
            .get()
            .into_iter()
            .filter(|column| tab.is_none_or(|category| column.category == category))
            .collect::<Vec<_>>()
    });

    // Visible-count badge per tab.
    let visible_ids = Memo::new(move |_| {
        store
            .columns()
            .get()
            .into_iter()
            .filter(|(_, visible)| *visible)
            .map(|(column, _)| column.id.to_string())
            .collect::<Vec<String>>()
    });

    // Re-seed the draft each time the modal opens.
    Effect::new(move |_| {
        if store.column_modal.get() {
            let mut preset: Vec<String> = preset_for_tab(
                store.screener_type.get_untracked(),
                store.active_tab.get_untracked(),
            )
            .unwrap_or_else(|| {
                columns_for_screener(store.screener_type.get_untracked())
                    .into_iter()
                    .map(|column| column.id)
                    .collect()
            })
            .into_iter()
            .map(str::to_string)
            .collect();
            if !preset.iter().any(|id| id == "symbol") {
                preset.insert(0, "symbol".to_string());
            }
            draft.set(preset);
            active_tab.set(None);
        }
    });

    let close = move |_| store.column_modal.set(false);

    let toggle = move |id: &'static str| {
        if id == "symbol" {
            return;
        }
        draft.update(|ids| {
            if ids.iter().any(|entry| entry == id) {
                ids.retain(|entry| entry != id);
            } else {
                ids.push(id.to_string());
            }
        });
    };

    let reset_defaults = move |_| {
        let preset: Vec<String> = preset_for_tab(
            store.screener_type.get_untracked(),
            store.active_tab.get_untracked(),
        )
        .unwrap_or_else(|| {
            columns_for_screener(store.screener_type.get_untracked())
                .into_iter()
                .map(|column| column.id)
                .collect()
        })
        .into_iter()
        .map(str::to_string)
        .collect();
        draft.set(preset);
    };

    let apply = move |_| {
        store.set_visible_columns(draft.get_untracked());
        store.column_modal.set(false);
    };

    view! {
        <Show when=move || store.column_modal.get()>
            <div class="hfui-modal__backdrop" on:click=close>
                <div class="hfui-modal__container sc-col-modal" on:click=move |event| event.stop_propagation()>

                    <div class="hfui-modal__header">
                        <div class="sc-col-modal__title">
                            <span>"Customize Columns"</span>
                            <span class="sc-col-modal__kind" data-kind=screener_kind_id(store.screener_type.get_untracked())>
                                {move || screener_kind_label(store.screener_type.get())}
                            </span>
                        </div>
                        <button class="hfui-panel__icon-button" title="Close" on:click=close>
                            "×"
                        </button>
                    </div>

                    <div class="hfui-modal__body">
                        <div class="sc-col-modal__tabs" role="tablist">
                            <button
                                class=move || {
                                    if active_tab.get().is_none() {
                                        "wb-pill wb-pill--on"
                                    } else {
                                        "wb-pill"
                                    }
                                }
                                role="tab"
                                on:click=move |_| active_tab.set(None)
                            >
                                {"All"}
                            </button>
                            {CATEGORY_TABS
                                .iter()
                                .map(|category| {
                                    let category = *category;
                                    let active = move || active_tab.get() == Some(category);
                                    view! {
                                        <button
                                            class=move || {
                                                if active() {
                                                    "wb-pill wb-pill--on"
                                                } else {
                                                    "wb-pill"
                                                }
                                            }
                                            role="tab"
                                            on:click=move |_| active_tab.set(Some(category))
                                        >
                                            {category.label()}
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>

                        <div class="sc-col-modal__grid">
                            {move || {
                                let visible = visible_ids.get();
                                shown.get().into_iter().map(|column| {
                                    let id = column.id;
                                    let locked = id == "symbol";
                                    let checked_ids = visible.clone();
                                    let checked = move || checked_ids.iter().any(|entry| entry == id);
                                    view! {
                                        <label
                                            class=move || {
                                                if locked {
                                                    "sc-col-modal__row sc-col-modal__row--locked"
                                                } else {
                                                    "sc-col-modal__row"
                                                }
                                            }
                                        >
                                            <input
                                                type="checkbox"
                                                prop:checked=checked()
                                                disabled=locked
                                                on:change=move |_| toggle(id)
                                            />
                                            <span class="sc-col-modal__label">{column.label}</span>
                                            <span class="sc-col-modal__id">{id}</span>
                                        </label>
                                    }
                                }).collect_view()
                            }}
                        </div>
                    </div>

                    <div class="hfui-modal__footer sc-col-modal__footer">
                        <span class="sc-col-modal__count">
                            {move || {
                                let shown = shown.get().len();
                                let on = visible_ids.get().len();
                                format!("{on} visible · {shown} listed")
                            }}
                        </span>
                        <span class="sc-col-modal__actions">
                            <button class="wb-btn" on:click=reset_defaults>
                                "Reset"
                            </button>
                            <button class="wb-btn wb-btn--solid" on:click=apply>
                                "Apply"
                            </button>
                            <button class="wb-btn" on:click=close>
                                "Cancel"
                            </button>
                        </span>
                    </div>
                </div>
            </div>
        </Show>
    }
}

/*
 * Classes land in style/screener.css (authored by the integrator):
 * sc-col-modal, sc-col-modal__title, __kind, __tabs, __grid, __row,
 * __row--locked, __label, __id, __footer, __count, __actions.
 */
