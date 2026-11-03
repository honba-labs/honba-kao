//! `ScreenerPage` — workstation integrator: owns all filter/sort/selection
//! signals, derives the filtered+sorted record set, and mounts the filter
//! bar, results table, detail drawer, and filter/column modals.

use super::components::{
    ScreenerColumnModal, ScreenerDetailDrawer, ScreenerFilterBar, ScreenerFilterModal,
    ScreenerHeatmap, ScreenerTable,
};
use crate::features::screener::filters::{filter_records, filter_universe, sort_universe};
use crate::features::screener::fixtures::{stock_universe, universe_for};
use crate::features::screener::prompt_ast::{PromptTerm, parse_prompt, prompt_label};
use crate::features::screener::types::{
    COLUMN_CATALOG, DEFAULT_VISIBLE_COLUMNS, FilterCriteria, ScreenerAssetType, ScreenerColumn,
    ScreenerEquityRecord, ScreenerRecord, ScreenerViewMode, all_master_column_keys, catalog_for,
    columns_for_tab,
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
    let selected_index = RwSignal::new(String::new());
    let ai_prompt = RwSignal::new(String::new());
    let ai_label = RwSignal::new(String::new());

    // --- Sorting ---------------------------------------------------------
    let sort_col: RwSignal<&'static str> = RwSignal::new("market_cap_cr");
    let sort_ascending = RwSignal::new(false);

    // --- Watchlist seeded from fixture flags ------------------------------
    let watchlist = RwSignal::new(
        stock_universe()
            .iter()
            .filter(|r| r.is_watchlist)
            .map(|r| r.symbol.clone())
            .collect::<HashSet<String>>(),
    );

    // --- Column flags + master order over every catalog key (doc §6.1) ----
    let master_keys: Vec<&'static str> = all_master_column_keys();
    let column_flags: Vec<(&'static str, RwSignal<bool>)> = master_keys
        .iter()
        .map(|key| {
            // Overview (stock catalog) keys keep today's default seeding;
            // every other tab metric column starts visible.
            let overview = COLUMN_CATALOG.iter().any(|c| c.key == *key);
            let visible = if overview {
                DEFAULT_VISIBLE_COLUMNS.contains(key)
            } else {
                true
            };
            (*key, RwSignal::new(visible))
        })
        .collect();
    let column_order: RwSignal<Vec<&'static str>> = RwSignal::new(master_keys);
    // Ownership copies for the two `move` closures that mount the table
    // (Move-closures capture the Vec by value; RwSignals inside are Copy).
    let flags_for_table_a = column_flags.clone();
    let flags_for_table_b = column_flags.clone();
    let flags_for_modal = column_flags.clone();

    // Ordered visible columns for the active (asset class, tab) pair:
    // master order ∩ class/tab catalog ∩ flags. Overview yields today's
    // default stock columns byte-for-byte.
    let flags_for_view = column_flags.clone();
    let visible_columns: Signal<Vec<ScreenerColumn>> = Memo::new(move |_| {
        let asset = ScreenerAssetType::from_bar(&asset_type.get());
        let slice: &'static [ScreenerColumn] = if asset == ScreenerAssetType::Stocks {
            columns_for_tab(&active_tab.get())
        } else {
            catalog_for(asset)
        };
        let mut out: Vec<ScreenerColumn> = Vec::new();
        for key in column_order.get() {
            let Some(col) = slice.iter().find(|c| c.key == key) else {
                continue;
            };
            let Some((_, flag)) = flags_for_view.iter().find(|(k, _)| *k == key) else {
                continue;
            };
            if flag.get() {
                out.push(*col);
            }
        }
        out
    })
    .into();

    // Shared criteria builder (Copy closure reused by the memo + count).
    let build_criteria = move || FilterCriteria {
        search_query: search_query.get(),
        exchange: selected_exchange.get(),
        sector: selected_sector.get(),
        market_cap_category: selected_market_cap.get(),
        market_index: selected_index.get(),
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

    // String bar value → typed asset class (filter bar stays stringly-typed).
    let typed_asset: Signal<ScreenerAssetType> =
        Memo::new(move |_| ScreenerAssetType::from_bar(&asset_type.get())).into();

    let filtered_records: Memo<Vec<ScreenerRecord>> = Memo::new(move |_| {
        let criteria = build_criteria();
        let base = universe_for(typed_asset.get());
        let filtered = filter_universe(&base, &criteria);
        sort_universe(&filtered, sort_col.get(), sort_ascending.get())
    });
    let records: Signal<Vec<ScreenerRecord>> = filtered_records.into();

    // Heatmap math is stocks-only; other asset classes render no cells.
    let stock_records: Memo<Vec<ScreenerEquityRecord>> = Memo::new(move |_| {
        if typed_asset.get() != ScreenerAssetType::Stocks {
            return Vec::new();
        }
        let criteria = build_criteria();
        filter_records(&stock_universe(), &criteria)
    });

    let active_filters: Signal<usize> = Signal::derive(move || build_criteria().active_count());

    let active_record: Memo<Option<ScreenerRecord>> = Memo::new(move |_| {
        let sym = active_symbol.get();
        universe_for(typed_asset.get())
            .into_iter()
            .find(|r| r.symbol() == sym)
    });
    let record: Signal<Option<ScreenerRecord>> = active_record.into();

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
        selected_index.set(String::new());
        ai_prompt.set(String::new());
        ai_label.set(String::new());
        sort_col.set("market_cap_cr");
        sort_ascending.set(false);
    });

    // ✦ AI bar: deterministic local parse (no honba_nou linkage) applied onto the
    // SAME signals the filter bar / modal already drive — single source of truth.
    let on_ai_apply = Callback::new(move |prompt: String| {
        let ast = parse_prompt(&prompt);
        for term in &ast.terms {
            match term {
                PromptTerm::PriceMin(value) => min_price.set(*value),
                PromptTerm::PriceMax(value) => max_price.set(*value),
                PromptTerm::PeMin(value) => min_pe.set(*value),
                PromptTerm::PeMax(value) => max_pe.set(*value),
                PromptTerm::RvolMin(value) => min_rvol.set(*value),
                PromptTerm::RsiMin(value) => min_rsi.set(*value),
                PromptTerm::RsiMax(value) => max_rsi.set(*value),
                PromptTerm::Index(key) => selected_index.set((*key).to_string()),
                PromptTerm::FnoOnly => fno_only.set(true),
                PromptTerm::WatchlistOnly => watchlist_only.set(true),
            }
        }
        if ast.terms.is_empty() && !ast.residual.is_empty() {
            search_query.set(ast.residual.clone());
        }
        ai_label.set(prompt_label(&prompt));
        ai_prompt.set(String::new());
    });

    // Sector dropdown options: "All" first, then unique sectors A→Z.
    let mut sector_options: Vec<(String, String)> = stock_universe()
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
                selected_index=selected_index
                selected_sector=selected_sector
                sector_options=sector_options
                selected_market_cap=selected_market_cap
                watchlist_only=watchlist_only
                fno_only=fno_only
                view_mode=view_mode
                filter_modal_open=filter_modal_open
                column_modal_open=column_modal_open
                min_price=min_price
                max_price=max_price
                min_pe=min_pe
                max_pe=max_pe
                ai_prompt=ai_prompt
                ai_label=ai_label
                on_ai_apply=on_ai_apply
                active_filters=active_filters
                on_reset=on_reset
            />
            <div class="screener-body flex flex-1 min-h-0 overflow-hidden">
                <div class="screener-main flex-1 min-w-0 overflow-x-auto">
                    {move || match view_mode.get() {
                        ScreenerViewMode::Table => view! {
                            <ScreenerTable
                                records=records
                                columns=visible_columns
                                column_flags=flags_for_table_a.clone()
                                column_order=column_order
                                active_symbol=active_symbol
                                watchlist=watchlist
                                sort_col=sort_col
                                sort_ascending=sort_ascending
                                drawer_open=drawer_open
                                min_price=min_price
                                max_price=max_price
                                min_pe=min_pe
                                max_pe=max_pe
                                min_rvol=min_rvol
                                min_rsi=min_rsi
                                max_rsi=max_rsi
                            />
                        }
                        .into_any(),
                        ScreenerViewMode::Heatmap => {
                            if typed_asset.get() == ScreenerAssetType::Stocks {
                                view! {
                                    <ScreenerHeatmap
                                        records=stock_records.into()
                                        active_symbol=active_symbol
                                        drawer_open=drawer_open
                                    />
                                }
                                .into_any()
                            } else {
                                view! {
                                    <div class="flex h-full flex-col">
                                        <div class="flex items-center border-b border-border-soft px-3 py-1.5 text-[11px] text-text-muted">
                                            "Heatmap view is available for Stocks. Showing the table for this asset class."
                                        </div>
                                        <div class="min-h-0 flex-1 overflow-x-auto">
                                            {view! {
                                                <ScreenerTable
                                                    records=records
                                                    columns=visible_columns
                                                    column_flags=flags_for_table_b.clone()
                                                    column_order=column_order
                                                    active_symbol=active_symbol
                                                    watchlist=watchlist
                                                    sort_col=sort_col
                                                    sort_ascending=sort_ascending
                                                    drawer_open=drawer_open
                                                    min_price=min_price
                                                    max_price=max_price
                                                    min_pe=min_pe
                                                    max_pe=max_pe
                                                    min_rvol=min_rvol
                                                    min_rsi=min_rsi
                                                    max_rsi=max_rsi
                                                />
                                            }
                                            .into_any()}
                                        </div>
                                    </div>
                                }
                                .into_any()
                            }
                        }
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
                                column_flags=flags_for_modal.clone()
                            />
                        }
                    })
            }}
        </div>
    }
}
