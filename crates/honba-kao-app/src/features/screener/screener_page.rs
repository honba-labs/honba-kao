//! `ScreenerPage` — workstation integrator: owns all filter/sort/selection
//! signals, derives the filtered+sorted record set, and mounts the filter
//! bar, results table, detail drawer, and filter/column modals.

use super::components::{
    ScreenerColumnModal, ScreenerDetailDrawer, ScreenerFilterBar, ScreenerFilterModal,
    ScreenerTable,
};
use crate::features::screener::filters::{filter_records, sort_records};
use crate::features::screener::fixtures::seed_fixtures;
use crate::features::screener::types::{
    COLUMN_CATALOG, DEFAULT_VISIBLE_COLUMNS, FilterCriteria, ScreenerEquityRecord, ScreenerViewMode,
};
use leptos::prelude::*;
use rust_decimal_macros::dec;
use std::collections::HashSet;

#[component]
pub fn ScreenerPage() -> impl IntoView {
    // --- Tab / mode / selection state -----------------------------------
    let active_tab = RwSignal::new("overview".to_string());
    let asset_type = RwSignal::new("stocks".to_string());
    let view_mode = RwSignal::new(ScreenerViewMode::Table);
    let active_symbol = RwSignal::new("RELIANCE".to_string());
    let drawer_open = RwSignal::new(false);
    let filter_modal_open = RwSignal::new(false);
    let column_modal_open = RwSignal::new(false);

    // --- Filter state ----------------------------------------------------
    let search_query = RwSignal::new(String::new());
    let selected_exchange = RwSignal::new("ALL".to_string());
    let selected_sector = RwSignal::new("ALL".to_string());
    let selected_market_cap = RwSignal::new("ALL".to_string());
    let fno_only = RwSignal::new(false);
    let watchlist_only = RwSignal::new(false);
    let min_price = RwSignal::new(dec!(0));
    let max_price = RwSignal::new(dec!(0));
    let min_pe = RwSignal::new(dec!(0));
    let max_pe = RwSignal::new(dec!(0));
    let min_rvol = RwSignal::new(dec!(0));
    let min_rsi = RwSignal::new(dec!(0));
    let max_rsi = RwSignal::new(dec!(0));

    // --- Sorting ---------------------------------------------------------
    let sort_col: RwSignal<&'static str> = RwSignal::new("market_cap_cr");
    let sort_ascending = RwSignal::new(false);

    // --- Watchlist seeded from fixture flags ------------------------------
    let watchlist = RwSignal::new(
        seed_fixtures()
            .iter()
            .filter(|r| r.is_watchlist)
            .map(|r| r.symbol.clone())
            .collect::<HashSet<String>>(),
    );

    // --- Column visibility (one signal per catalog column) ----------------
    let column_flags: Vec<(&'static str, RwSignal<bool>)> = COLUMN_CATALOG
        .iter()
        .map(|c| {
            (
                c.key,
                RwSignal::new(DEFAULT_VISIBLE_COLUMNS.contains(&c.key)),
            )
        })
        .collect();
    let flags_for_visibility = column_flags.clone();
    let visible_keys: Memo<Vec<&'static str>> = Memo::new(move |_| {
        flags_for_visibility
            .iter()
            .filter(|(_, f)| f.get())
            .map(|(k, _)| *k)
            .collect()
    });
    let visible_columns: Signal<Vec<&'static str>> = visible_keys.into();

    // Shared criteria builder (Copy closure reused by the memo + count).
    let build_criteria = move || FilterCriteria {
        search_query: search_query.get(),
        exchange: selected_exchange.get(),
        sector: selected_sector.get(),
        market_cap_category: selected_market_cap.get(),
        min_price: min_price.get(),
        max_price: max_price.get(),
        min_pe: min_pe.get(),
        max_pe: max_pe.get(),
        min_rvol: min_rvol.get(),
        min_rsi: min_rsi.get(),
        max_rsi: max_rsi.get(),
        fno_only: fno_only.get(),
        watchlist_only: watchlist_only.get(),
    };

    let filtered_records: Memo<Vec<ScreenerEquityRecord>> = Memo::new(move |_| {
        let criteria = build_criteria();
        let base = seed_fixtures();
        let filtered = filter_records(&base, &criteria);
        sort_records(&filtered, sort_col.get(), sort_ascending.get())
    });
    let records: Signal<Vec<ScreenerEquityRecord>> = filtered_records.into();

    let active_filters: Signal<usize> = Signal::derive(move || build_criteria().active_count());

    let active_record: Memo<Option<ScreenerEquityRecord>> = Memo::new(move |_| {
        let sym = active_symbol.get();
        seed_fixtures().into_iter().find(|r| r.symbol == sym)
    });
    let record: Signal<Option<ScreenerEquityRecord>> = active_record.into();

    let on_reset = Callback::new(move |_| {
        search_query.set(String::new());
        selected_exchange.set("ALL".to_string());
        selected_sector.set("ALL".to_string());
        selected_market_cap.set("ALL".to_string());
        fno_only.set(false);
        watchlist_only.set(false);
        min_price.set(dec!(0));
        max_price.set(dec!(0));
        min_pe.set(dec!(0));
        max_pe.set(dec!(0));
        min_rvol.set(dec!(0));
        min_rsi.set(dec!(0));
        max_rsi.set(dec!(0));
        sort_col.set("market_cap_cr");
        sort_ascending.set(false);
    });

    // Sector dropdown options: "All" first, then unique sectors A→Z.
    let mut sector_options: Vec<(String, String)> = seed_fixtures()
        .iter()
        .map(|r| r.sector.clone())
        .collect::<HashSet<String>>()
        .into_iter()
        .map(|s| (s.clone(), s))
        .collect();
    sector_options.sort_by_key(|s| s.0.to_lowercase());
    sector_options.insert(0, ("All".to_string(), "ALL".to_string()));

    view! {
        <div class="hf-screener-workstation flex flex-col h-full bg-surface-base text-text-high">
            <ScreenerFilterBar
                asset_type=asset_type
                active_tab=active_tab
                search_query=search_query
                selected_exchange=selected_exchange
                selected_sector=selected_sector
                sector_options=sector_options
                selected_market_cap=selected_market_cap
                watchlist_only=watchlist_only
                fno_only=fno_only
                view_mode=view_mode
                filter_modal_open=filter_modal_open
                column_modal_open=column_modal_open
                active_filters=active_filters
                on_reset=on_reset
            />
            <div class="screener-body flex flex-1 min-h-0 overflow-hidden">
                <div class="screener-main flex-1 min-w-0 overflow-x-auto">
                    {move || match view_mode.get() {
                        ScreenerViewMode::Table => view! {
                            <ScreenerTable
                                records=records
                                active_symbol=active_symbol
                                watchlist=watchlist
                                visible_columns=visible_columns
                                sort_col=sort_col
                                sort_ascending=sort_ascending
                                drawer_open=drawer_open
                            />
                        }
                        .into_any(),
                        ScreenerViewMode::Heatmap => view! {
                            <div class="flex h-full items-center justify-center text-xs text-text-muted">
                                "Market Heatmap arrives in Phase 2 (sector treemap)"
                            </div>
                        }
                        .into_any(),
                    }}
                </div>
                {move || {
                    drawer_open
                        .get()
                        .then(|| {
                            view! {
                                <ScreenerDetailDrawer
                                    open=drawer_open
                                    record=record
                                    watchlist=watchlist
                                />
                            }
                        })
                }}
            </div>
            {move || {
                filter_modal_open
                    .get()
                    .then(|| {
                        view! {
                            <ScreenerFilterModal
                                open=filter_modal_open
                                min_price=min_price
                                max_price=max_price
                                min_pe=min_pe
                                max_pe=max_pe
                                min_rvol=min_rvol
                                min_rsi=min_rsi
                                max_rsi=max_rsi
                                fno_only=fno_only
                                watchlist_only=watchlist_only
                                on_reset=on_reset
                            />
                        }
                    })
            }}
            {move || {
                column_modal_open
                    .get()
                    .then(|| {
                        view! {
                            <ScreenerColumnModal
                                open=column_modal_open
                                column_flags=column_flags.clone()
                            />
                        }
                    })
            }}
        </div>
    }
}
