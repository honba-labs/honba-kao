//! The screener drawers — the Leptos port of the labs app's
//! `index-drawer.tsx` + `symbol-detail-drawer.tsx`:
//!
//! * [`IndexDrawer`] — the anchor-pill multi-select dropdown over
//!   `MARKET_INDICES` (search, badge chips, per-item check rows,
//!   select-all / clear footer) that publishes the selected codes into
//!   `store.advanced_filters.indices`.
//! * [`SymbolDetailDrawer`] — the right-hand preview panel docked next to
//!   the table/heatmap body: quote banner, spark preview, dynamic range
//!   bar, technical rating breakdown, key statistics and performance
//!   bars, with the shortlist star and workbench click-through.

use leptos::prelude::*;

use honba_kao_ui_core::HfIconButton;
use honba_kao_ui_core::icons::outline::x_close;

use crate::features::screener::data::indices::{IndexItem, indices_for_market};
use crate::features::screener::data::instruments::{Instrument, by_symbol};
use crate::features::screener::stores::screener_store::{DetailTab, ScreenerStore};
use crate::shared::utils::format::percent;

// ---------------------------------------------------------------------------
// IndexDrawer
// ---------------------------------------------------------------------------

/// The index pill-dropdown (labs `IndexDrawer`): search + check rows +
/// select-all footer. Rendered under the filter bar's Index trigger.
#[component]
pub fn IndexDrawer(
    /// Shared open flag (owned by the filter bar trigger).
    open: RwSignal<bool>,
    host: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    let store = ScreenerStore::take();
    let query = RwSignal::new(String::new());

    // Dismiss on outside clicks (same pattern as the table header menus).
    Effect::new(move |_| {
        if !open.get() {
            return;
        }
        let Some(element) = host.get() else {
            return;
        };
        let handle = window_event_listener_untyped("mousedown", move |event: web_sys::Event| {
            let target = event_target::<web_sys::Element>(&event);
            if !element.contains(Some(&target)) {
                open.set(false);
            }
        });
        on_cleanup(move || handle.remove());
    });

    // The indices of the active market, filtered by the search query.
    let listed = Memo::new(move |_| {
        let text = query.get().to_lowercase();
        let rows = indices_for_market(store.market.get_untracked());
        if text.trim().is_empty() {
            rows
        } else {
            rows.into_iter()
                .filter(|item| {
                    item.code.to_lowercase().contains(&text)
                        || item.name.to_lowercase().contains(&text)
                })
                .collect()
        }
    });

    let selected = move || store.advanced_filters.get().indices;

    let all_selected = Memo::new(move |_| {
        let picked = selected();
        let rows = listed.get();
        !rows.is_empty()
            && rows
                .iter()
                .all(|item| picked.iter().any(|code| code == item.code))
    });
    let some_selected = Memo::new(move |_| {
        let picked = selected();
        listed
            .get()
            .iter()
            .any(|item| picked.iter().any(|code| code == item.code))
            && !all_selected.get()
    });

    let toggle_index = move |code: &'static str| {
        store.advanced_filters.update(|filters| {
            if filters.indices.iter().any(|entry| entry == code) {
                filters.indices.retain(|entry| entry != code);
            } else {
                filters.indices.push(code.to_string());
            }
        });
        store.page.set(1);
    };

    let toggle_all = move |_| {
        store.advanced_filters.update(|filters| {
            let codes = listed.get_untracked();
            if codes
                .iter()
                .all(|item| filters.indices.iter().any(|code| code == item.code))
            {
                filters
                    .indices
                    .retain(|code| !codes.iter().any(|item| item.code == code));
            } else {
                for item in codes {
                    if !filters.indices.iter().any(|code| code == item.code) {
                        filters.indices.push(item.code.to_string());
                    }
                }
            }
        });
        store.page.set(1);
    };

    let clear_all = move |_| {
        store
            .advanced_filters
            .update(|filters| filters.indices.clear());
        store.page.set(1);
        open.set(false);
    };

    view! {
        <Show when=move || open.get()>
            <div class="sc-index-drawer" node_ref=host>
                // Header: title + clear-all.
                <div class="sc-index-drawer__head">
                    <span class="sc-index-drawer__title">"Index"</span>
                    <button
                        class=move || {
                            if selected().is_empty() {
                                "sc-index-drawer__clear"
                            } else {
                                "sc-index-drawer__clear sc-index-drawer__clear--on"
                            }
                        }
                        title="Clear all selected indices"
                        disabled=move || selected().is_empty()
                        on:click=clear_all
                    >
                        "✕ Clear"
                    </button>
                </div>

                // Search.
                <div class="sc-index-drawer__search">
                    <input
                        class="sc-index-drawer__search-input"
                        type="text"
                        placeholder="Search indices…"
                        prop:value=move || query.get()
                        on:input=move |ev| {
                            let input = event_target::<web_sys::HtmlInputElement>(&ev);
                            query.set(input.value());
                        }
                    />
                    <Show when=move || !query.get().is_empty()>
                        <HfIconButton
                            icon=x_close()
                            size=12
                            unstyled=true
                            class="sc-index-drawer__search-clear"
                            title="Clear search"
                            on_click=Callback::new(move |_| query.set(String::new()))
                        />
                    </Show>
                </div>

                // Rows.
                <div class="sc-index-drawer__list">
                    <Show when=move || listed.get().is_empty()>
                        <div class="sc-index-drawer__empty">"No indices match your search"</div>
                    </Show>
                    {move || {
                        let picked = selected();
                        listed.get().into_iter().map(|item: &'static IndexItem| {
                            let on = picked.iter().any(|code| code == item.code);
                            view! {
                                <button
                                    class=move || {
                                        if on {
                                            "sc-index-item sc-index-item--sel"
                                        } else {
                                            "sc-index-item"
                                        }
                                    }
                                    on:click=move |_| toggle_index(item.code)
                                >
                                    <span
                                        class=move || {
                                            if on {
                                                "sc-index-item__check sc-index-item__check--on"
                                            } else {
                                                "sc-index-item__check"
                                            }
                                        }
                                    >
                                        "✓"
                                    </span>
                                    <span class="sc-index-item__badge">{item.badge}</span>
                                    <span class="sc-index-item__text">
                                        <span class="sc-index-item__code">{item.code}</span>
                                        <span class="sc-index-item__dot">"•"</span>
                                        <span class="sc-index-item__name">{item.name}</span>
                                    </span>
                                </button>
                            }
                        }).collect_view()
                    }}
                </div>

                // Footer: select-all + selected count.
                <div class="sc-index-drawer__foot">
                    <button
                        class="sc-index-drawer__all"
                        title="Select or clear every visible index"
                        on:click=toggle_all
                    >
                        <span
                            class=move || {
                                if all_selected.get() {
                                    "sc-index-item__check sc-index-item__check--on"
                                } else if some_selected.get() {
                                    "sc-index-item__check sc-index-item__check--mid"
                                } else {
                                    "sc-index-item__check"
                                }
                            }
                        >
                            "✓"
                        </span>
                        <span class="sc-index-drawer__all-label">"Select all"</span>
                    </button>
                    <Show when=move || !selected().is_empty()>
                        <span class="sc-index-drawer__count">
                            {move || format!("{} selected", selected().len())}
                        </span>
                    </Show>
                </div>
            </div>
        </Show>
    }
}

