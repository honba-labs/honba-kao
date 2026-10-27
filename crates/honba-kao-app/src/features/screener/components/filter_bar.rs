//! The screener filter bar — the Leptos port of the labs app's
//! `filter-bar.tsx`: asset-class pills, "Screens …" trigger (with the
//! active screen title), Filters / Columns buttons with count badges, the
//! Table/Heatmap view toggle, the quick-preset + market row, the row-3
//! status strip ("All N / Filtered M", shortlist chip, Export CSV).

use honba_kao_ui_core::{
    HfButton, HfInput, HfSegmentedControl, HfSegmentedControlItem, HfToolbar, HfToolbarSpacer,
};
use leptos::prelude::*;

use crate::features::screener::components::drawers::IndexDrawer;
use crate::features::screener::stores::columns::{ALL_SCREENER_TYPES, ScreenerType};
use crate::features::screener::stores::filter_engine::{QUICK_PRESETS, count_active_filters};
use crate::features::screener::stores::screener_store::{ScreenerStore, ViewMode};

/// Pill label per asset class.
fn type_label(screener_type: ScreenerType) -> &'static str {
    match screener_type {
        ScreenerType::Stocks => "Stocks",
        ScreenerType::Etf => "ETFs",
        ScreenerType::Bonds => "Bonds",
        ScreenerType::Mf => "Mutual Funds",
    }
}

/// Asset class pill for the segmented control.
fn asset_class_pill(screener_type: ScreenerType, active: bool) -> impl IntoView {
    view! {
        <HfSegmentedControlItem
            class="hf-segmented-control__item"
            selected=active
        >
            {type_label(screener_type)}
        </HfSegmentedControlItem>
    }
}

/// Quick preset pill.
fn quick_preset_pill(_preset_id: String, preset_label: String, active: bool) -> impl IntoView {
    view! {
        <HfSegmentedControlItem
            class="hf-segmented-control__item"
            selected=active
        >
            {preset_label}
        </HfSegmentedControlItem>
    }
}

/// Market tab pill.
fn market_tab_pill(market: &'static str, active: bool) -> impl IntoView {
    view! {
        <HfSegmentedControlItem
            class="hf-segmented-control__item"
            selected=active
        >
            {market}
        </HfSegmentedControlItem>
    }
}

#[component]
pub fn FilterBar() -> impl IntoView {
    let store = ScreenerStore::take();

    let total = Memo::new(move |_| store.universe().len());
    let filtered = store.filtered();
    let filtered_len = Memo::new(move |_| filtered.get().len());
    let filter_count = Memo::new(move |_| count_active_filters(&store.advanced_filters.get()));
    let market_tabs: [&'static str; 2] = ["IN", "US"];
    let index_drawer_open = RwSignal::new(false);
    let index_drawer_host = NodeRef::<leptos::html::Div>::new();

    view! {
        <HfToolbar class="filter-bar" direction="row">
            <HfToolbarSpacer class="filter-bar__spacer" />

            // --- Row 1: asset classes + screen + tool triggers + view toggle ---
            <div class="filter-bar__row">
                {ALL_SCREENER_TYPES
                    .iter()
                    .map(|screener_type| {
                        let screener_type = *screener_type;
                        asset_class_pill(screener_type, store.screener_type.get() == screener_type)
                    })
                    .collect_view()}

                <HfButton
                    class="filter-bar__screens"
                    title="Open the screen catalog"
                    on_click=Callback::new(move |_| store.toggle_open_screen_modal(true))
                >
                    <span class="filter-bar__screens-label">"Screens"</span>
                    <span class="filter-bar__screens-title">
                        {move || store.screen_title.get()}
                    </span>
                </HfButton>

                <HfButton
                    class="filter-bar__tool"
                    title="Open the columns configurator"
                    on_click=Callback::new(move |_| store.column_modal.set(true))
                >
                    "Columns"
                    <HfSegmentedControlItem
                        class="filter-bar__badge"
                    >
                        {move || store.visible_columns.get().len()}
                    </HfSegmentedControlItem>
                </HfButton>

                <HfButton
                    class="filter-bar__tool"
                    title="Open the filters modal"
                    on_click=Callback::new(move |_| store.filter_modal.set(true))
                >
                    "Filters"
                    <HfSegmentedControlItem
                        class="filter-bar__badge"
                    >
                        {move || filter_count.get()}
                    </HfSegmentedControlItem>
                </HfButton>

                <HfSegmentedControl>
                    <HfSegmentedControlItem
                        class="filter-bar__seg-btn"
                        selected=store.view_mode.get() == ViewMode::Table
                    >
                        "Table"
                    </HfSegmentedControlItem>
                    <HfSegmentedControlItem
                        class="filter-bar__seg-btn"
                        selected=store.view_mode.get() == ViewMode::Heatmap
                    >
                        "Heatmap"
                    </HfSegmentedControlItem>
                    <HfSegmentedControlItem
                        class="filter-bar__seg-btn"
                        selected=store.view_mode.get() == ViewMode::FactorHeatmap
                    >
                        "Factors"
                    </HfSegmentedControlItem>
                </HfSegmentedControl>
            </div>

            // --- Row 2: search + quick presets + market + live ---
            <div class="filter-bar__row">
                <HfInput
                    class="filter-bar__search"
                    r#type="text"
                    placeholder="Search symbols, companies, sectors…"
                    prop:value=move || store.search_query.get()
                    on_input=Callback::new(move |ev| {
                        let input = event_target::<web_sys::HtmlInputElement>(&ev);
                        store.set_search_query(input.value());
                    })
                />

                {QUICK_PRESETS
                    .iter()
                    .map(|(preset_id, preset_label)| {
                        let preset_id = *preset_id;
                        quick_preset_pill(
                            preset_id.to_string(),
                            preset_label.to_string(),
                            store.quick_preset.get() == preset_id,
                        )
                    })
                    .collect_view()}

                {market_tabs
                    .iter()
                    .map(|market| {
                        let market = *market;
                        market_tab_pill(market, store.market.get() == market)
                    })
                    .collect_view()}

                // Index multi-select dropdown (labs IndexDrawer).
                <div class="filter-bar__index-anchor">
                    <HfSegmentedControlItem
                        class="filter-bar__market"
                        selected=index_drawer_open.get()
                    >
                        "Index"
                        <span class="filter-bar__badge">{move || 0}</span>
                    </HfSegmentedControlItem>
                    <IndexDrawer open=index_drawer_open host=index_drawer_host />
                </div>

                <HfButton
                    class="filter-bar__live"
                    title="Toggle the simulated live tick badge"
                    on_click=Callback::new(move |_| store.toggle_live())
                >
                    {move || {
                        if store.is_live.get() { "Live" } else { "Paused" }
                    }}
                </HfButton>
            </div>

            // --- Row 3: counts + watchlist + export ---
            <div class="filter-bar__row">
                <span class="filter-bar__status">
                    "All "
                    {move || total.get()}
                    " / Filtered "
                    {move || filtered_len.get()}
                </span>
                <span
                    class=move || {
                        if store.shortlist.get().is_empty() {
                            "hf-chip"
                        } else {
                            "hf-chip hf-chip--on"
                        }
                    }
                >
                    "★ "
                    {move || store.shortlist.get().len()}
                </span>
                <HfToolbarSpacer class="filter-bar__spacer" />
                <HfButton
                    class="filter-bar__export"
                    title="Download the current screen as CSV"
                    on_click=Callback::new(move |_| store.export_csv())
                >
                    "Export CSV"
                </HfButton>
            </div>
        </HfToolbar>
    }
}
