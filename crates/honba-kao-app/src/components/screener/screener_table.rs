//! The screener table — the Leptos port of the labs app's
//! `screener-table.tsx` + `column-header-menu.tsx` with the structural
//! column renderers of `core/columns.tsx`.
//!
//! Header cells open a per-column menu (sort asc / desc / none, hide
//! column, customize columns; the labs filter shortcuts live in the Filters
//! modal), the symbol header expands into the inline ticker search, rows
//! select the symbol (preview drawer), and the pagination footer mirrors
//! the reference (page x of y · rows 50/100/250/All).

use leptos::prelude::*;

use crate::components::workbench::dismiss_on_outside_click;
use crate::data::instruments::Instrument;
use crate::stores::screener::{
    AdvancedFilters, ColumnDef, DetailTab, ScreenerStore, ScreenerType, SortDirection,
    static_cagr3y, static_cagr5y,
};
use crate::utils::currency;

// ---------------------------------------------------------------------------
// Labs formatting helpers
// ---------------------------------------------------------------------------

/// `123,456.00` (US grouping, like the labs `formatNumber`).
fn format_number(num: f64) -> String {
    let sign = if num < 0.0 { "-" } else { "" };
    let formatted = format!("{:.2}", num.abs());
    let (unit, fraction) = formatted
        .split_once('.')
        .unwrap_or((formatted.as_str(), "00"));

    let bytes = unit.as_bytes();
    let mut grouped = String::with_capacity(unit.len() + unit.len() / 3);
    for (position, digit) in unit.char_indices() {
        if position > 0 && (bytes.len() - position) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    format!("{sign}{grouped}.{fraction}")
}

/// The labs compact scale: `1.80 T · 5.40 B · 8.20 Cr · 3.90 M · 12.40`.
fn format_compact(num: f64) -> String {
    if num >= 1e12 {
        format!("{:.2} T", num / 1e12)
    } else if num >= 1e9 {
        format!("{:.2} B", num / 1e9)
    } else if num >= 1e7 {
        format!("{:.2} Cr", num / 1e7)
    } else if num >= 1e6 {
        format!("{:.2} M", num / 1e6)
    } else if num >= 1e3 {
        format!("{:.2} K", num / 1e3)
    } else {
        format!("{num:.2}")
    }
}

/// `(rounded number, up/down suffix)` percent text, `formatPercent`.
fn percent_text(value: f64) -> String {
    let text = format!("{value:.2}%");
    if value >= 0.0 {
        format!("+{text}")
    } else {
        text
    }
}

fn rating_label(rating: &str) -> String {
    if rating.is_empty() {
        "—".to_string()
    } else {
        rating.to_string()
    }
}

/// `isColumnFiltered` — does the current advanced state constrain this column?
fn is_column_filtered(col_id: &str, advanced: &AdvancedFilters) -> bool {
    match col_id {
        "price" => advanced.min_price.is_some() || advanced.max_price.is_some(),
        "changePercent" => {
            advanced.min_change_percent.is_some() || advanced.max_change_percent.is_some()
        }
        "volume" => advanced.min_volume.is_some(),
        "rsi14" => advanced.rsi_min.is_some() || advanced.rsi_max.is_some(),
        "pe" => advanced.pe_min.is_some() || advanced.pe_max.is_some(),
        "marketCap" => !advanced.market_cap_tier.is_empty() && advanced.market_cap_tier != "all",
        "dividendYield" => advanced.min_dividend_yield.is_some(),
        "analystRating" | "technicalRating" => {
            !advanced.technical_rating.is_empty() && advanced.technical_rating != "all"
        }
        "sector" => !advanced.sector.is_empty() && advanced.sector != "all",
        _ => false,
    }
}

fn country_flag(country: &str) -> &'static str {
    match country {
        "IN" => "🇮🇳",
        "US" => "🇺🇸",
        _ => "🌐",
    }
}

// ---------------------------------------------------------------------------
// Cell renderers (one helper per labs renderer)
// ---------------------------------------------------------------------------

fn plain_num(text: String, up: bool, down: bool) -> AnyView {
    view! {
        <span class=move || {
            if down { "sc-td-num sc-td-num--dn" } else if up { "sc-td-num sc-td-num--up" } else { "sc-td-num" }
        }>{text}</span>
    }
    .into_any()
}

