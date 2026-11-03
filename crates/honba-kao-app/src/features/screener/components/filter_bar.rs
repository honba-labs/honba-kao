//! `ScreenerFilterBar` — four-row command bar above the screener results:
//! asset segment + preset chips, screener-view tab strip, search/filter row,
//! and view-mode toggles + modal triggers.

use crate::features::screener::types::ScreenerViewMode;
use honba_kao_ui_core::atoms::segmented::Segment;
use honba_kao_ui_core::primitive::{ButtonSize, ButtonVariant, HfButton};
use honba_kao_ui_core::{
    HfCheckbox, HfChip, HfInput, HfKbdBadge, HfSegmentedControl, HfSelect, HfTabPanel,
};
use leptos::prelude::*;

#[component]
pub fn ScreenerFilterBar(
    asset_type: RwSignal<String>,
    active_tab: RwSignal<String>,
    search_query: RwSignal<String>,
    selected_exchange: RwSignal<String>,
    selected_sector: RwSignal<String>,
    sector_options: Vec<(String, String)>,
    selected_market_cap: RwSignal<String>,
    watchlist_only: RwSignal<bool>,
    fno_only: RwSignal<bool>,
    view_mode: RwSignal<ScreenerViewMode>,
    filter_modal_open: RwSignal<bool>,
    column_modal_open: RwSignal<bool>,
    active_filters: Signal<usize>,
    on_reset: Callback<()>,
) -> impl IntoView {
    // (label, value) tuples — HfSelect renders `<option value=value>{label}</option>`.
    let exchange_options: Vec<(String, String)> = vec![
        ("All".to_string(), "ALL".to_string()),
        ("NSE".to_string(), "NSE".to_string()),
        ("BSE".to_string(), "BSE".to_string()),
        ("NASDAQ".to_string(), "NASDAQ".to_string()),
        ("NYSE".to_string(), "NYSE".to_string()),
    ];
    let cap_options: Vec<(String, String)> = vec![
        ("All".to_string(), "ALL".to_string()),
        ("Large Cap".to_string(), "Large Cap".to_string()),
        ("Mid Cap".to_string(), "Mid Cap".to_string()),
        ("Small Cap".to_string(), "Small Cap".to_string()),
    ];

    view! {
        <div class="hf-filter-bar screener-filter-bar flex flex-col gap-2 px-3 py-2 border-b border-border-soft bg-surface-base">
            // Row A — asset segment + inert preset showcase (Phase 4+ wiring).
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
                    <HfChip active=false on_click=Callback::new(|_| {})>
                        "✦ AI Bar (Phase 4)"
                    </HfChip>
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

            // Row C — search + select filters + universe checkboxes.
            <div class="flex items-center gap-2 flex-wrap">
                <div class="w-56">
                    <HfInput
                        value=search_query
                        placeholder="🔍 Ticker / name"
                        on_input=Callback::new(move |v| search_query.set(v))
                    />
                </div>
                <div class="w-36">
                    <HfSelect value=selected_exchange options=exchange_options />
                </div>
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
                    on_click=on_reset
                >
                    "Reset"
                </HfButton>
                <span class="ml-auto flex items-center gap-1">
                    <HfKbdBadge key="." />
                    <HfKbdBadge key="Ctrl K" />
                </span>
            </div>
        </div>
    }
}
