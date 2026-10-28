//! Filings Intelligence tab (Phase 2.4) — scored growth/headwind drivers with
//! verbatim quotes, an "Ask the Filings" QA panel and a guidance tracker.
//!
//! Driver *labels and scores* below are a small hand-authored sample list —
//! a real RAG-driven driver-scoring model is out of scope for this crate.
//! The *citation* attached to each driver, and the answer shown by "Ask the
//! Filings", are both real results of running `shared::filings::search`
//! against a small checked-in sample document corpus (see
//! `crate::shared::filings`): genuine exact/fuzzy keyword search returning a
//! verbatim quote, document name, and page number — not a hardcoded
//! citation string or canned answer map. This is a lightweight local demo
//! mechanism, not a production RAG pipeline; the guidance tracker numbers
//! below remain illustrative sample data. See honba-nou#1 / honba-data#1 /
//! honba-tamashii#1 on GitHub for the tracked work to replace this with a
//! real ingestion + retrieval backend.

use crate::shared::filings::{self, Citation};
use honba_kao_ui_core::{
    HfButton, HfInput, HfPanel, HfPanelHeader, HfTable, HfTableBody, HfTableCell, HfTableHeader,
    HfTableHeaderCell, HfTableRow,
};
use leptos::prelude::*;

struct Driver {
    score: &'static str,
    gain: bool,
    title: &'static str,
    /// Keywords passed to `filings::search` to look up this driver's
    /// verbatim citation at render time.
    search_query: &'static str,
}

const GROWTH_ENGINES: &[Driver] = &[
    Driver {
        score: "+85",
        gain: true,
        title: "Jio ARPU expansion on premium plan mix",
        search_query: "Jio ARPU premium postpaid True-5G",
    },
    Driver {
        score: "+72",
        gain: true,
        title: "Infosys digital revenue now the majority of the book",
        search_query: "digital revenue large deal total contract value",
    },
    Driver {
        score: "+61",
        gain: true,
        title: "TCS order book led by cloud and cybersecurity demand",
        search_query: "order book cloud migration cybersecurity bookings",
    },
];

const HEADWINDS: &[Driver] = &[
    Driver {
        score: "-45",
        gain: false,
        title: "O2C margin pressure from crude-linked feedstock costs",
        search_query: "feedstock costs margin finance costs borrowings",
    },
    Driver {
        score: "-38",
        gain: false,
        title: "BFSI discretionary spend still muted for Infosys",
        search_query: "discretionary spending BFSI cost takeout attrition",
    },
    Driver {
        score: "-29",
        gain: false,
        title: "TCS contingent liability under service tax dispute",
        search_query: "contingent liability service tax appellate tribunal",
    },
];

struct GuidanceRow {
    quarter: &'static str,
    guidance: &'static str,
    actual: &'static str,
}

/// Illustrative sample guidance-vs-actual numbers; not sourced from the
/// filings corpus above (guidance tracking is out of scope for the search
/// module — it is not a verbatim-quote claim).
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

/// Renders a resolved citation as a verbatim blockquote + source line, or an
/// honest "no match" state when `filings::search` found nothing.
#[component]
fn CitationBlock(citation: Option<Citation>) -> impl IntoView {
    match citation {
        Some(c) => view! {
            <>
                <blockquote class="filings-tab__quote">{c.quote.clone()}</blockquote>
                <span class="filings-tab__source">{format!("{}, p.{}", c.document, c.page_number)}</span>
            </>
        }
        .into_any(),
        None => view! {
            <span class="filings-tab__source filings-tab__source--missing">
                "No verbatim match found in the sample filings corpus."
            </span>
        }
        .into_any(),
    }
}

#[component]
fn DriverCard(driver: &'static Driver) -> impl IntoView {
    let score_class = if driver.gain {
        "filings-tab__score filings-tab__score--gain tnum"
    } else {
        "filings-tab__score filings-tab__score--loss tnum"
    };
    // Real lookup: the quote and page number rendered below come straight
    // out of `filings::search`, not a literal string in this component.
    let citation = filings::search(driver.search_query);
    view! {
        <HfPanel class="filings-tab__card">
            <div class="filings-tab__card-head">
                <span class=score_class>{driver.score}</span>
                <span class="filings-tab__card-title">{driver.title}</span>
            </div>
            <CitationBlock citation=citation />
        </HfPanel>
    }
}

#[component]
pub fn FilingsTab() -> impl IntoView {
    let question = RwSignal::new(String::new());
    let asked = RwSignal::new(None::<String>);

    view! {
        <div class="filings-tab">
            <p class="placeholder__note">
                "Driver scores are a hand-authored sample list · citations and QA answers are real lookups against a small checked-in sample document corpus — not a production RAG pipeline"
            </p>

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
                            let display_q = if q.trim().is_empty() {
                                "(no question entered)".to_string()
                            } else {
                                q.clone()
                            };
                            // Real query: runs the typed question straight through
                            // `filings::search`, so the answer below is whatever that
                            // function actually finds (or a genuine "no match").
                            let result = filings::search(&q);
                            view! {
                                <div class="filings-tab__answer">
                                    <span class="filings-tab__answer-q">{display_q}</span>
                                    <CitationBlock citation=result />
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
