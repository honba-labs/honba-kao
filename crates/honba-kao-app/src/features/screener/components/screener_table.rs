//! `ScreenerTable` — reactive results grid over the per-(asset-class, tab)
//! column list with tri-state sort arrows, per-header column menus (sort,
//! move, hide, criteria-bound numeric bounds), watchlist star toggles, and
//! row-click drawer activation.

use crate::features::screener::components::column_header_menu::{
    ColumnHeaderMenu, MoveDirection, swap_in_order,
};
use crate::features::screener::filters::record_cell;
use crate::features::screener::types::{
    ScreenerAssetType, ScreenerColumn, ScreenerRecord, catalog_for,
};
use honba_kao_ui_core::SortDirection;
use honba_kao_ui_core::primitive::HfSortArrows;
use leptos::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashSet;

const TH_CLASS: &str = "px-3 py-2 text-[11px] font-semibold uppercase tracking-wide text-text-muted border-b border-border-soft whitespace-nowrap";

fn row_class(selected: bool) -> String {
    let base =
        "border-b border-border-soft cursor-pointer hover:bg-surface-subtle transition-colors";
    if selected {
        format!("{} bg-emerald-600/10", base)
    } else {
        base.to_string()
    }
}

fn range_marker_pct(price: Decimal, low: Decimal, high: Decimal) -> u64 {
    let span = high - low;
    let pct: Decimal = if span == Decimal::ZERO {
        dec!(50)
    } else {
        (price - low) * dec!(100) / span
    };
    pct.round_dp(0)
        .to_string()
        .parse::<u64>()
        .unwrap_or(50)
        .min(100)
}