// ---------------------------------------------------------------------------
// SymbolDetailDrawer
// ---------------------------------------------------------------------------

/// Avatar tile hue per first symbol letter (labs `avatarBg` cycle).
fn avatar_class(symbol: &str) -> &'static str {
    let tone = symbol.chars().next().map_or(0, |first| first as usize) % 6;
    match tone {
        0 => "sc-drawer__avatar--c0",
        1 => "sc-drawer__avatar--c1",
        2 => "sc-drawer__avatar--c2",
        3 => "sc-drawer__avatar--c3",
        4 => "sc-drawer__avatar--c4",
        _ => "sc-drawer__avatar--c5",
    }
}

fn rating_class(rating: &str) -> &'static str {
    match rating {
        "Strong Buy" => " sc-rate--strong-buy",
        "Buy" => " sc-rate--buy",
        "Neutral" => " sc-rate--neutral",
        "Sell" => " sc-rate--sell",
        "Strong Sell" => " sc-rate--strong-sell",
        _ => " sc-rate--none",
    }
}

/// The labs compact scale (`1.80 T · 5.40 B · 8.20 Cr · 3.90 M`).
fn compact(num: f64) -> String {
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

/// `(value, width%)` performance bar row (labs `renderPerfRow`).
fn perf_row(label: &'static str, value: f64) -> AnyView {
    let up = value >= 0.0;
    let abs = (value.min(100.0) * 2.5).clamp(4.0, 100.0);
    let pct = if up {
        format!("+{}", percent(value))
    } else {
        percent(value)
    };
    view! {
        <div class="sc-perf-row">
            <span class="sc-perf-row__label">{label}</span>
            <span class="sc-perf-row__track">
                <span
                    class=move || {
                        if up {
                            "sc-perf-row__fill sc-perf-row__fill--up"
                        } else {
                            "sc-perf-row__fill sc-perf-row__fill--down"
                        }
                    }
                    style=format!("width: {abs:.0}%")
                ></span>
            </span>
            <span
                class=move || {
                    if up {
                        "sc-perf-row__val sc-perf-row__val--up"
                    } else {
                        "sc-perf-row__val sc-perf-row__val--down"
                    }
                }
            >
                {pct}
            </span>
        </div>
    }
    .into_any()
}

/// The docked symbol preview panel (labs `SymbolDetailDrawer`).
#[component]
pub fn SymbolDetailDrawer() -> impl IntoView {
    let store = ScreenerStore::take();

    // Resolve the drawn instrument each time the drawer opens.
    let inst = Memo::new(move |_| -> Option<&'static Instrument> {
        if !store.detail_drawer_open.get() {
            return None;
        }
        let symbol = store.active_symbol.get();
        let in_page = store
            .sorted()
            .get_untracked()
            .into_iter()
            .find(|row| row.symbol == symbol);
        in_page.or_else(|| by_symbol(&symbol))
    });

    view! {
        <Show when=move || inst.get().is_some()>
            {move || {
                let Some(inst) = inst.get() else {
                    return ().into_any();
                };
                let shortlisted = move || store.shortlist.get().contains(&inst.symbol);
                let symbol_for_star = inst.symbol.clone();
                let symbol_for_open = inst.symbol.clone();
                let is_up = inst.change_percent >= 0.0;
                let range_pct = inst.range52() * 100.0;

                let overview = move || {
                    let draw_symbol = inst.symbol.clone();
                    view! {
                        // 52-week position bar.
                        <div class="sc-drawer-section">
                            <div class="sc-drawer-section__head">
                                <span class="sc-drawer-section__title">"52-Week Range"</span>
                            </div>
                            <div class="sc-drawer__range">
                                <div class="sc-drawer__range-endpoints">
                                    <span class="sc-drawer__range-endpoint">
                                        <span class="sc-drawer__range-tag">"LOW"</span>
                                        {format!("₹{:.2}", inst.high52.min(inst.low52))}
                                    </span>
                                    <span class="sc-drawer__range-now">
                                        {format!("Now: ₹{:.2}", inst.price)}
                                    </span>
                                    <span class="sc-drawer__range-endpoint">
                                        <span class="sc-drawer__range-tag">{format!("₹{:.2}", inst.high52.max(inst.low52))}</span>
                                    </span>
                                </div>
                                <div class="sc-drawer__range-track">
                                    <span
                                        class="sc-drawer__range-pip"
                                        style=format!("left: {range_pct:.1}%")
                                    ></span>
                                    <span
                                        class="sc-drawer__range-fill"
                                        style=format!("width: {range_pct:.1}%")
                                    ></span>
                                </div>
                            </div>
                        </div>

                        // Key statistics.
                        <div class="sc-drawer-section">
                            <div class="sc-drawer-section__head">
                                <span class="sc-drawer-section__title">"Key statistics"</span>
                            </div>
                            <div class="sc-drawer__stats">
                                <StatCell label="Market cap" value=compact(inst.market_cap) />
                                <StatCell label="P/E (TTM)" value=optional_num(inst.pe) />
                                <StatCell label="Forward P/E" value=optional_num(inst.forward_pe) />
                                <StatCell label="Price to book" value=optional_num(inst.pb) />
                                <StatCell label="EPS (TTM)" value=format!("₹{:.2}", inst.eps) />
                                <StatCell label="Dividend yield" value=format!("{:.2}%", inst.dividend_yield) />
                                <StatCell label="RSI (14)" value=format!("{:.1}", inst.rsi14) />
                                <StatCell label="200 SMA" value=format!("₹{:.2}", inst.sma200) />
                                <StatCell label="ROCE %" value=format!("{:.1}%", inst.roce) />
                                <StatCell label="Debt / equity" value=optional_num(inst.debt_to_equity) />
                            </div>
                        </div>

                        // Mini history preview.
                        <div class="sc-drawer-section">
                            <div class="sc-drawer-section__head">
                                <span class="sc-drawer-section__title">"Spark preview"</span>
                            </div>
                            <SparkPreview symbol=draw_symbol />
                        </div>
                    }.into_any()
                };

                let technicals = move || {
                    view! {
                        <div class="sc-drawer-section">
                            <div class="sc-drawer-section__head">
                                <span class="sc-drawer-section__title">"Technical analysis"</span>
                                <span class=format!("sc-rate{}", rating_class(&inst.technical_rating))>
                                    {inst.technical_rating.clone()}
                                </span>
                            </div>
                            <div class="sc-drawer__gauges">
                                <div class="sc-drawer__gauge-card">
                                    <div class="sc-drawer__gauge-title">"Oscillators"</div>
                                    <div class="sc-drawer__gauge-row">
                                        <span class="sc-drawer__num sc-drawer__num--sell">
                                            {if inst.rsi14 > 70.0 { 2 } else { 0 }}" sell"
                                        </span>
                                        <span class="sc-drawer__num sc-drawer__num--neutral">
                                            {if inst.rsi14 >= 40.0 && inst.rsi14 <= 70.0 { 8 } else { 4 }}" neutral"
                                        </span>
                                        <span class="sc-drawer__num sc-drawer__num--buy">
                                            {if inst.rsi14 < 40.0 { 3 } else { 1 }}" buy"
                                        </span>
                                    </div>
                                </div>
                                <div class="sc-drawer__gauge-card">
                                    <div class="sc-drawer__gauge-title">"Moving averages"</div>
                                    <div class="sc-drawer__gauge-row">
                                        <span class="sc-drawer__num sc-drawer__num--sell">
                                            {if inst.price < inst.sma200 { 5 } else { 1 }}" sell"
                                        </span>
                                        <span class="sc-drawer__num sc-drawer__num--neutral">"1 neutral"</span>
                                        <span class="sc-drawer__num sc-drawer__num--buy">
                                            {if inst.price >= inst.sma200 { 9 } else { 4 }}" buy"
                                        </span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                };

                let financials = move || {
                    view! {
                        <div class="sc-drawer-section">
                            <div class="sc-drawer-section__head">
                                <span class="sc-drawer-section__title">"Performance"</span>
                            </div>
                            <div class="sc-drawer__perf">
                                {perf_row("1 Week", inst.perf1w)}
                                {perf_row("1 Month", inst.perf1m)}
                                {perf_row("3 Months", inst.perf3m)}
                                {perf_row("1 Year", inst.perf1y)}
                            </div>
                        </div>
                        <div class="sc-drawer-section">
                            <div class="sc-drawer-section__head">
                                <span class="sc-drawer-section__title">"Fundamentals"</span>
                            </div>
                            <div class="sc-drawer__stats">
                                <StatCell label="Revenue growth" value=format!("{:.1}%", inst.revenue_growth) />
                                <StatCell label="Net margin" value=format!("{:.1}%", inst.net_margin) />
                                <StatCell label="ROCE" value=format!("{:.1}%", inst.roce) />
                                <StatCell label="ROE" value=format!("{:.1}%", inst.roe) />
                                <StatCell label="Sector" value=if inst.sector.is_empty() { "—".to_string() } else { inst.sector.clone() } />
                                <StatCell label="Industry" value=if inst.industry.is_empty() { "—".to_string() } else { inst.industry.clone() } />
                            </div>
                        </div>
                    }.into_any()
                };

                let body = move || match store.detail_tab.get() {
                    DetailTab::Technicals => technicals(),
                    DetailTab::Financials => financials(),
                    DetailTab::Overview => overview(),
                };

                view! {
                    <aside class="sc-drawer" id="sc-detail-drawer">
                        // Header: avatar + ticker + company + actions.
                        <div class="sc-drawer__head">
                            <span class=format!("sc-drawer__avatar {}", avatar_class(&inst.symbol))>
                                {inst.symbol.chars().take(3).collect::<String>()}
                            </span>
                            <span class="sc-drawer__titles">
                                <span class="sc-drawer__ticker-row">
                                    <span class="sc-drawer__ticker">{inst.symbol.clone()}</span>
                                    <span class="sc-drawer__exchange">{inst.exchange.clone()}</span>
                                </span>
                                <span class="sc-drawer__name">{inst.name.clone()}</span>
                            </span>
                            <span class="sc-drawer__actions">
                                <button
                                    class=move || {
                                        if shortlisted() {
                                            "sc-drawer__action sc-drawer__action--star"
                                        } else {
                                            "sc-drawer__action"
                                        }
                                    }
                                    title=move || {
                                        if shortlisted() { "Remove from shortlist" } else { "Add to shortlist" }
                                    }
                                    on:click=move |_| store.toggle_shortlist(&symbol_for_star)
                                >
                                    {move || if shortlisted() { "★" } else { "☆" }}
                                </button>
                                <button
                                    class="sc-drawer__action"
                                    title="Open in the workbench terminal"
                                    on:click=move |_| {
                                        if let Some(window) = web_sys::window() {
                                            let url = format!("/workbench?symbol={}", symbol_for_open.to_lowercase());
                                            let _ = window.open_with_url_and_target(&url, "_self");
                                        }
                                    }
                                >
                                    "⎋"
                                </button>
                                <button
                                    class="sc-drawer__action"
                                    title="Close panel"
                                    on:click=move |_| store.close_detail()
                                >
                                    "✕"
                                </button>
                            </span>
                        </div>

                        <div class="sc-drawer__body">
                            // Quote banner.
                            <div class="sc-drawer__quote">
                                <div class="sc-drawer__quote-row">
                                    <span class=move || {
                                        if is_up {
                                            "sc-drawer__price sc-drawer__price--up"
                                        } else {
                                            "sc-drawer__price sc-drawer__price--down"
                                        }
                                    }>
                                        {format!("₹{:.2}", inst.price)}
                                    </span>
                                    <span
                                        class=move || {
                                            if is_up {
                                                "sc-drawer__badge sc-drawer__badge--up"
                                            } else {
                                                "sc-drawer__badge sc-drawer__badge--down"
                                            }
                                        }
                                    >
                                        {format!("{:+.2} ({:+.2}%)", inst.change, inst.change_percent)}
                                    </span>
                                </div>
                                <div class="sc-drawer__quote-sub">
                                    <span>{format!("Vol {}", compact(inst.volume))}</span>
                                    <span>"•"</span>
                                    <span>{format!("52W ₹{:.2} – ₹{:.2}", inst.low52, inst.high52)}</span>
                                </div>
                            </div>

                            // Tab strip.
                            <div class="sc-drawer__tabs">
                                <button
                                    class=move || {
                                        if store.detail_tab.get() == DetailTab::Overview {
                                            "sc-drawer__tab sc-drawer__tab--on"
                                        } else {
                                            "sc-drawer__tab"
                                        }
                                    }
                                    on:click=move |_| store.detail_tab.set(DetailTab::Overview)
                                >
                                    "Overview"
                                </button>
                                <button
                                    class=move || {
                                        if store.detail_tab.get() == DetailTab::Technicals {
                                            "sc-drawer__tab sc-drawer__tab--on"
                                        } else {
                                            "sc-drawer__tab"
                                        }
                                    }
                                    on:click=move |_| store.detail_tab.set(DetailTab::Technicals)
                                >
                                    "Technicals"
                                </button>
                                <button
                                    class=move || {
                                        if store.detail_tab.get() == DetailTab::Financials {
                                            "sc-drawer__tab sc-drawer__tab--on"
                                        } else {
                                            "sc-drawer__tab"
                                        }
                                    }
                                    on:click=move |_| store.detail_tab.set(DetailTab::Financials)
                                >
                                    "Financials"
                                </button>
                            </div>

                            {body()}
                        </div>
                    </aside>
                }.into_any()
            }}
        </Show>
    }
}

/// Inline `Show`-safe formatting (kept small so this file stays structural).
fn optional_num(value: f64) -> String {
    if value > 0.0 {
        format!("{:.2}", value)
    } else {
        "—".to_string()
    }
}

#[component]
fn StatCell(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="sc-drawer__stat">
            <span class="sc-drawer__stat-label">{label}</span>
            <span class="sc-drawer__stat-value">{value}</span>
        </div>
    }
}