fn currency_unit(text: String) -> AnyView {
    view! {
        <span class="sc-td-pair">
            <span class="sc-td-num">{text}</span>
            <span class="sc-td-unit">"₹"</span>
        </span>
    }
    .into_any()
}

fn sector_text(text: String) -> AnyView {
    view! { <span class="sc-td-sector">{text}</span> }.into_any()
}

fn percent_cell(value: f64) -> AnyView {
    let text = percent_text(value);
    plain_num(text, value >= 0.0, value < 0.0)
}

fn rating_cell(rating: &str) -> AnyView {
    let (class_suffix, label): (&'static str, String) = match rating {
        "Strong Buy" => ("strong-buy", "Strong Buy".into()),
        "Buy" => ("buy", "Buy".into()),
        "Neutral" => ("neutral", "Neutral".into()),
        "Sell" => ("sell", "Sell".into()),
        "Strong Sell" => ("strong-sell", "Strong Sell".into()),
        _ => ("none", rating_label(rating)),
    };
    view! {
        <span class=format!("sc-rate sc-rate--{class_suffix}")>{label}</span>
    }
    .into_any()
}

/// The labs `RangeBar`: full 52W track with the price position marker.
fn range52_cell(inst: &Instrument) -> AnyView {
    let span = (inst.high52 - inst.low52).abs().max(1e-6);
    let pct = ((inst.price - inst.low52) / span).clamp(0.0, 1.0) * 100.0;
    let marker = format!("left: {pct:.1}%;");
    view! {
        <div class="sc-rangebar" title=format!("52W: {:.2} — {:.2}", inst.low52, inst.high52)>
            <span class="sc-rangebar__marker" style=marker></span>
        </div>
    }
    .into_any()
}