#[component]
pub fn ScreenerTable(
    records: Signal<Vec<ScreenerRecord>>,
    columns: Signal<Vec<ScreenerColumn>>,
    column_flags: Vec<(&'static str, RwSignal<bool>)>,
    column_order: RwSignal<Vec<&'static str>>,
    active_symbol: RwSignal<String>,
    watchlist: RwSignal<HashSet<String>>,
    sort_col: RwSignal<&'static str>,
    sort_ascending: RwSignal<bool>,
    drawer_open: RwSignal<bool>,
    min_price: RwSignal<Decimal>,
    max_price: RwSignal<Decimal>,
    min_pe: RwSignal<Decimal>,
    max_pe: RwSignal<Decimal>,
    min_rvol: RwSignal<Decimal>,
    min_rsi: RwSignal<Decimal>,
    max_rsi: RwSignal<Decimal>,
) -> impl IntoView {
    // One tri-state arrow signal per unique key across all four catalogs
    // (component body runs once; "symbol" etc. is shared by every class).
    let mut dirs: Vec<(&'static str, RwSignal<SortDirection>)> = Vec::new();
    for asset in [
        ScreenerAssetType::Stocks,
        ScreenerAssetType::Etf,
        ScreenerAssetType::Bonds,
        ScreenerAssetType::MutualFunds,
    ] {
        for col in catalog_for(asset) {
            if !dirs.iter().any(|(k, _)| *k == col.key) {
                dirs.push((col.key, RwSignal::new(SortDirection::Neutral)));
            }
        }
    }

    // Single-open column menu state across the whole table.
    let menu_open: RwSignal<Option<&'static str>> = RwSignal::new(None);

    let header_cells = move || {
        let mut out: Vec<AnyView> = Vec::new();
        for col in columns.get() {
            let key = col.key;
            let label = col.label;
            let this_dir = dirs
                .iter()
                .find(|entry| entry.0 == key)
                .map(|entry| entry.1);
            let hide_flag = column_flags
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, f)| *f);
            let dirs_for_reset = dirs.clone();
            let on_sort = Callback::new(move |ascending: bool| {
                sort_col.set(key);
                sort_ascending.set(ascending);
                for (_, other) in dirs_for_reset.iter() {
                    other.set(SortDirection::Neutral);
                }
                if let Some(dir) = this_dir {
                    dir.set(if ascending {
                        SortDirection::Ascending
                    } else {
                        SortDirection::Descending
                    });
                }
            });
            let on_move = Callback::new(move |direction: MoveDirection| {
                column_order.update(|order| {
                    if let Some(next) = swap_in_order(order, key, direction) {
                        *order = next;
                    }
                });
            });
            let sortable = col.sortable && this_dir.is_some();
            let sort_for_menu = sortable.then_some(on_sort);
            let menu = view! {
                <ColumnHeaderMenu
                    column_key=key
                    open=menu_open
                    hide_flag=hide_flag
                    on_sort=sort_for_menu
                    on_move=on_move
                    min_price=min_price
                    max_price=max_price
                    min_pe=min_pe
                    max_pe=max_pe
                    min_rvol=min_rvol
                    min_rsi=min_rsi
                    max_rsi=max_rsi
                />
            };
            let cell = if sortable {
                let dir_signal = this_dir.unwrap();
                let dirs_cycle = dirs.clone();
                view! {
                    <th class=TH_CLASS>
                        <span class="inline-flex items-center gap-1">
                            {label}
                            <HfSortArrows
                                direction=dir_signal
                                on_cycle=Callback::new(move |next| {
                                    sort_col.set(key);
                                    sort_ascending.set(next == SortDirection::Ascending);
                                    for (_, other) in dirs_cycle.iter() {
                                        other.set(SortDirection::Neutral);
                                    }
                                    dir_signal.set(next);
                                })
                            />
                            {menu}
                        </span>
                    </th>
                }
                .into_any()
            } else {
                view! {
                    <th class=TH_CLASS>
                        <span class="inline-flex items-center gap-1">
                            {label}
                            {menu}
                        </span>
                    </th>
                }
                .into_any()
            };
            out.push(cell);
        }
        out
    };

    let row_cells = move || {
        let visible_keys: Vec<&'static str> = columns.get().iter().map(|c| c.key).collect();
        records
            .get()
            .into_iter()
            .map(|record| {
                let rec_for_click = record.clone();
                let rec_for_class = record.clone();
                let mut cells: Vec<AnyView> = Vec::new();

                for &key in &visible_keys {
                    match key {
                        "star" => {
                            let rec_star = record.clone();
                            let rec_glyph = record.clone();
                            cells.push(
                                view! {
                                    <td
                                        class="px-3 py-2 text-xs cursor-pointer"
                                        on:click=move |ev: web_sys::MouseEvent| {
                                            ev.stop_propagation();
                                            let sym = rec_star.symbol().to_string();
                                            watchlist.update(move |w| {
                                                if w.contains(&sym) {
                                                    w.remove(&sym);
                                                } else {
                                                    w.insert(sym);
                                                }
                                            });
                                        }
                                    >
                                        <span class="text-[13px] leading-none">
                                            {move || {
                                                if watchlist.get().contains(rec_glyph.symbol()) {
                                                    "★"
                                                } else {
                                                    "☆"
                                                }
                                            }}
                                        </span>
                                    </td>
                                }
                                .into_any(),
                            );
                        }
                        "symbol" => {
                            let sym = record.symbol().to_string();
                            let nm = record.name().to_string();
                            cells.push(
                                view! {
                                    <td class="px-3 py-2 font-mono text-xs tabular-nums">
                                        <span class="block font-semibold text-text-high">
                                            {sym}
                                        </span>
                                        <span class="block text-[10px] text-text-muted">
                                            {nm}
                                        </span>
                                    </td>
                                }
                                .into_any(),
                            );
                        }
                        "range_52w" => {
                            let cell = if let ScreenerRecord::Stock(r) = &record {
                                let pct = range_marker_pct(r.price, r.low_52w, r.high_52w);
                                view! {
                                    <td class="px-3 py-2">
                                        <span class="hf-range-bar relative inline-block h-1.5 w-24 rounded-full bg-emerald-500/15 align-middle">
                                            <span
                                                class="absolute top-1/2 -translate-y-1/2 h-1 w-1 rounded-full bg-text-muted"
                                                style=format!("left: {}%", pct)
                                            ></span>
                                        </span>
                                    </td>
                                }
                                .into_any()
                            } else {
                                view! { <td class="px-3 py-2"></td> }.into_any()
                            };
                            cells.push(cell);
                        }
                        _ => {
                            let txt = record_cell(&record, key);
                            cells.push(
                                view! {
                                    <td class="px-3 py-2 font-mono text-xs tabular-nums text-text-high">
                                        {txt}
                                    </td>
                                }
                                .into_any(),
                            );
                        }
                    }
                }

                view! {
                    <tr
                        class=move || row_class(active_symbol.get() == rec_for_class.symbol())
                        on:click=move |_| {
                            active_symbol.set(rec_for_click.symbol().to_string());
                            drawer_open.set(true);
                        }
                    >
                        {cells}
                    </tr>
                }
                .into_any()
            })
            .collect::<Vec<AnyView>>()
    };

    view! {
        <table class="hf-screener-table w-full min-w-[720px] border-collapse">
            <thead>
                <tr>{header_cells}</tr>
            </thead>
            <tbody>{row_cells}</tbody>
        </table>
    }
}
