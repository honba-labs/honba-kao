//! The screener filter bar — the Leptos port of the labs app's
//! `filter-bar.tsx`: asset-class pills, "Screens …" trigger (with the
//! active screen title), Filters / Columns buttons with count badges, the
//! Table/Heatmap view toggle, the quick-preset + market row, the row-3
//! status strip ("All N / Filtered M", shortlist chip, Export CSV).

use leptos::prelude::*;

use crate::components::screener::drawers::IndexDrawer;
use crate::stores::screener::{
    ALL_SCREENER_TYPES, QUICK_PRESETS, ScreenerStore, ScreenerType, ViewMode, count_active_filters,
};

/// Pill label per asset class.
fn type_label(screener_type: ScreenerType) -> &'static str {
    match screener_type {
        ScreenerType::Stocks => "Stocks",
        ScreenerType::Etf => "ETFs",
        ScreenerType::Bonds => "Bonds",
        ScreenerType::Mf => "Mutual Funds",
    }
}

#[component]
pub fn FilterBar() -> impl IntoView {
    let store = ScreenerStore::take();

    let total = Memo::new(move |_| store.universe().len());
    let filtered = store.filtered();
    let filtered_len = Memo::new(move |_| filtered.get().len());
    let filter_count = Memo::new(move |_| count_active_filters(&store.advanced_filters.get()));

    // Index pill (labs `IndexDrawer` trigger) + selected count badge.
    let index_open = RwSignal::new(false);
    let index_host = NodeRef::<leptos::html::Div>::new();
    let index_count = Memo::new(move |_| store.advanced_filters.get().indices.len());

    let market_tabs: [&'static str; 2] = ["IN", "US"];

    view! {
        <div class="sc-filter-bar">
            // --- Row 1: asset classes + screen + tool triggers + view toggle
            <div class="sc-filter-bar__row">
                {ALL_SCREENER_TYPES
                    .iter()
                    .map(|screener_type| {
                        let screener_type = *screener_type;
                        let active = move || store.screener_type.get() == screener_type;
                        view! {
                            <button
                                class=move || {
                                    if active() {
                                        "sc-filter-bar__type sc-filter-bar__type--on"
                                    } else {
                                        "sc-filter-bar__type"
                                    }
                                }
                                title=screener_type.stock_name()
                                on:click=move |_| store.set_screener_type(screener_type)
                            >
                                {type_label(screener_type)}
                            </button>
                        }
                    })
                    .collect_view()}

                <button
                    class="sc-filter-bar__screens"
                    title="Open the screen catalog"
                    on:click=move |_| store.toggle_open_screen_modal(true)
                >
                    <span class="sc-filter-bar__screens-label">"Screens"</span>
                    <span class="sc-filter-bar__screens-title">
                        {move || store.screen_title.get()}
                    </span>
                </button>

                <span class="sc-filter-bar__spacer"></span>

                <button
                    class="sc-filter-bar__tool"
                    title="Open the columns configurator"
                    on:click=move |_| store.column_modal.set(true)
                >
                    "Columns"
                    <span class="sc-filter-bar__badge">
                        {move || store.visible_columns.get().len()}
                    </span>
                </button>

                <button
                    class=move || {
                        if filter_count.get() == 0 {
                            "sc-filter-bar__tool"
                        } else {
                            "sc-filter-bar__tool sc-filter-bar__tool--fair"
                        }
                    }
                    title="Open the filters modal"
                    on:click=move |_| store.filter_modal.set(true)
                >
                    "Filters"
                    <span class="sc-filter-bar__badge">{move || filter_count.get()}</span>
                </button>

                <div class="sc-filter-bar__seg">
                    <button
                        class=move || {
                            if store.view_mode.get() == ViewMode::Table {
                                "sc-filter-bar__seg-btn sc-filter-bar__seg-btn--on"
                            } else {
                                "sc-filter-bar__seg-btn"
                            }
                        }
                        on:click=move |_| store.view_mode.set(ViewMode::Table)
                    >
                        "Table"
                    </button>
                    <button
                        class=move || {
                            if store.view_mode.get() == ViewMode::Heatmap {
                                "sc-filter-bar__seg-btn sc-filter-bar__seg-btn--on"
                            } else {
                                "sc-filter-bar__seg-btn"
                            }
                        }
                        on:click=move |_| store.view_mode.set(ViewMode::Heatmap)
                    >
                        "Heatmap"
                    </button>
                </div>
            </div>

            // --- Row 2: search + quick presets + market + live
            <div class="sc-filter-bar__row">
                <input
                    class="sc-filter-bar__search"
                    type="text"
                    placeholder="Search symbols, companies, sectors…"
                    prop:value=move || store.search_query.get()
                    on:input=move |ev| {
                        let input = event_target::<web_sys::HtmlInputElement>(&ev);
                        store.set_search_query(input.value());
                    }
                />

                {QUICK_PRESETS
                    .iter()
                    .map(|(preset_id, preset_label)| {
                        let preset_id = *preset_id;
                        let active = move || store.quick_preset.get() == preset_id;
                        view! {
                            <button
                                class=move || {
                                    if active() {
                                        "sc-filter-bar__type sc-filter-bar__type--on"
                                    } else {
                                        "sc-filter-bar__type"
                                    }
                                }
                                on:click=move |_| store.set_quick_preset(preset_id)
                            >
                                {preset_label.to_string()}
                            </button>
                        }
                    })
                    .collect_view()}

                {market_tabs
                    .iter()
                    .map(|market| {
                        let market = *market;
                        let active = move || store.market.get() == market;
                        view! {
                            <button
                                class=move || {
                                    if active() {
                                        "sc-filter-bar__market sc-filter-bar__market--on"
                                    } else {
                                        "sc-filter-bar__market"
                                    }
                                }
                                on:click=move |_| store.set_market(market)
                            >
                                {market}
                            </button>
                        }
                    })
                    .collect_view()}

                // Index multi-select dropdown (labs IndexDrawer).
                <div class="sc-filter-bar__index-anchor" node_ref=index_host>
                    <button
                        class=move || {
                            if index_open.get() || index_count.get() > 0 {
                                "sc-filter-bar__market sc-filter-bar__market--on"
                            } else {
                                "sc-filter-bar__market"
                            }
                        }
                        title="Filter instruments by benchmark or sectoral index"
                        on:click=move |_| index_open.update(|open| *open = !*open)
                    >
                        "Index"
                        <span class="sc-filter-bar__badge">{move || index_count.get()}</span>
                    </button>
                    <IndexDrawer open=index_open host=index_host />
                </div>

                <button
                    class=move || {
                        if store.is_live.get() {
                            "sc-filter-bar__live sc-filter-bar__live--on"
                        } else {
                            "sc-filter-bar__live"
                        }
                    }
                    title="Toggle the simulated live tick badge"
                    on:click=move |_| store.toggle_live()
                >
                    {move || {
                        if store.is_live.get() { "Live" } else { "Paused" }
                    }}
                </button>
            </div>

            // --- Row 3: counts + watchlist + export
            <div class="sc-filter-bar__row">
                <span class="sc-filter-bar__status">
                    "All "
                    {move || total.get()}
                    " / Filtered "
                    {move || filtered_len.get()}
                </span>
                <span
                    class=move || {
                        if store.shortlist.get().is_empty() {
                            "sc-filter-bar__chip"
                        } else {
                            "sc-filter-bar__chip sc-filter-bar__chip--on"
                        }
                    }
                    title="Shortlisted symbols"
                >
                    "★ "
                    {move || store.shortlist.get().len()}
                </span>
                <span class="sc-filter-bar__spacer"></span>
                <button
                    class="sc-filter-bar__export"
                    title="Download the current screen as CSV"
                    on:click=move |_| store.export_csv()
                >
                    "Export CSV"
                </button>
            </div>
        </div>
    }
}
