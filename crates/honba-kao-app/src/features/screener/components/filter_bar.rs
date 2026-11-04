//! `ScreenerFilterBar` — four-row command bar above the screener results:
//! asset segment + ✦ AI prompt bar, screener-view tab strip, search + flyout
//! filter row (Market / Index / Price / P-E pills), and view-mode toggles
//! with modal triggers and hotkey badges.

use crate::features::screener::indices::{INDEX_CATALOG, IndexClass, benchmark_for, member_count};
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
) -> impl IntoView {
    view! {
        <HfButton
            variant=ButtonVariant::Tertiary
            size=ButtonSize::Xs
            on_click=Callback::new(move |_| {
                let next = if open.get_untracked() == Some(key) { None } else { Some(key) };
                open.set(next);
            })
        >
            {move || format!("{} ▾", label.get())}
        </HfButton>
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

    view! {
        <div class="hf-filter-bar screener-filter-bar flex flex-col gap-2 px-3 py-2 border-b border-border-soft bg-surface-base">
            // Row A — asset segment, preset showcase chips, and the ✦ AI prompt bar.
            <div class="flex items-center gap-2 flex-wrap">
                <HfSegmentedControl
                    value=asset_type
                    size="px-2 py-0.5"
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
                <span class="flex items-center gap-1 flex-wrap">
                    <HfChip active=false on_click=Callback::new(|_| {})>"Breakouts"</HfChip>
                    <HfChip active=false on_click=Callback::new(|_| {})>"52W High"</HfChip>
                    <HfChip active=false on_click=Callback::new(|_| {})>"NR7 Squeeze"</HfChip>
                    <HfChip active=false on_click=Callback::new(|_| {})>"Value"</HfChip>
                    <HfChip active=false on_click=Callback::new(|_| {})>"High Div"</HfChip>
                </span>
                <span class="flex items-center gap-1">
                    <span class="w-72">
                        <HfInput
                            value=ai_prompt
                            placeholder="✦ Try: pe < 25, rvol > 2, nifty 50"
                            on_input=Callback::new(move |v| ai_prompt.set(v))
                        />
                    </span>
                    <HfButton
                        variant=ButtonVariant::Tertiary
                        size=ButtonSize::Xs
                        on_click=Callback::new(move |_| on_ai_apply.run(ai_prompt.get_untracked()))
                    >
                        "✦ Parse"
                    </HfButton>
                    {move || {
                        let label = ai_label.get();
                        (!label.is_empty()).then(|| {
                            view! {
                                <button
                                    type="button"
                                    title="AI filter applied — click to dismiss the indicator"
                                    class="px-2 py-0.5 rounded-full border border-emerald-600 bg-emerald-600/10 text-[11px] text-emerald-500"
                                    on:click=move |_| {
                                        ai_label.set(String::new());
                                        ai_prompt.set(String::new());
                                    }
                                >
                                    {label}
                                </button>
                            }
                        })
                    }}
                </span>
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

            // Row C — search + Market/Index/Price/P-E flyouts + select filters.
            <div class="flex items-center gap-2 flex-wrap">
                <div class="w-56">
                    <HfInput
                        value=search_query
                        placeholder="🔍 Ticker / name"
                        on_input=Callback::new(move |v| search_query.set(v))
                    />
                </div>
                <span class="relative">
                    <FlyoutPill label=market_label open=flyout_open key=FlyoutKey::Market />
                    {move || {
                        (flyout_open.get() == Some(FlyoutKey::Market)).then(|| {
                            view! {
                                <div class="absolute left-0 top-full z-30 mt-1 w-64 rounded border border-border-soft bg-surface-base p-3 shadow-lg shadow-black/20 flex flex-col gap-1 text-[11px] text-text-high">
                                    {EXCHANGE_ROWS
                                        .iter()
                                        .map(|(label, value)| {
                                            let active =
                                                selected_exchange.get().as_str() == *value;
                                            let row_class = if active {
                                                "text-emerald-500 font-semibold"
                                            } else {
                                                "hover:bg-surface-subtle text-left"
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
                    <FlyoutPill label=index_label open=flyout_open key=FlyoutKey::Index />
                    {move || {
                        (flyout_open.get() == Some(FlyoutKey::Index)).then(|| {
                            let key = selected_index.get();
                            let all_active = key.is_empty() || key == "ALL";
                            let all_class = if all_active {
                                "text-emerald-500 font-semibold"
                            } else {
                                "hover:bg-surface-subtle text-left"
                            };
                            view! {
                                <div class="absolute left-0 top-full z-30 mt-1 w-64 rounded border border-border-soft bg-surface-base p-3 shadow-lg shadow-black/20 flex flex-col gap-1 text-[11px] text-text-high">
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
                                    <span class="text-[10px] uppercase tracking-wide text-text-muted pt-1">
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
                                                "text-emerald-500 font-semibold"
                                            } else {
                                                "hover:bg-surface-subtle text-left"
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
                                    <span class="text-[10px] uppercase tracking-wide text-text-muted pt-1">
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
                                                "text-emerald-500 font-semibold"
                                            } else {
                                                "hover:bg-surface-subtle text-left"
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
                    <FlyoutPill label=price_label open=flyout_open key=FlyoutKey::Price />
                    {move || {
                        (flyout_open.get() == Some(FlyoutKey::Price)).then(|| {
                            view! {
                                <div class="absolute left-0 top-full z-30 mt-1 w-64 rounded border border-border-soft bg-surface-base p-3 shadow-lg shadow-black/20 flex flex-col gap-1 text-[11px] text-text-high">
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
                                    <span class="text-text-muted">
                                        "0 clears the bound. Shared with the Filters modal."
                                    </span>
                                </div>
                            }
                        })
                    }}
                </span>
                <span class="relative">
                    <FlyoutPill label=pe_label open=flyout_open key=FlyoutKey::Pe />
                    {move || {
                        (flyout_open.get() == Some(FlyoutKey::Pe)).then(|| {
                            view! {
                                <div class="absolute left-0 top-full z-30 mt-1 w-64 rounded border border-border-soft bg-surface-base p-3 shadow-lg shadow-black/20 flex flex-col gap-1 text-[11px] text-text-high">
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
                                    <span class="text-text-muted">
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
            </div>

            // Row D — view mode toggles, modal triggers, hotkey badges.
            <div class="flex items-center gap-2 flex-wrap border-t border-border-soft pt-2">
                <button
                    type="button"
                    class=move || {
                        format!(
                            "hf-view-toggle px-2 py-0.5 rounded border text-[11px] font-medium transition-colors {}",
                            if matches!(view_mode.get(), ScreenerViewMode::Table) {
                                "bg-emerald-600 text-white border-emerald-600"
                            } else {
                                "border-border-soft text-text-muted hover:bg-surface-subtle"
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
                            "hf-view-toggle px-2 py-0.5 rounded border text-[11px] font-medium transition-colors {}",
                            if matches!(view_mode.get(), ScreenerViewMode::Heatmap) {
                                "bg-emerald-600 text-white border-emerald-600"
                            } else {
                                "border-border-soft text-text-muted hover:bg-surface-subtle"
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
