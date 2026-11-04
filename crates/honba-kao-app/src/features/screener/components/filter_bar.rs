//! `ScreenerFilterBar` — four-row command bar above the screener results:
//! asset segment + ✦ AI prompt bar, screener-view tab strip, search + flyout
//! filter row (Market / Index / Price / P-E pills), and view-mode toggles
//! with modal triggers and hotkey badges.

use crate::features::screener::indices::{INDEX_CATALOG, IndexClass, benchmark_for, member_count};
use crate::features::screener::screen_catalog::{CustomScreen, builtin_screens};
use crate::features::screener::types::ScreenerViewMode;
use honba_kao_ui_core::atoms::segmented::Segment;
use honba_kao_ui_core::primitive::{ButtonSize, ButtonVariant, HfButton};
use honba_kao_ui_core::{
    HfCheckbox, HfChip, HfInput, HfKbdBadge, HfNumberInput, HfSegmentedControl, HfSelect,
    HfTabPanel,
};
use leptos::prelude::*;
use rust_decimal::Decimal;

/// Market flyout rows: (display label, exchange value).
const EXCHANGE_ROWS: &[(&str, &str)] = &[
    ("All", "ALL"),
    ("NSE", "NSE"),
    ("BSE", "BSE"),
    ("NASDAQ", "NASDAQ"),
    ("NYSE", "NYSE"),
];

/// Which Row-C flyout popover is open — at most one at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlyoutKey {
    Market,
    Index,
    Price,
    Pe,
}

/// Pill band label: "min – max" / "≥ min" / "≤ max" / "Any" (`ZERO` = unset).
fn decimal_band(min_value: Decimal, max_value: Decimal) -> String {
    let min_set = min_value > Decimal::ZERO;
    let max_set = max_value > Decimal::ZERO;
    match (min_set, max_set) {
        (true, true) => format!("{min_value} – {max_value}"),
        (true, false) => format!("≥ {min_value}"),
        (false, true) => format!("≤ {max_value}"),
        (false, false) => "Any".to_string(),
    }
}

/// Shared pill button for one Row-C flyout; toggles its own key (single-open).
#[component]
fn FlyoutPill(
    label: Signal<String>,
    open: RwSignal<Option<FlyoutKey>>,
    key: FlyoutKey,
    #[prop(into, optional)] is_active: Option<Signal<bool>>,
) -> impl IntoView {
    let active = move || is_active.map(|a| a.get()).unwrap_or(false) || open.get() == Some(key);
    let class = move || {
        format!(
            "hf-filter-pill cursor-pointer select-none transition-all duration-150 inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium border {}",
            if active() {
                "hf-filter-pill--active bg-emerald-600/20 border-emerald-500 text-emerald-300"
            } else {
                "border-border-soft bg-surface-subtle text-text-muted hover:text-text-high hover:border-border-strong"
            }
        )
    };
    view! {
        <button
            type="button"
            class=class
            on:click=move |_| {
                let next = if open.get_untracked() == Some(key) { None } else { Some(key) };
                open.set(next);
            }
        >
            <span>{move || label.get()}</span>
            <span class="text-[9px] text-text-muted">"▾"</span>
        </button>
    }
}

