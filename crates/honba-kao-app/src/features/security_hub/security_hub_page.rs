//! Security Hub page (Phase 2.1 core).
//!
//! The DES header (ticker, LTP, change, session range, placeholder valuation
//! ratios) sits above an 11-tab row. The tab is carried in `?tab=` so the
//! command bar's `security_tab_for` mapping deep-links straight into a tab.
//! Only Overview has content; the other tabs are "Coming soon" placeholders.

use honba_kao_ui_core::{
    HfKpi, HfMiniRangeBar, HfPLNumber, HfPLNumberFormat, HfPageHeader, HfPanel, HfPanelHeader,
    HfTabs, HfTabsList, HfTabsTrigger,
};
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};

use crate::shared::command::symbols::{MockSymbol, lookup};

/// Tab ids, in display order. Ids match `security_tab_for` in `commanding.rs`
/// and the `tab=` values the command bar emits.
const TABS: &[(&str, &str)] = &[
    ("overview", "Overview"),
    ("financials", "Financials"),
    ("chart", "Chart"),
    ("news", "News"),
    ("ownership", "Ownership"),
    ("estimates", "Estimates"),
    ("peers", "Peers"),
    ("esg", "ESG"),
    ("tape", "Tape"),
    ("insider", "Insider"),
    ("filings", "Filings"),
];

const DEFAULT_TAB: &str = "overview";

/// Resolve a raw `tab=` value to a known tab id, falling back to Overview.
fn resolve_tab(raw: Option<String>) -> &'static str {
    let raw = raw.unwrap_or_default();
    TABS.iter()
        .find(|(id, _)| id.eq_ignore_ascii_case(raw.trim()))
        .map(|(id, _)| *id)
        .unwrap_or(DEFAULT_TAB)
}

fn tab_label(id: &str) -> &'static str {
    TABS.iter()
        .find(|(tab, _)| *tab == id)
        .map(|(_, label)| *label)
        .unwrap_or("Overview")
}

fn parse_f64(raw: &str) -> f64 {
    raw.parse::<f64>().unwrap_or(0.0)
}

/// Session low/high from the mock intraday series (placeholder until a real
/// quote feed supplies the day range).
fn session_range(symbol: &MockSymbol) -> (f64, f64) {
    symbol
        .series
        .iter()
        .copied()
        .fold(None, |acc: Option<(f64, f64)>, v| match acc {
            None => Some((v, v)),
            Some((lo, hi)) => Some((lo.min(v), hi.max(v))),
        })
        .unwrap_or((0.0, 0.0))
}

#[component]
pub fn SecurityHubPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let navigate = StoredValue::new(use_navigate());

    let ticker = Memo::new(move |_| {
        params
            .get()
            .get("ticker")
            .unwrap_or_default()
            .trim()
            .to_ascii_uppercase()
    });
    let active_tab = Memo::new(move |_| resolve_tab(query.get().get("tab")));

    // Shared tab state for the HfTabs triggers. HfTabs does not provide its
    // own context, so the page provides it here and keeps it in sync with the
    // URL in both directions.
    let active_signal = RwSignal::new(active_tab.get_untracked().to_string());
    provide_context(active_signal);
    Effect::new(move |_| {
        active_signal.set(active_tab.get().to_string());
    });

    let symbol = Memo::new(move |_| lookup(&ticker.get()));

    let go_tab = move |id: &'static str| {
        let target = format!("/equity/security/{}?tab={id}", ticker.get_untracked());
        navigate.with_value(|navigate| navigate(&target, NavigateOptions::default()));
    };

    view! {
        <div class="console security-hub">
            <HfPageHeader title="Security Hub" subtitle="Company intelligence" />

            <HfPanel class="security-hub__des">
                <div class="security-hub__identity">
                    <span class="mono">{move || ticker.get()}</span>
                    <span class="security-hub__name">
                        {move || symbol.get().map(|s| s.name).unwrap_or("Unknown instrument")}
                    </span>
                    <span class="security-hub__exchange">
                        {move || {
                            symbol
                                .get()
                                .map(|s| format!("{} · {}", s.exchange, s.asset_class))
                                .unwrap_or_default()
                        }}
                    </span>
                </div>

                <div class="security-hub__quote">
                    <span class="tnum security-hub__ltp">
                        {move || symbol.get().map(|s| s.ltp.to_string()).unwrap_or("—".into())}
                    </span>
                    {move || {
                        symbol
                            .get()
                            .map(|s| {
                                view! {
                                    <HfPLNumber
                                        value=parse_f64(s.change_pct)
                                        format=HfPLNumberFormat::Percent
                                    />
                                }
                                .into_any()
                            })
                            .unwrap_or_else(|| view! { <span>"—"</span> }.into_any())
                    }}
                </div>

                <div class="security-hub__ratios">
                    <HfKpi label="Mkt cap">"—"</HfKpi>
                    <HfKpi label="P/E">"—"</HfKpi>
                    {move || {
                        symbol
                            .get()
                            .map(|s| {
                                let (lo, hi) = session_range(&s);
                                view! {
                                    <HfMiniRangeBar
                                        low_label=format!("{lo:.2}")
                                        high_label=format!("{hi:.2}")
                                        low=lo
                                        high=hi
                                        last=parse_f64(s.ltp)
                                    />
                                }
                                .into_any()
                            })
                            .unwrap_or_else(|| view! { <span>"No range"</span> }.into_any())
                    }}
                </div>
            </HfPanel>

            <HfTabs default_value=DEFAULT_TAB>
                <HfTabsList class="security-hub__tabs">
                    {TABS
                        .iter()
                        .map(|&(id, label)| {
                            view! {
                                <HfTabsTrigger value=id on_click=Callback::new(move |_| go_tab(id))>
                                    {label}
                                </HfTabsTrigger>
                            }
                        })
                        .collect_view()}
                </HfTabsList>
            </HfTabs>

            {move || {
                let tab = active_tab.get();
                if tab == "overview" {
                    view! {
                        <HfPanel>
                            <HfPanelHeader title="Overview" />
                            <div class="hfui-panel__content">
                                <div class="summary-row">
                                    <span>"Ticker"</span>
                                    <strong class="mono">{move || ticker.get()}</strong>
                                </div>
                                <div class="summary-row">
                                    <span>"Last traded price"</span>
                                    <strong class="tnum">
                                        {move || symbol.get().map(|s| s.ltp.to_string()).unwrap_or("—".into())}
                                    </strong>
                                </div>
                                <div class="summary-row">
                                    <span>"Market cap / P/E"</span>
                                    <strong>"Coming soon"</strong>
                                </div>
                            </div>
                        </HfPanel>
                    }
                    .into_any()
                } else {
                    view! {
                        <HfPanel>
                            <HfPanelHeader title=tab_label(tab) />
                            <div class="hfui-panel__content">
                                <p class="placeholder__note">"Coming soon"</p>
                            </div>
                        </HfPanel>
                    }
                    .into_any()
                }
            }}
        </div>
    }
}