/// The labs `Sparkline`: svg polyline in gain/loss color.
fn sparkline_cell(inst: &Instrument) -> AnyView {
    let points = if inst.sparkline.len() >= 2 {
        inst.sparkline.clone()
    } else {
        vec![inst.price * 0.98, inst.price]
    };
    let min = points.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = points.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let span = (max - min).abs().max(1e-6);
    let step = 60.0 / (points.len().max(2) - 1) as f64;
    let coords = points
        .iter()
        .enumerate()
        .map(|(position, point)| {
            let x = position as f64 * step;
            let y = 15.0 - ((point - min) / span) * 14.0;
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ");
    let color = if inst.change_percent >= 0.0 {
        "var(--pnl-gain)"
    } else {
        "var(--pnl-loss)"
    };
    view! {
        <svg
            class="sc-spark"
            width="60"
            height="16"
            viewBox="0 0 60 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <polyline points=coords stroke=color stroke-width="1.2" stroke-linejoin="round" />
        </svg>
    }
    .into_any()
}

/// The structural-cell dispatch — one arm per `columns_registry()` id with
/// the labs fallbacks for debt / fund columns not present in the dataset.
fn cell(id: &str, inst: &Instrument, store: ScreenerStore) -> AnyView {
    let asset_type = inst.asset_type.as_str();
    match id {
        "symbol" => {
            let ticker = inst.symbol.clone();
            let ticker_for_click = inst.symbol.clone();
            let exchange = inst.exchange.clone();
            let name = inst.name.clone();
            let country = inst.country.clone();
            let dividend = inst.dividend_yield > 0.0;
            view! {
                <span class="sc-symbol__cell">
                    <span class="sc-symbol__logo">{logo_letters(&inst.symbol)}</span>
                    <span class="sc-symbol__details">
                        <span class="sc-symbol__row1">
                            <button
                                class="sc-symbol__ticker"
                                title="Open the detail drawer"
                                on:click=move |_| store.open_detail(&ticker_for_click, DetailTab::Overview)
                            >
                                <strong>{ticker}</strong>
                            </button>
                            <span class="sc-symbol__exchange">{exchange}</span>
                        </span>
                        <span class="sc-symbol__row2">
                            <span>{country_flag(&country)}</span>
                            <span class="sc-symbol__name">{name}</span>
                            <Show when=move || dividend>
                                <span class="sc-symbol__dividend" title="Regular dividend payer">"D"</span>
                            </Show>
                        </span>
                    </span>
                </span>
            }
            .into_any()
        }
        "exchange" => sector_text(inst.exchange.clone()),
        "country" => {
            let text = format!("{} {}", country_flag(&inst.country), inst.country);
            plain_num(text, false, false)
        }
        "price" => currency_unit(format_number(inst.price)),
        "changePercent" => percent_cell(inst.change_percent),
        "change" => {
            let text = if inst.change >= 0.0 {
                format!("+{}", format_number(inst.change))
            } else {
                format_number(inst.change)
            };
            plain_num(text, inst.change >= 0.0, inst.change < 0.0)
        }
        "volume" => plain_num(format_compact(inst.volume), false, false),
        "relVol" => plain_num(format!("{:.2}", inst.rel_vol()), false, false),
        "perf1W" => percent_cell(inst.perf1w),
        "perf1M" => percent_cell(inst.perf1m),
        "perf3M" => percent_cell(inst.perf3m),
        "perf1Y" => percent_cell(inst.perf1y),
        "technicalRating" | "analystRating" => rating_cell(&inst.technical_rating),
        "rsi14" => plain_num(
            format!("{:.1}", inst.rsi14),
            inst.rsi14 < 35.0,
            inst.rsi14 > 70.0,
        ),
        "range52" => range52_cell(inst),
        "sma200" => currency_unit(format_number(inst.sma200)),
        "sma50" => currency_unit(format_number(inst.sma50)),
        "sparkline" => sparkline_cell(inst),
        "high52" => currency_unit(format_number(inst.high52)),
        "low52" => currency_unit(format_number(inst.low52)),
        "marketCap" => currency_unit(format_compact(inst.market_cap)),
        "pe" => plain_num(
            if inst.pe > 0.0 {
                format!("{:.2}", inst.pe)
            } else {
                "—".to_string()
            },
            false,
            false,
        ),
        "forwardPe" => plain_num(
            if inst.forward_pe > 0.0 {
                format!("{:.2}", inst.forward_pe)
            } else {
                "—".to_string()
            },
            false,
            false,
        ),
        "pb" => plain_num(
            if inst.pb > 0.0 {
                format!("{:.2}", inst.pb)
            } else {
                "—".to_string()
            },
            false,
            false,
        ),
        "eps" => currency_unit(format!("{:.2}", inst.eps)),
        "epsGrowth" => percent_cell(inst.revenue_growth),
        "dividendYield" => plain_num(
            format!("{:.2}%", inst.dividend_yield),
            inst.dividend_yield > 0.0,
            false,
        ),
        "sector" => sector_text(if inst.sector.is_empty() {
            "—".into()
        } else {
            inst.sector.clone()
        }),
        "netMargin" => plain_num(
            if inst.net_margin > 0.0 {
                format!("{:.1}%", inst.net_margin)
            } else {
                "—".to_string()
            },
            false,
            false,
        ),
        "roce" => plain_num(
            if inst.roce > 0.0 {
                format!("{:.1}%", inst.roce)
            } else {
                "—".to_string()
            },
            false,
            false,
        ),
        "roe" => plain_num(
            if inst.roe > 0.0 {
                format!("{:.1}%", inst.roe)
            } else {
                "—".to_string()
            },
            false,
            false,
        ),
        "debtToEquity" => plain_num(
            if inst.debt_to_equity > 0.0 {
                format!("{:.2}", inst.debt_to_equity)
            } else {
                "—".to_string()
            },
            false,
            false,
        ),
        // Funds / ETF domain columns (labs fallbacks over the dataset).
        "aum" => currency_unit(format_compact(inst.market_cap)),
        "expenseRatio" => {
            let text = if asset_type == "etf" {
                "0.12%"
            } else if asset_type == "mf" {
                "0.69%"
            } else {
                "—"
            };
            plain_num(text.into(), false, false)
        }
        "totalExpenseRatio" => plain_num(
            if asset_type == "mf" {
                "0.69%".into()
            } else {
                "—".into()
            },
            false,
            false,
        ),
        "catTotalExpenseRatio" => plain_num(
            if asset_type == "mf" {
                "0.53%".into()
            } else {
                "—".into()
            },
            false,
            false,
        ),
        "sharpeRatio" => {
            let available = asset_type == "mf";
            plain_num(
                if available {
                    "1.42".into()
                } else {
                    "—".into()
                },
                available,
                false,
            )
        }
        "catSharpeRatio" => plain_num(
            if asset_type == "mf" {
                "1.21".into()
            } else {
                "—".into()
            },
            false,
            false,
        ),
        "cagr3y" => percent_cell(static_cagr3y(inst)),
        "cagr5y" => percent_cell(static_cagr5y(inst)),
        "schemeType" => sector_text(if inst.industry.contains("Fund") {
            "Growth".into()
        } else if inst.industry.is_empty() {
            "—".into()
        } else {
            inst.industry.clone()
        }),
        "brand" | "assetClass" | "focus" | "planType" | "riskometer" | "issuer" | "issuerType"
        | "couponType" | "maturityYear" => sector_text(if inst.industry.is_empty() {
            "—".into()
        } else {
            inst.industry.clone()
        }),
        "navReturn" => percent_cell(inst.perf1y),
        // Bonds domain columns (labs fallbacks).
        "ytw" => plain_num(
            format!("{:.2}%", inst.dividend_yield),
            inst.dividend_yield > 0.0,
            false,
        ),
        "coupon" => plain_num(
            if inst.eps > 0.0 {
                format!("{:.2}%", inst.eps)
            } else {
                "—".to_string()
            },
            false,
            false,
        ),
        "creditRating" => rating_cell(&inst.technical_rating),
        _ => plain_num("—".into(), false, false),
    }
}

// ---------------------------------------------------------------------------
// Column header menu
// ---------------------------------------------------------------------------

#[component]
fn ColHeader(col: ColumnDef, is_filtered: bool, store: ScreenerStore) -> impl IntoView {
    let menu_open = RwSignal::new(false);
    let menu_host = NodeRef::<leptos::html::Div>::new();
    dismiss_on_outside_click(menu_open, menu_host);

    let label = col.label.to_string();
    let has_arrow = true;

    let sort_asc = move |_| {
        store.sort_field.set(col.id.to_string());
        store.sort_order.set(Some(SortDirection::Asc));
        menu_open.set(false);
    };
    let sort_desc = move |_| {
        store.sort_field.set(col.id.to_string());
        store.sort_order.set(Some(SortDirection::Desc));
        menu_open.set(false);
    };
    let clear_sort = move |_| {
        store.sort_field.set(String::new());
        store.sort_order.set(None);
        menu_open.set(false);
    };
    let hide = move |_| {
        let id = col.id.to_string();
        store
            .visible_columns
            .update(|ids| ids.retain(|entry| entry != &id));
        menu_open.set(false);
    };
    let manage = move |_| {
        store.column_modal.set(true);
        menu_open.set(false);
    };

    // Sort indicator readouts.
    let is_sorted_asc = move || {
        store.sort_field.get() == col.id && store.sort_order.get() == Some(SortDirection::Asc)
    };
    let is_sorted_desc = move || {
        store.sort_field.get() == col.id && store.sort_order.get() == Some(SortDirection::Desc)
    };

    view! {
        <div class="sc-th" node_ref=menu_host>
            <button
                class="sc-th__label"
                title=format!("Filter, sort, or configure {}", col.label)
                on:click=move |_| menu_open.update(|open| *open = !*open)
            >
                <Show when=move || is_filtered>
                    <span class="sc-th__filter-dot" title="Column is filtered"></span>
                </Show>
                <Show when=is_sorted_asc>
                    <span class="sc-th__arrow">"↑"</span>
                </Show>
                <Show when=is_sorted_desc>
                    <span class="sc-th__arrow">"↓"</span>
                </Show>
                <span>{label.clone()}</span>
                <Show when=move || has_arrow>
                    <span class="sc-th__caretp">"▾"</span>
                </Show>
            </button>

            <Show when=move || menu_open.get()>
                <div class="sc-th__menu" role="menu">
                    <div class="sc-th__menu-title">{label.clone()}</div>
                    <button
                        class="sc-th__menu-item"
                        class:sc-th__menu-item--on=is_sorted_asc
                        role="menuitem"
                        on:click=sort_asc
                    >
                        "Sort ascending"
                    </button>
                    <button
                        class="sc-th__menu-item"
                        class:sc-th__menu-item--on=is_sorted_desc
                        role="menuitem"
                        on:click=sort_desc
                    >
                        "Sort descending"
                    </button>
                    <button class="sc-th__menu-item" role="menuitem" on:click=clear_sort>
                        "No sort"
                    </button>
                    <Show when=move || col.id != "symbol">
                        <div class="sc-th__menu-rule"></div>
                        <button class="sc-th__menu-item" role="menuitem" on:click=hide>
                            "Hide column"
                        </button>
                    </Show>
                    <div class="sc-th__menu-rule"></div>
                    <button class="sc-th__menu-item" role="menuitem" on:click=manage>
                        "Customize columns…"
                    </button>
                </div>
            </Show>
        </div>
    }
}

// ---------------------------------------------------------------------------
// ScreenerTable
// ---------------------------------------------------------------------------

/// `tick` padding for the symbol cell logo tile.
fn logo_letters(symbol: &str) -> String {
    symbol.chars().take(2).collect::<String>().to_uppercase()
}

#[component]
pub fn ScreenerTable() -> impl IntoView {
    let store = ScreenerStore::take();

    let header_search_open = RwSignal::new(false);
    let search_query = store.search_query;

    let visible_columns = Memo::new(move |_| {
        store
            .columns()
            .get()
            .into_iter()
            .filter(|(_, visible)| *visible)
            .map(|(column, _)| column)
            .collect::<Vec<ColumnDef>>()
    });

    let page_rows = store.page_rows();
    let sorted_rows = store.sorted();

    let total_items = Memo::new(move |_| sorted_rows.get().len());
    let item_label = Memo::new(move |_| match store.screener_type.get() {
        ScreenerType::Etf => "ETFs",
        ScreenerType::Bonds => "bonds",
        ScreenerType::Mf => "funds",
        ScreenerType::Stocks => "stocks",
    });

    let max_page = Memo::new(move |_| {
        let size = store.page_size.get();
        if size == 0 {
            1
        } else {
            total_items.get().div_ceil(size)
        }
    });
    let valid_page = Memo::new(move |_| store.page.get().min(max_page.get().max(1)));

    let search_active = move || header_search_open.get() || !search_query.get().trim().is_empty();

    let clear_search = move |_| {
        header_search_open.set(false);
        search_query.set(String::new());
    };

    let row_click = move |symbol: &str| {
        store.set_symbol(symbol);
    };
    let row_double_click = move |symbol: String| {
        let url = format!("/?symbol={}", symbol.to_lowercase());
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url_and_target(&url, "_blank");
        }
    };

    view! {
        <div class="sc-table-wrap">
            <table class="sc-table">
                <thead>
                    <tr>
                        {move || {
                            let is_filtered_by = store.advanced_filters.get_untracked();
                            visible_columns.get().into_iter().map(|col| {
                                let filtered = is_column_filtered(col.id, &is_filtered_by);

                                if col.id == "symbol" {
                                    view! {
                                        <th class="sc-th sc-th--symbol">
                                            <div class="sc-th__symbol">
                                                <Show when=search_active>
                                                    <input
                                                        class="sc-th__search"
                                                        type="text"
                                                        placeholder="Search"
                                                        prop:value=search_query
                                                        on:input=move |ev| {
                                                            let input = event_target::<web_sys::HtmlInputElement>(&ev);
                                                            store.set_search_query(input.value());
                                                        }
                                                        on:keydown=move |ev| {
                                                            if event_target::<web_sys::HtmlInputElement>(&ev).value().is_empty() && ev.key() == "Escape" {
                                                                header_search_open.set(false);
                                                            }
                                                        }
                                                    />
                                                    <button
                                                        class="sc-th__search-close"
                                                        title="Clear search (Esc)"
                                                        on:click=clear_search
                                                    >
                                                        "×"
                                                    </button>
                                                    <span class="sc-th__count">
                                                        {move || format_number(total_items.get() as f64)}
                                                    </span>
                                                </Show>
                                                <Show when=move || !search_active()>
                                                    <button
                                                        class="sc-th__search-btn"
                                                        title="Search tickers"
                                                        on:click=move |_| header_search_open.set(true)
                                                    >
                                                        "⌕"
                                                    </button>
                                                    <span
                                                        class="sc-th__title sc-th__title--click"
                                                        title="Sort by symbol"
                                                        on:click=move |_| store.set_sort("symbol")
                                                    >
                                                        "Symbol"
                                                    </span>
                                                    <span class="sc-th__count">
                                                        {move || format_number(total_items.get() as f64)}
                                                    </span>
                                                </Show>
                                            </div>
                                        </th>
                                    }
                                    .into_any()
                                } else {
                                    view! {
                                        <th class="sc-th" scope="col">
                                            <ColHeader col=col is_filtered=filtered store=store />
                                        </th>
                                    }
                                    .into_any()
                                }
                            }).collect_view()
                        }}
                        <th class="sc-th sc-th--add">
                            <button
                                class="sc-th__add"
                                title="Add / remove columns"
                                on:click=move |_| store.column_modal.set(true)
                            >
                                "+"
                            </button>
                        </th>
                    </tr>
                </thead>

                <tbody>
                    <Show when=move || page_rows.get().is_empty()>
                        <tr>
                            <td class="sc-table__empty" colspan=move || (visible_columns.get().len() + 1) as i32>
                                "No matching instruments found"
                                <div class="sc-table__empty-hint">
                                    "Try clearing some of your filter parameters or search query."
                                </div>
                            </td>
                        </tr>
                    </Show>
                    {move || {
                        page_rows.get().into_iter().map(|inst| {
                            let symbol = inst.symbol.clone();
                            let symbol_for_dbl = inst.symbol.clone();
                            let symbol_for_store = inst.symbol.clone();
                            let selected = move || store.active_symbol.get() == symbol;
                            let _ = logo_letters(&inst.symbol);
                            let cells = visible_columns.get().into_iter().map(|col| {
                                let id = col.id;
                                let class = if id == "symbol" {
                                    "sc-td sc-td--symbol".to_string()
                                } else {
                                    format!("sc-td sc-td--{}", col.align.as_class())
                                };
                                let content = cell(id, inst, store);
                                view! {
                                    <td class=class>{content}</td>
                                }
                            }).collect_view();
                            view! {
                                <tr
                                    class=move || {
                                        if selected() {
                                            "sc-tr sc-tr--selected"
                                        } else {
                                            "sc-tr"
                                        }
                                    }
                                    title="Click to preview · double-click to open in the Workbench"
                                    on:click=move |_| row_click(&symbol_for_store)
                                    on:dblclick=move |_| row_double_click(symbol_for_dbl.clone())
                                >
                                    {cells}
                                    <td class="sc-td sc-td--pad"></td>
                                </tr>
                            }
                        }).collect_view()
                    }}
                </tbody>
            </table>

            // Pagination footer
            <div class="sc-table__footer">
                <div class="sc-table__pageinfo">
                    "Page "
                    <strong>{move || valid_page.get()}</strong>
                    " of "
                    <strong>{move || max_page.get()}</strong>
                    " ( "
                    {move || format_number(total_items.get() as f64)}
                    " "
                    {move || item_label.get()}
                    " )"
                </div>
                <div class="sc-table__pager">
                    <button
                        class="sc-table__pager-btn"
                        title="Previous page"
                        disabled=move || valid_page.get() <= 1
                        on:click=move |_| store.set_page(valid_page.get() - 1)
                    >
                        "‹"
                    </button>
                    <span class="sc-table__page-badge">{move || valid_page.get()}</span>
                    <button
                        class="sc-table__pager-btn"
                        title="Next page"
                        disabled=move || valid_page.get() >= max_page.get()
                        on:click=move |_| store.set_page(valid_page.get() + 1)
                    >
                        "›"
                    </button>
                    <div class="sc-table__pagesize">
                        <span class="sc-table__pagesize-label">"Rows"</span>
                        <select
                            class="sc-table__pagesize-select"
                            prop:value=move || store.page_size.get().to_string()
                            on:change=move |ev| {
                                let text = event_target::<web_sys::HtmlSelectElement>(&ev).value();
                                let size = text.parse::<usize>().unwrap_or(0);
                                store.set_page_size(size);
                            }
                        >
                            <option value="50">"50"</option>
                            <option value="100">"100"</option>
                            <option value="250">"250"</option>
                            <option value="0">"All"</option>
                        </select>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Mirror of the ticker-button click-through used by cells: opens the detail
/// drawer through a fresh context lookup (cells run inside the tree, so a
/// simple re-use of the thread-local is safe).
#[allow(dead_code)]
fn _keep_cell_helper_names() {
    let _ = logo_letters;
    let _ = percent_text;
    let _ = is_column_filtered;
    let _ = currency;
}
