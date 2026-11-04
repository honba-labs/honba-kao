//! `ScreenerPage` — Futuristic Neural AST Screener workstation integrator:
//! owns all filter/sort/selection signals, derives the filtered+sorted record
//! set and live `FunnelTelemetry`, and mounts the AST HUD filter bar,
//! multi-modal viewport (`Matrix`, `Constellation`, `Table`, `Heatmap`),
//! Target Inspector drawer, and filter/column/catalog modals.

use super::components::{
    ScreenCatalogModal, ScreenerColumnModal, ScreenerConstellation, ScreenerDetailDrawer,
    ScreenerFilterBar, ScreenerFilterModal, ScreenerHeatmap, ScreenerSignalMatrix, ScreenerTable,
};
use crate::features::screener::csv_export::{csv_document, csv_headers, csv_rows};
use crate::features::screener::filters::{filter_records, filter_universe, sort_universe};
use crate::features::screener::fixtures::{stock_universe, universe_for};
use crate::features::screener::keyboard::{ScreenerKeyAction, use_screener_shortcuts};
use crate::features::screener::prompt_ast::{PromptTerm, parse_prompt, prompt_label};
use crate::features::screener::screen_catalog::{CustomScreen, snapshot_screen, upsert_screen};
#[cfg(target_arch = "wasm32")]
use crate::features::screener::screen_catalog::{
    SCREENS_KEY, deserialize_screens, serialize_screens,
};
use crate::features::screener::telemetry::{FunnelTelemetry, compute_funnel};
use crate::features::screener::types::{
    COLUMN_CATALOG, DEFAULT_VISIBLE_COLUMNS, FilterCriteria, ScreenerAssetType, ScreenerColumn,
    ScreenerEquityRecord, ScreenerRecord, ScreenerViewMode, all_master_column_keys, catalog_for,
    columns_for_tab,
};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use rust_decimal_macros::dec;
use std::collections::HashSet;

#[cfg(target_arch = "wasm32")]
fn load_custom_screens() -> Vec<CustomScreen> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(SCREENS_KEY).ok().flatten())
        .map(|raw| deserialize_screens(&raw))
        .unwrap_or_default()
}
#[cfg(not(target_arch = "wasm32"))]
fn load_custom_screens() -> Vec<CustomScreen> {
    Vec::new()
}

fn persist_custom_screens(screens: &[CustomScreen]) {
    #[cfg(target_arch = "wasm32")]
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if storage
            .set_item(SCREENS_KEY, &serialize_screens(screens))
            .is_err()
        {
            tracing::warn!("screener screens: storage full");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = &screens;
        tracing::info!("screener screens: batch updated (native, no storage)");
    }
}

