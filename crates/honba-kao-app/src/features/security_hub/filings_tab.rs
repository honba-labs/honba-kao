//! Filings Intelligence tab (Phase 2.4) — scored growth/headwind drivers with
//! verbatim quotes, an "Ask the Filings" QA panel and a guidance tracker.
//!
//! All content is hardcoded mock data. There is no RAG or filings backend yet;
//! the QA panel shows a canned answer for any question. Scores are signed
//! integers carried as display strings so the mock holds no floating-point values.

use honba_kao_ui_core::{
    HfButton, HfInput, HfPanel, HfPanelHeader, HfTable, HfTableBody, HfTableCell, HfTableHeader,
    HfTableHeaderCell, HfTableRow,
};
use leptos::prelude::*;

struct Driver {
    score: &'static str,
    gain: bool,
    title: &'static str,
    quote: &'static str,
    source: &'static str,
}

const GROWTH_ENGINES: &[Driver] = &[
    Driver {
        score: "+85",
        gain: true,
        title: "Premium segment volume compounding double digits",
        quote: "Premium product volumes grew 17% year on year, with mix improving for a fifth consecutive quarter.",
        source: "Q1 FY26 Report, p.14",
    },
    Driver {
        score: "+72",
        gain: true,
        title: "Digital channel now a meaningful revenue contributor",
        quote: "Digital channels contributed 22% of gross revenue, up from 15% in the prior year.",
        source: "Q1 FY26 Earnings Call, p.6",
    },
    Driver {
        score: "+61",
        gain: true,
        title: "Capacity expansion commissioned ahead of schedule",
        quote: "The second line was commissioned two months ahead of plan and is already running at 80% utilisation.",
        source: "FY25 Annual Report, p.31",
    },
    Driver {
        score: "+48",
        gain: true,
        title: "Export order book widening into EU markets",
        quote: "Order inflows from European customers rose sharply, with three new multi-year supply agreements signed.",
        source: "Q4 FY25 Investor Presentation, p.9",
    },
];

const HEADWINDS: &[Driver] = &[
    Driver {
        score: "-45",
        gain: false,
        title: "Raw material cost pressure on gross margin",
        quote: "Gross margin contracted 140 bps as crude-linked feedstock costs were only partially passed through to customers.",
        source: "Q1 FY26 Report, p.16",
    },
    Driver {
        score: "-38",
        gain: false,
        title: "Rising finance cost from debt-funded capex",
        quote: "Finance costs increased 24% on higher average borrowings to fund the new capacity build-out.",
        source: "Q1 FY26 Report, p.19",
    },
    Driver {
        score: "-29",
        gain: false,
        title: "Contingent liability under tax dispute",
        quote: "The company has disclosed a contingent liability of INR 412 crore pending appeal before the appellate tribunal.",
        source: "FY25 Annual Report, p.158",
    },
];

struct GuidanceRow {
    quarter: &'static str,
    guidance: &'static str,
    actual: &'static str,
}

const GUIDANCE: &[GuidanceRow] = &[
    GuidanceRow {
        quarter: "Q4 FY25",
        guidance: "Revenue growth 10-12%",
        actual: "11.4%",
    },
    GuidanceRow {
        quarter: "Q1 FY26",
        guidance: "EBITDA margin 18-19%",
        actual: "18.6%",
    },
    GuidanceRow {
        quarter: "Q2 FY26",
        guidance: "Capex INR 1,200 cr",
        actual: "INR 1,050 cr",
    },
];

const MOCK_ANSWER: &str = "Management attributes the premium volume growth to mix shift toward higher-margin SKUs, \
     while raw material pass-through remains the main margin risk for the next two quarters.";
const MOCK_CITATION: &str = "Q1 FY26 Report, p.14";

#[component]
fn DriverCard(driver: &'static Driver) -> impl IntoView {
    let score_class = if driver.gain {
        "filings-tab__score filings-tab__score--gain tnum"
    } else {
        "filings-tab__score filings-tab__score--loss tnum"
    };
    view! {
        <HfPanel class="filings-tab__card">
            <div class="filings-tab__card-head">
                <span class=score_class>{driver.score}</span>
                <span class="filings-tab__card-title">{driver.title}</span>
            </div>
            <blockquote class="filings-tab__quote">{driver.quote}</blockquote>
            <span class="filings-tab__source">{driver.source}</span>
        </HfPanel>
    }
}

#[component]
pub fn FilingsTab() -> impl IntoView {
    let question = RwSignal::new(String::new());
    let asked = RwSignal::new(None::<String>);

    view! {
        <div class="filings-tab">
            <p class="placeholder__note">"Mock data · no filings backend connected"</p>

            <section class="filings-tab__section">
                <h3 class="filings-tab__heading">"Growth Engines"</h3>
                <div class="filings-tab__cards">
                    {GROWTH_ENGINES
                        .iter()
                        .map(|driver| view! { <DriverCard driver=driver /> })
                        .collect_view()}
                </div>
            </section>

            <section class="filings-tab__section">
                <h3 class="filings-tab__heading">"Headwinds"</h3>
                <div class="filings-tab__cards">
                    {HEADWINDS
                        .iter()
                        .map(|driver| view! { <DriverCard driver=driver /> })
                        .collect_view()}
                </div>
            </section>

            <HfPanel>
                <HfPanelHeader title="Ask the Filings" />
                <div class="filings-tab__ask">
                    <HfInput
                        class="filings-tab__ask-input"
                        r#type="text"
                        placeholder="Ask a question about the filings…"
                        on_input=Callback::new(move |ev: web_sys::Event| {
                            let input = event_target::<web_sys::HtmlInputElement>(&ev);
                            question.set(input.value());
                        })
                    />
                    <HfButton on_click=Callback::new(move |_| asked.set(Some(question.get_untracked())))>
                        "Ask"
                    </HfButton>
                </div>
                {move || {
                    asked
                        .get()
                        .map(|q| {
                            let q = if q.trim().is_empty() {
                                "(no question entered)".to_string()
                            } else {
                                q
                            };
                            view! {
                                <div class="filings-tab__answer">
                                    <span class="filings-tab__answer-q">{q}</span>
                                    <p class="filings-tab__answer-text">{MOCK_ANSWER}</p>
                                    <span class="filings-tab__source">{MOCK_CITATION}</span>
                                </div>
                            }
                        })
                }}
            </HfPanel>

            <HfPanel>
                <HfPanelHeader title="Guidance Tracker" />
                <HfTable class="filings-tab__guidance">
                    <HfTableHeader>
                        <HfTableRow>
                            <HfTableHeaderCell>"Quarter"</HfTableHeaderCell>
                            <HfTableHeaderCell>"Management Guidance"</HfTableHeaderCell>
                            <HfTableHeaderCell class="tnum">"Actual Delivered"</HfTableHeaderCell>
                        </HfTableRow>
                    </HfTableHeader>
                    <HfTableBody>
                        {GUIDANCE
                            .iter()
                            .map(|row| {
                                view! {
                                    <HfTableRow>
                                        <HfTableCell>{row.quarter}</HfTableCell>
                                        <HfTableCell>{row.guidance}</HfTableCell>
                                        <HfTableCell class="tnum">{row.actual}</HfTableCell>
                                    </HfTableRow>
                                }
                            })
                            .collect_view()}
                    </HfTableBody>
                </HfTable>
            </HfPanel>
        </div>
    }
}
