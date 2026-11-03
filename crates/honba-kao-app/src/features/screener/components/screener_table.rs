//! `ScreenerTable` — reactive results grid over `COLUMN_CATALOG` with
//! per-column visibility, tri-state sort arrows, watchlist star toggles,
//! and row-click drawer activation.

use crate::features::screener::filters::{
    format_change_pct, format_mcap_rs_cr, format_price, format_volume_compact,
};
use crate::features::screener::types::{COLUMN_CATALOG, ScreenerEquityRecord};
use honba_kao_ui_core::SortDirection;
use honba_kao_ui_core::primitive::HfSortArrows;
use leptos::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashSet;

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
    records: Signal<Vec<ScreenerEquityRecord>>,
    active_symbol: RwSignal<String>,
    watchlist: RwSignal<HashSet<String>>,
    visible_columns: Signal<Vec<&'static str>>,
    sort_col: RwSignal<&'static str>,
    sort_ascending: RwSignal<bool>,
    drawer_open: RwSignal<bool>,
) -> impl IntoView {
    // Per-column arrow signals, created once (component body runs once).
    let dir_symbol = RwSignal::new(SortDirection::Neutral);
    let dir_price = RwSignal::new(SortDirection::Neutral);
    let dir_chg = RwSignal::new(SortDirection::Neutral);
    let dir_vol = RwSignal::new(SortDirection::Neutral);
    let dir_cap = RwSignal::new(SortDirection::Descending);
    let dir_pe = RwSignal::new(SortDirection::Neutral);

    let header_cells = move || {
        let mut out: Vec<AnyView> = Vec::new();
        for col in COLUMN_CATALOG {
            if !visible_columns.get().contains(&col.key) {
                continue;
            }
            let key = col.key;
            let label = col.label;
            let cell = match key {
                "symbol" => view! {
                    <th class="px-3 py-2 text-[11px] font-semibold uppercase tracking-wide text-text-muted border-b border-border-soft whitespace-nowrap">
                        <span class="inline-flex items-center gap-1">
                            {label}
                            <HfSortArrows
                                direction=dir_symbol
                                on_cycle=Callback::new(move |next| {
                                    sort_col.set("symbol");
                                    sort_ascending.set(next == SortDirection::Ascending);
                                    dir_price.set(SortDirection::Neutral);
                                    dir_chg.set(SortDirection::Neutral);
                                    dir_vol.set(SortDirection::Neutral);
                                    dir_cap.set(SortDirection::Neutral);
                                    dir_pe.set(SortDirection::Neutral);
                                })
                            />
                        </span>
                    </th>
                }
                .into_any(),
                "price" => view! {
                    <th class="px-3 py-2 text-[11px] font-semibold uppercase tracking-wide text-text-muted border-b border-border-soft whitespace-nowrap">
                        <span class="inline-flex items-center gap-1">
                            {label}
                            <HfSortArrows
                                direction=dir_price
                                on_cycle=Callback::new(move |next| {
                                    sort_col.set("price");
                                    sort_ascending.set(next == SortDirection::Ascending);
                                    dir_symbol.set(SortDirection::Neutral);
                                    dir_chg.set(SortDirection::Neutral);
                                    dir_vol.set(SortDirection::Neutral);
                                    dir_cap.set(SortDirection::Neutral);
                                    dir_pe.set(SortDirection::Neutral);
                                })
                            />
                        </span>
                    </th>
                }
                .into_any(),
                "change_pct" => view! {
                    <th class="px-3 py-2 text-[11px] font-semibold uppercase tracking-wide text-text-muted border-b border-border-soft whitespace-nowrap">
                        <span class="inline-flex items-center gap-1">
                            {label}
                            <HfSortArrows
                                direction=dir_chg
                                on_cycle=Callback::new(move |next| {
                                    sort_col.set("change_pct");
                                    sort_ascending.set(next == SortDirection::Ascending);
                                    dir_symbol.set(SortDirection::Neutral);
                                    dir_price.set(SortDirection::Neutral);
                                    dir_vol.set(SortDirection::Neutral);
                                    dir_cap.set(SortDirection::Neutral);
                                    dir_pe.set(SortDirection::Neutral);
                                })
                            />
                        </span>
                    </th>
                }
                .into_any(),
                "volume" => view! {
                    <th class="px-3 py-2 text-[11px] font-semibold uppercase tracking-wide text-text-muted border-b border-border-soft whitespace-nowrap">
                        <span class="inline-flex items-center gap-1">
                            {label}
                            <HfSortArrows
                                direction=dir_vol
                                on_cycle=Callback::new(move |next| {
                                    sort_col.set("volume");
                                    sort_ascending.set(next == SortDirection::Ascending);
                                    dir_symbol.set(SortDirection::Neutral);
                                    dir_price.set(SortDirection::Neutral);
                                    dir_chg.set(SortDirection::Neutral);
                                    dir_cap.set(SortDirection::Neutral);
                                    dir_pe.set(SortDirection::Neutral);
                                })
                            />
                        </span>
                    </th>
                }
                .into_any(),
                "market_cap_cr" => view! {
                    <th class="px-3 py-2 text-[11px] font-semibold uppercase tracking-wide text-text-muted border-b border-border-soft whitespace-nowrap">
                        <span class="inline-flex items-center gap-1">
                            {label}
                            <HfSortArrows
                                direction=dir_cap
                                on_cycle=Callback::new(move |next| {
                                    sort_col.set("market_cap_cr");
                                    sort_ascending.set(next == SortDirection::Ascending);
                                    dir_symbol.set(SortDirection::Neutral);
                                    dir_price.set(SortDirection::Neutral);
                                    dir_chg.set(SortDirection::Neutral);
                                    dir_vol.set(SortDirection::Neutral);
                                    dir_pe.set(SortDirection::Neutral);
                                })
                            />
                        </span>
                    </th>
                }
                .into_any(),
                "pe_ratio" => view! {
                    <th class="px-3 py-2 text-[11px] font-semibold uppercase tracking-wide text-text-muted border-b border-border-soft whitespace-nowrap">
                        <span class="inline-flex items-center gap-1">
                            {label}
                            <HfSortArrows
                                direction=dir_pe
                                on_cycle=Callback::new(move |next| {
                                    sort_col.set("pe_ratio");
                                    sort_ascending.set(next == SortDirection::Ascending);
                                    dir_symbol.set(SortDirection::Neutral);
                                    dir_price.set(SortDirection::Neutral);
                                    dir_chg.set(SortDirection::Neutral);
                                    dir_vol.set(SortDirection::Neutral);
                                    dir_cap.set(SortDirection::Neutral);
                                })
                            />
                        </span>
                    </th>
                }
                .into_any(),
                _ => view! {
                    <th class="px-3 py-2 text-[11px] font-semibold uppercase tracking-wide text-text-muted border-b border-border-soft whitespace-nowrap">
                        {label}
                    </th>
                }
                .into_any(),
            };
            out.push(cell);
        }
        out
    };

    let row_cells = move || {
        records
            .get()
            .into_iter()
            .map(|record| {
                let rec_for_click = record.clone();
                let rec_for_class = record.clone();
                let mut cells: Vec<AnyView> = Vec::new();

                if visible_columns.get().contains(&"star") {
                    let rec_star = record.clone();
                    let rec_glyph = record.clone();
                    cells.push(
                        view! {
                            <td
                                class="px-3 py-2 text-xs cursor-pointer"
                                on:click=move |ev: web_sys::MouseEvent| {
                                    ev.stop_propagation();
                                    let sym = rec_star.symbol.clone();
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
                                        if watchlist.get().contains(&rec_glyph.symbol) {
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
                if visible_columns.get().contains(&"symbol") {
                    cells.push(
                        view! {
                            <td class="px-3 py-2 font-mono text-xs tabular-nums">
                                <span class="block font-semibold text-text-high">
                                    {record.symbol.clone()}
                                </span>
                                <span class="block text-[10px] text-text-muted">
                                    {record.name.clone()}
                                </span>
                            </td>
                        }
                        .into_any(),
                    );
                }
                if visible_columns.get().contains(&"price") {
                    let txt = format_price(record.price);
                    cells.push(
                        view! {
                            <td class="px-3 py-2 font-mono text-xs tabular-nums text-text-high">
                                {txt}
                            </td>
                        }
                        .into_any(),
                    );
                }
                if visible_columns.get().contains(&"change_pct") {
                    let txt = format_change_pct(record.change_pct);
                    let tone = if record.change_pct >= Decimal::ZERO {
                        "text-emerald-400"
                    } else {
                        "text-rose-400"
                    };
                    cells.push(
                        view! {
                            <td class="px-3 py-2 font-mono text-xs tabular-nums">
                                <span class=tone>{txt}</span>
                            </td>
                        }
                        .into_any(),
                    );
                }
                if visible_columns.get().contains(&"volume") {
                    let txt = format_volume_compact(record.volume);
                    cells.push(
                        view! {
                            <td class="px-3 py-2 font-mono text-xs tabular-nums text-text-high">
                                {txt}
                            </td>
                        }
                        .into_any(),
                    );
                }
                if visible_columns.get().contains(&"rvol") {
                    let txt = format!("{}x", record.rvol.round_dp(2));
                    cells.push(
                        view! {
                            <td class="px-3 py-2 font-mono text-xs tabular-nums text-text-high">
                                {txt}
                            </td>
                        }
                        .into_any(),
                    );
                }
                if visible_columns.get().contains(&"market_cap_cr") {
                    let txt = format_mcap_rs_cr(record.market_cap_cr);
                    cells.push(
                        view! {
                            <td class="px-3 py-2 font-mono text-xs tabular-nums text-text-high">
                                {txt}
                            </td>
                        }
                        .into_any(),
                    );
                }
                if visible_columns.get().contains(&"pe_ratio") {
                    let txt = if record.pe_ratio == Decimal::ZERO {
                        "—".to_string()
                    } else {
                        record.pe_ratio.round_dp(1).to_string()
                    };
                    cells.push(
                        view! {
                            <td class="px-3 py-2 font-mono text-xs tabular-nums text-text-high">
                                {txt}
                            </td>
                        }
                        .into_any(),
                    );
                }
                if visible_columns.get().contains(&"range_52w") {
                    let pct = range_marker_pct(record.price, record.low_52w, record.high_52w);
                    cells.push(
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
                        .into_any(),
                    );
                }

                view! {
                    <tr
                        class=move || row_class(active_symbol.get() == rec_for_class.symbol)
                        on:click=move |_| {
                            active_symbol.set(rec_for_click.symbol.clone());
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