fn sort_key_for(name: &str, current: &'static str) -> &'static str {
    match name {
        "symbol" => "symbol",
        "price" => "price",
        "change_pct" => "change_pct",
        "volume" => "volume",
        "market_cap_cr" => "market_cap_cr",
        "pe_ratio" => "pe_ratio",
        "rsi_14" => "rsi_14",
        _ => current,
    }
}

#[component]
pub fn ScreenerPage() -> impl IntoView {
    // --- Tab / mode / selection state -----------------------------------
    let active_tab = RwSignal::new("overview".to_string());
    let asset_type = RwSignal::new("stocks".to_string());
    let view_mode = RwSignal::new(ScreenerViewMode::Matrix);
    let active_symbol = RwSignal::new("RELIANCE".to_string());
    let drawer_open = RwSignal::new(true);
    let filter_modal_open = RwSignal::new(false);
    let column_modal_open = RwSignal::new(false);
    let catalog_modal_open = RwSignal::new(false);
    let save_sheet_open = RwSignal::new(false);
    let custom_screens = RwSignal::new(load_custom_screens());
    let active_screen_id = RwSignal::new("builtin.all".to_string());

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
    let flags_for_table_a = column_flags.clone();
    let flags_for_table_b = column_flags.clone();
    let flags_for_table_c = column_flags.clone();
    let flags_for_modal = column_flags.clone();
    let flags_for_apply = column_flags.clone();

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

    // Shared criteria builder (Copy closure reused by the memo + count + funnel).
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

    let typed_asset: Signal<ScreenerAssetType> =
        Memo::new(move |_| ScreenerAssetType::from_bar(&asset_type.get())).into();

    let filtered_records: Memo<Vec<ScreenerRecord>> = Memo::new(move |_| {
        let criteria = build_criteria();
        let base = universe_for(typed_asset.get());
        let filtered = filter_universe(&base, &criteria);
        sort_universe(&filtered, sort_col.get(), sort_ascending.get())
    });
    let records: Signal<Vec<ScreenerRecord>> = filtered_records.into();

    let funnel_memo: Memo<FunnelTelemetry> =
        Memo::new(move |_| compute_funnel(&stock_universe(), &build_criteria()));
    let funnel: Signal<FunnelTelemetry> = funnel_memo.into();

    let on_export_csv = Callback::new(move |_| {
        let columns = visible_columns.get();
        let records_now = records.get();
        let headers = csv_headers(&columns);
        let rows = csv_rows(&records_now, &columns);
        let document = csv_document(&headers, &rows);
        #[cfg(target_arch = "wasm32")]
        crate::features::screener::csv_export::export_screener_csv(
            "screener-export.csv",
            &document,
        );
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = &document;
        }
    });

    let navigate = use_navigate();
    use_screener_shortcuts(Callback::new(
        move |action: ScreenerKeyAction| match action {
            ScreenerKeyAction::NextRow | ScreenerKeyAction::PrevRow => {
                let delta: usize = if matches!(action, ScreenerKeyAction::NextRow) {
                    1
                } else {
                    0
                };
                let rows = records.get_untracked();
                if rows.is_empty() {
                    return;
                }
                let current = rows
                    .iter()
                    .position(|r| r.symbol() == active_symbol.get_untracked())
                    .unwrap_or(0);
                let next = if delta == 1 {
                    (current + 1) % rows.len()
                } else {
                    (current + rows.len() - 1) % rows.len()
                };
                active_symbol.set(rows[next].symbol().to_string());
                drawer_open.set(true);
            }
            ScreenerKeyAction::ToggleStar => {
                let sym = active_symbol.get_untracked();
                watchlist.update(|w| {
                    if w.contains(&sym) {
                        w.remove(&sym);
                    } else {
                        w.insert(sym.clone());
                    }
                });
            }
            ScreenerKeyAction::EnterRoute => {
                navigate("/trading/cockpit", Default::default());
            }
            ScreenerKeyAction::OpenCatalog => catalog_modal_open.set(true),
            ScreenerKeyAction::SaveScreen => {
                catalog_modal_open.set(true);
                save_sheet_open.set(true);
            }
            ScreenerKeyAction::OpenFilters => filter_modal_open.set(true),
            ScreenerKeyAction::OpenColumns => column_modal_open.set(true),
            ScreenerKeyAction::ToggleView => {
                view_mode.set(match view_mode.get_untracked() {
                    ScreenerViewMode::Matrix => ScreenerViewMode::Constellation,
                    ScreenerViewMode::Constellation => ScreenerViewMode::Table,
                    ScreenerViewMode::Table => ScreenerViewMode::Heatmap,
                    ScreenerViewMode::Heatmap => ScreenerViewMode::Matrix,
                });
            }
            ScreenerKeyAction::Dismiss => {
                filter_modal_open.set(false);
                column_modal_open.set(false);
                catalog_modal_open.set(false);
                save_sheet_open.set(false);
                drawer_open.set(false);
            }
        },
    ));

    // Stocks-only subset used by Constellation and Heatmap views.
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
        active_screen_id.set("builtin.all".to_string());
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

    Effect::new(move |_| {
        let snap = custom_screens.get();
        persist_custom_screens(&snap);
    });

    let on_apply_screen = Callback::new(move |screen: CustomScreen| {
        active_screen_id.set(screen.id.clone());
        let c = screen.criteria.clone();
        search_query.set(c.search_query);
        selected_exchange.set(c.exchange);
        selected_sector.set(c.sector);
        selected_market_cap.set(c.market_cap_category);
        selected_index.set(c.market_index);
        min_price.set(c.min_price);
        max_price.set(c.max_price);
        min_pe.set(c.min_pe);
        max_pe.set(c.max_pe);
        min_rvol.set(c.min_rvol);
        min_rsi.set(c.min_rsi);
        max_rsi.set(c.max_rsi);
        fno_only.set(c.fno_only);
        watchlist_only.set(c.watchlist_only);
        active_tab.set(screen.tab.clone());
        sort_col.set(sort_key_for(&screen.sort_col, sort_col.get_untracked()));
        sort_ascending.set(screen.sort_ascending);
        if !screen.visible_columns.is_empty() {
            for (key, flag) in flags_for_apply.iter() {
                flag.set(screen.visible_columns.iter().any(|v| v == key));
            }
        }
    });

    let on_save_screen = Callback::new(move |name: String| {
        let plain = name.trim().to_string();
        if plain.is_empty() {
            return;
        }
        let visible: Vec<String> = visible_columns
            .get_untracked()
            .iter()
            .map(|c| c.key.to_string())
            .collect();
        let snap = snapshot_screen(
            &plain,
            "Saved screener view.",
            &active_tab.get_untracked(),
            sort_col.get_untracked(),
            sort_ascending.get_untracked(),
            build_criteria(),
            &visible,
        );
        custom_screens.update(|v| upsert_screen(v, snap));
    });

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
                on_export_csv=on_export_csv
                on_apply_screen=on_apply_screen
                active_screen_id=active_screen_id
                funnel=funnel
            />
            <div class="screener-body flex flex-1 min-h-0 overflow-hidden hf-card rounded-2xl border border-border-soft bg-surface-base shadow-sm">
                <div class="screener-main flex-1 min-w-0 overflow-x-auto">
                    {move || match view_mode.get() {
                        ScreenerViewMode::Matrix => view! {
                            <ScreenerSignalMatrix
                                records=records
                                active_symbol=active_symbol
                                watchlist=watchlist
                                drawer_open=drawer_open
                            />
                        }
                        .into_any(),
                        ScreenerViewMode::Constellation => {
                            if typed_asset.get() == ScreenerAssetType::Stocks {
                                view! {
                                    <ScreenerConstellation
                                        records=stock_records.into()
                                        active_symbol=active_symbol
                                        drawer_open=drawer_open
                                    />
                                }
                                .into_any()
                            } else {
                                view! {
                                    <div class="flex h-full flex-col">
                                        <div class="flex items-center border-b border-border-soft px-3 py-1.5 text-[11px] font-mono text-text-muted">
                                            "Alpha Constellation is available for Stocks. Showing Quantum Tape for this asset class."
                                        </div>
                                        <div class="min-h-0 flex-1 overflow-x-auto">
                                            <ScreenerTable
                                                records=records
                                                columns=visible_columns
                                                column_flags=flags_for_table_c.clone()
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
                                        </div>
                                    </div>
                                }
                                .into_any()
                            }
                        }
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
            {move || {
                catalog_modal_open
                    .get()
                    .then(|| {
                        view! {
                            <ScreenCatalogModal
                                open=catalog_modal_open
                                custom_screens=custom_screens
                                save_sheet_open=save_sheet_open
                                on_apply=on_apply_screen
                                on_save=on_save_screen
                            />
                        }
                    })
            }}
        </div>
    }
}