/// The stored `sparkline` polyline, colored by the last move.
#[component]
fn SparkPreview(symbol: String) -> impl IntoView {
    let points = Memo::new(move |_| {
        by_symbol(&symbol)
            .map(|inst| inst.sparkline.clone())
            .filter(|points| points.len() >= 2)
            .unwrap_or_default()
    });
    let up = Memo::new(move |_| {
        let rows = points.get();
        rows.first().map_or(0.0, |first| *first) <= rows.last().map_or(0.0, |last| *last)
    });

    view! {
        <div class="sc-drawer__spark">
            <svg width="100%" height="64" viewBox="0 0 300 64" preserveAspectRatio="none" fill="none">
                {move || {
                    let rows = points.get();
                    if rows.len() < 2 {
                        return ().into_any();
                    }
                    let min = rows.iter().cloned().fold(f64::INFINITY, f64::min);
                    let max = rows.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    let span = (max - min).abs().max(1e-6);
                    let step = 300.0 / (rows.len() - 1) as f64;
                    let coords = rows
                        .iter()
                        .enumerate()
                        .map(|(position, point)| {
                            format!("{:.1},{:.1}", position as f64 * step, 62.0 - ((point - min) / span) * 56.0)
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    let color = if up.get() { "var(--pnl-gain)" } else { "var(--pnl-loss)" };
                    view! { <polyline points=coords stroke=color stroke-width="1.4" stroke-linejoin="round" /> }
                        .into_any()
                }}
            </svg>
            <span class="sc-drawer__spark-foot">
                {move || format!("{} points", points.get().len())}
            </span>
        </div>
    }
}