#[component]
pub fn ScreenerFilterBar(
    asset_type: RwSignal<String>,
    active_tab: RwSignal<String>,
    search_query: RwSignal<String>,
    selected_exchange: RwSignal<String>,
    selected_index: RwSignal<String>,
    selected_sector: RwSignal<String>,
    sector_options: Vec<(String, String)>,
    selected_market_cap: RwSignal<String>,
    watchlist_only: RwSignal<bool>,
    fno_only: RwSignal<bool>,
    min_price: RwSignal<Decimal>,
    max_price: RwSignal<Decimal>,
    min_pe: RwSignal<Decimal>,
    max_pe: RwSignal<Decimal>,
    ai_prompt: RwSignal<String>,
    ai_label: RwSignal<String>,
    view_mode: RwSignal<ScreenerViewMode>,
    filter_modal_open: RwSignal<bool>,
    column_modal_open: RwSignal<bool>,
    on_ai_apply: Callback<String>,
    active_filters: Signal<usize>,
    on_reset: Callback<()>,
    on_export_csv: Callback<()>,
    #[prop(into, optional)] on_apply_screen: Option<Callback<CustomScreen>>,
    #[prop(into, optional)] active_screen_id: Option<RwSignal<String>>,
) -> impl IntoView {
    // (label, value) tuples — HfSelect renders `<option value=value>{label}</option>`.
    let cap_options: Vec<(String, String)> = vec![
        ("All".to_string(), "ALL".to_string()),
        ("Large Cap".to_string(), "Large Cap".to_string()),
        ("Mid Cap".to_string(), "Mid Cap".to_string()),
        ("Small Cap".to_string(), "Small Cap".to_string()),
    ];

    // Row C flyout state — the pills flip their own key (single-open toggle).
    let flyout_open = RwSignal::new(Option::<FlyoutKey>::None);
    let show_ai_prompt = RwSignal::new(false);
    let active_screen =
        active_screen_id.unwrap_or_else(|| RwSignal::new("builtin.all".to_string()));
    let presets = builtin_screens();

    // Pill labels derived from the SAME criteria signals the modal drives.
    let market_label: Signal<String> = Signal::derive(move || {
        let value = selected_exchange.get();
        let name = match value.as_str() {
            "ALL" => "All",
            other => other,
        };
        format!("Market: {name}")
    });
    let index_label: Signal<String> = Signal::derive(move || {
        let key = selected_index.get();
        if key.is_empty() || key == "ALL" {
            "Any".to_string()
        } else if let Some(benchmark) = benchmark_for(&key) {
            benchmark.label.to_string()
        } else {
            "Any".to_string()
        }
    });
    let price_label: Signal<String> = Signal::derive(move || {
        format!("Price: {}", decimal_band(min_price.get(), max_price.get()))
    });
    let pe_label: Signal<String> =
        Signal::derive(move || format!("P/E: {}", decimal_band(min_pe.get(), max_pe.get())));

    let market_active = Signal::derive(move || selected_exchange.get() != "ALL");
    let index_active = Signal::derive(move || {
        let key = selected_index.get();
        !key.is_empty() && key != "ALL"
    });
    let price_active =
        Signal::derive(move || min_price.get() > Decimal::ZERO || max_price.get() > Decimal::ZERO);
    let pe_active =
        Signal::derive(move || min_pe.get() > Decimal::ZERO || max_pe.get() > Decimal::ZERO);

    view! {
        <div class="hf-filter-bar screener-filter-bar hf-card flex flex-col gap-2.5 p-3.5 border border-border-soft bg-surface-base rounded-xl shadow-sm mb-3">
            // Row A — Asset Segment + Popular Screens bubbles (honba-fe inspired).
            <div class="flex items-center justify-between gap-3 flex-wrap">
                <div class="flex items-center gap-2 flex-wrap min-w-0 flex-1">
                    <HfSegmentedControl
                        value=asset_type
                        size="px-2.5 py-1"
                        options=vec![
                            Segment {
                                value: "stocks".to_string(),
                                label: "Stocks".to_string(),
                            },
                            Segment {
                                value: "etf".to_string(),
                                label: "ETF".to_string(),
                            },
                            Segment {
                                value: "bonds".to_string(),
                                label: "Bonds".to_string(),
                            },
                            Segment {
                                value: "mf".to_string(),
                                label: "MF".to_string(),
                            },
                        ]
                    />
                    <div class="h-4 w-px bg-border-soft mx-1 hidden sm:block" />
                    <div class="tv-popular-screens-container flex items-center gap-2 overflow-x-auto py-0.5">
                        <span class="text-[10px] font-bold uppercase tracking-wider text-text-muted whitespace-nowrap">
                            "Popular Screens:"
                        </span>
                        <div class="tv-popular-bubbles-list flex items-center gap-1.5 flex-nowrap">
                            {move || {
                                let current = active_screen.get();
                                presets
                                    .iter()
                                    .map(|sc| {
                                        let is_active = current == sc.id;
                                        let sc_clone = sc.clone();
                                        let sc_id = sc.id.clone();
                                        let on_click = Callback::new(move |_| {
                                            active_screen.set(sc_id.clone());
                                            if let Some(cb) = on_apply_screen.as_ref() {
                                                cb.run(sc_clone.clone());
                                            }
                                        });
                                        let sc_title = sc.name.clone();
                                        view! {
                                            <HfChip
                                                active=is_active
                                                on_click=on_click
                                            >
                                                <span>{sc_title}</span>
                                            </HfChip>
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            }}
                        </div>
                    </div>
                </div>
            </div>

            // Row B — screener view tab strip (HfTabPanel updates `active` itself).
            <div class="hf-tab-strip flex items-center gap-1 flex-wrap">
                <HfTabPanel value="overview" label="Overview" active=active_tab />
                <HfTabPanel value="performance" label="Performance" active=active_tab />
                <HfTabPanel value="technicals" label="Technicals" active=active_tab />
                <HfTabPanel value="valuation" label="Valuation" active=active_tab />
                <HfTabPanel value="dividends" label="Dividends" active=active_tab />
                <HfTabPanel value="profitability" label="Profitability" active=active_tab />
                <HfTabPanel value="financials" label="Financials" active=active_tab />
                <HfTabPanel
                    value="income_statement"
                    label="Income Statement"
                    active=active_tab
                />
                <HfTabPanel value="balance_sheet" label="Balance Sheet" active=active_tab />
                <HfTabPanel value="cash_flow" label="Cash Flow" active=active_tab />
                <HfTabPanel value="per_share" label="Per Share" active=active_tab />
                <HfTabPanel value="forecasts" label="Forecasts" active=active_tab />
            </div>

            // Row C — search + AI trigger + Market/Index/Price/P-E flyouts + select filters.
            <div class="flex items-center gap-2 flex-wrap">
                <div class="w-52">
                    <HfInput
                        value=search_query
                        placeholder="🔍 Ticker / name"
                        on_input=Callback::new(move |v| search_query.set(v))
                    />
                </div>
                <button
                    type="button"
                    class=move || format!(
                        "hf-filter-pill cursor-pointer select-none transition-all duration-150 inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium border {}",
                        if show_ai_prompt.get() || !ai_label.get().is_empty() {
                            "bg-purple-600/20 border-purple-500 text-purple-300 shadow-sm"
                        } else {
                            "border-border-soft bg-surface-subtle text-purple-400 hover:text-purple-300 hover:border-purple-500/50"
                        }
                    )
                    on:click=move |_| show_ai_prompt.update(|v| *v = !*v)
                    title="Filter stocks using AI natural language prompt"
                >
                    <span class="text-xs">"✦"</span>
                    <span>"AI"</span>
                </button>
                <span class="relative">
                    <FlyoutPill label=market_label open=flyout_open key=FlyoutKey::Market is_active=market_active />
                    {move || {
                        (flyout_open.get() == Some(FlyoutKey::Market)).then(|| {
                            view! {
                                <div class="hf-menu-popover absolute left-0 top-full z-40 mt-1.5 w-64 rounded-xl border border-border-strong bg-surface-base p-2.5 shadow-2xl flex flex-col gap-1 text-xs text-text-high">
                                    {EXCHANGE_ROWS
                                        .iter()
                                        .map(|(label, value)| {
                                            let active =
                                                selected_exchange.get().as_str() == *value;
                                            let row_class = if active {
                                                "hf-menu-item text-emerald-400 font-semibold bg-emerald-500/15 rounded-md px-2.5 py-1.5 text-left transition-colors"
                                            } else {
                                                "hf-menu-item text-text-high hover:bg-surface-subtle hover:text-emerald-400 rounded-md px-2.5 py-1.5 text-left transition-colors"
                                            };
                                            view! {
                                                <button
                                                    type="button"
                                                    class=row_class
                                                    on:click=move |_| {
                                                        selected_exchange
                                                             .set((*value).to_string());
                                                        flyout_open.set(None);
                                                    }
                                                >
                                                    {*label}
                                                </button>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                            </div>
                            }
                        })
                    }}
                </span>
                <span class="relative">
                    <FlyoutPill label=index_label open=flyout_open key=FlyoutKey::Index is_active=index_active />
                    {move || {
                        (flyout_open.get() == Some(FlyoutKey::Index)).then(|| {
                            let key = selected_index.get();
                            let all_active = key.is_empty() || key == "ALL";
                            let all_class = if all_active {
                                "hf-menu-item text-emerald-400 font-semibold bg-emerald-500/15 rounded-md px-2.5 py-1.5 text-left transition-colors"
                            } else {
                                "hf-menu-item text-text-high hover:bg-surface-subtle hover:text-emerald-400 rounded-md px-2.5 py-1.5 text-left transition-colors"
                            };
                            view! {
                                <div class="hf-menu-popover absolute left-0 top-full z-40 mt-1.5 w-64 rounded-xl border border-border-strong bg-surface-base p-2.5 shadow-2xl flex flex-col gap-1 text-xs text-text-high">
                                    <button
                                        type="button"
                                        class=all_class
                                        on:click=move |_| {
                                            selected_index.set(String::new());
                                            flyout_open.set(None);
                                        }
                                    >
                                        "All Indices"
                                    </button>
                                    <span class="text-[10px] uppercase tracking-wide text-text-muted pt-1 px-1">
                                        "Broad"
                                    </span>
                                    {INDEX_CATALOG
                                        .iter()
                                        .filter(|b| b.class == IndexClass::Broad)
                                        .map(|b| {
                                            let b_key = b.key;
                                            let active = key == b_key;
                                            let text =
                                                format!("{} ({})", b.label, member_count(b_key));
                                            let row_class = if active {
                                                "hf-menu-item text-emerald-400 font-semibold bg-emerald-500/15 rounded-md px-2.5 py-1.5 text-left transition-colors"
                                            } else {
                                                "hf-menu-item text-text-high hover:bg-surface-subtle hover:text-emerald-400 rounded-md px-2.5 py-1.5 text-left transition-colors"
                                            };
                                            view! {
                                                <button
                                                    type="button"
                                                    class=row_class
                                                    on:click=move |_| {
                                                        selected_index.set(b_key.to_string());
                                                        flyout_open.set(None);
                                                    }
                                                >
                                                    {text}
                                                </button>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                    <span class="text-[10px] uppercase tracking-wide text-text-muted pt-1 px-1">
                                        "Sectoral"
                                    </span>
                                    {INDEX_CATALOG
                                        .iter()
                                        .filter(|b| b.class == IndexClass::Sectoral)
                                        .map(|b| {
                                            let b_key = b.key;
                                            let active = key == b_key;
                                            let text =
                                                format!("{} ({})", b.label, member_count(b_key));
                                            let row_class = if active {
                                                "hf-menu-item text-emerald-400 font-semibold bg-emerald-500/15 rounded-md px-2.5 py-1.5 text-left transition-colors"
                                            } else {
                                                "hf-menu-item text-text-high hover:bg-surface-subtle hover:text-emerald-400 rounded-md px-2.5 py-1.5 text-left transition-colors"
                                            };
                                            view! {
                                                <button
                                                    type="button"
                                                    class=row_class
                                                    on:click=move |_| {
                                                        selected_index.set(b_key.to_string());
                                                        flyout_open.set(None);
                                                    }
                                                >
                                                    {text}
                                                </button>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </div>
                            }
                        })
                    }}
                </span>
                <span class="relative">
                    <FlyoutPill label=price_label open=flyout_open key=FlyoutKey::Price is_active=price_active />
                    {move || {
                        (flyout_open.get() == Some(FlyoutKey::Price)).then(|| {
                            view! {
                                <div class="hf-menu-popover absolute left-0 top-full z-40 mt-1.5 w-64 rounded-xl border border-border-strong bg-surface-base p-3 shadow-2xl flex flex-col gap-1.5 text-xs text-text-high">
                                    <span class="text-[10px] uppercase tracking-wide text-text-muted">
                                        "Price Band ₹ (dec! 0 clears)"
                                    </span>
                                    <div class="grid grid-cols-2 gap-2">
                                        <div class="flex flex-col gap-0.5">
                                            <span class="text-[10px] text-text-muted">"Min"</span>
                                            <HfNumberInput value=min_price placeholder="0" />
                                        </div>
                                        <div class="flex flex-col gap-0.5">
                                            <span class="text-[10px] text-text-muted">"Max"</span>
                                            <HfNumberInput value=max_price placeholder="0" />
                                        </div>
                                    </div>
                                    <span class="text-[10px] text-text-muted">
                                        "0 clears the bound. Shared with the Filters modal."
                                    </span>
                                </div>
                            }
                        })
                    }}
                </span>
                <span class="relative">
                    <FlyoutPill label=pe_label open=flyout_open key=FlyoutKey::Pe is_active=pe_active />
                    {move || {
                        (flyout_open.get() == Some(FlyoutKey::Pe)).then(|| {
                            view! {
                                <div class="hf-menu-popover absolute left-0 top-full z-40 mt-1.5 w-64 rounded-xl border border-border-strong bg-surface-base p-3 shadow-2xl flex flex-col gap-1.5 text-xs text-text-high">
                                    <span class="text-[10px] uppercase tracking-wide text-text-muted">
                                        "P/E Band"
                                    </span>
                                    <div class="grid grid-cols-2 gap-2">
                                        <div class="flex flex-col gap-0.5">
                                            <span class="text-[10px] text-text-muted">"Min"</span>
                                            <HfNumberInput value=min_pe placeholder="0" />
                                        </div>
                                        <div class="flex flex-col gap-0.5">
                                            <span class="text-[10px] text-text-muted">"Max"</span>
                                            <HfNumberInput value=max_pe placeholder="0" />
                                        </div>
                                    </div>
                                    <span class="text-[10px] text-text-muted">
                                        "0 clears the bound. Shared with the Filters modal."
                                    </span>
                                </div>
                            }
                        })
                    }}
                </span>
                <div class="w-44">
                    <HfSelect value=selected_sector options=sector_options />
                </div>
                <div class="w-40">
                    <HfSelect value=selected_market_cap options=cap_options />
                </div>
                <HfCheckbox checked=watchlist_only label="★ Only" />
                <HfCheckbox checked=fno_only label="F&O" />
                {move || {
                    let label = ai_label.get();
                    (!label.is_empty()).then(|| {
                        view! {
                            <button
                                type="button"
                                title="AI filter applied — click to dismiss the indicator"
                                class="px-2.5 py-0.5 rounded-full border border-purple-500/50 bg-purple-500/15 text-[11px] text-purple-300 font-medium flex items-center gap-1 hover:bg-purple-500/25 transition-colors"
                                on:click=move |_| {
                                    ai_label.set(String::new());
                                    ai_prompt.set(String::new());
                                }
                            >
                                <span>{label}</span>
                                <span class="text-[9px]">"✕"</span>
                            </button>
                        }
                    })
                }}
            </div>

            // Expandable AI prompt row (when toggled via ✦ AI button)
            {move || {
                show_ai_prompt.get().then(|| {
                    view! {
                        <div class="flex items-center gap-2 p-2 rounded-xl border border-purple-500/40 bg-surface-subtle shadow-inner">
                            <span class="text-purple-400 pl-2 font-mono text-sm">"✦"</span>
                            <div class="flex-1">
                                <HfInput
                                    value=ai_prompt
                                    placeholder="✦ Ask AI: e.g. 'pe < 25, rvol > 2, nifty 50' or 'large cap tech stocks'"
                                    on_input=Callback::new(move |v| ai_prompt.set(v))
                                />
                            </div>
                            <HfButton
                                variant=ButtonVariant::Primary
                                size=ButtonSize::Xs
                                on_click=Callback::new(move |_| {
                                    on_ai_apply.run(ai_prompt.get_untracked());
                                    show_ai_prompt.set(false);
                                })
                            >
                                "Apply AI"
                            </HfButton>
                        </div>
                    }
                })
            }}

            // Row D — view mode toggles, modal triggers, hotkey badges.
            <div class="flex items-center gap-2 flex-wrap border-t border-border-soft pt-2.5">
                <button
                    type="button"
                    class=move || {
                        format!(
                            "hf-view-toggle px-3 py-1 rounded-md border text-xs font-medium transition-colors {}",
                            if matches!(view_mode.get(), ScreenerViewMode::Table) {
                                "bg-emerald-600 text-white border-emerald-600 shadow-sm"
                            } else {
                                "border-border-soft text-text-muted hover:bg-surface-subtle hover:text-text-high"
                            }
                        )
                    }
                    on:click=move |_| view_mode.set(ScreenerViewMode::Table)
                >
                    "▦ Grid"
                </button>
                <button
                    type="button"
                    class=move || {
                        format!(
                            "hf-view-toggle px-3 py-1 rounded-md border text-xs font-medium transition-colors {}",
                            if matches!(view_mode.get(), ScreenerViewMode::Heatmap) {
                                "bg-emerald-600 text-white border-emerald-600 shadow-sm"
                            } else {
                                "border-border-soft text-text-muted hover:bg-surface-subtle hover:text-text-high"
                            }
                        )
                    }
                    on:click=move |_| view_mode.set(ScreenerViewMode::Heatmap)
                >
                    "☷ Heatmap"
                </button>
                <HfButton
                    variant=ButtonVariant::Tertiary
                    size=ButtonSize::Xs
                    on_click=Callback::new(move |_| filter_modal_open.set(true))
                >
                    {move || format!("Filters ({})", active_filters.get())}
                </HfButton>
                <HfButton
                    variant=ButtonVariant::Tertiary
                    size=ButtonSize::Xs
                    on_click=Callback::new(move |_| column_modal_open.set(true))
                >
                    "Columns"
                </HfButton>
                <HfButton
                    variant=ButtonVariant::Tertiary
                    size=ButtonSize::Xs
                    on_click=on_export_csv
                >
                    "Export CSV"
                </HfButton>
                <HfButton
                    variant=ButtonVariant::Tertiary
                    size=ButtonSize::Xs
                    on_click=on_reset
                >
                    "Reset"
                </HfButton>
                <span class="ml-auto flex items-center gap-1">
                    <HfKbdBadge key="." />
                    <HfKbdBadge key="Shift N" />
                    <HfKbdBadge key="Ctrl K" />
                </span>
            </div>
        </div>
    }
}
