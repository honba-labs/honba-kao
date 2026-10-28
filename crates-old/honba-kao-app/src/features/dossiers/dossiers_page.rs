//! Research Dossiers page (Phase 2.5) — a single dossier with its thesis,
//! tagged evidence claims, revision history and a shareable-snapshot toggle.
//!
//! All content on this page is hardcoded mock data, including the evidence
//! quotes below. Unlike the Filings tab (`features::security_hub::filings_tab`,
//! which now runs real citation lookups via `shared::filings::search`), this
//! page's thesis is about Titan Company, a jewellery retailer with no
//! corresponding document in the small checked-in sample corpus (which only
//! covers Reliance, Infosys and TCS) — so wiring it to that module would
//! either require fabricating a fourth company's filing text or produce
//! nonsense matches. Left as pure mock for now; see honba-nou#1 /
//! honba-data#1 / honba-tamashii#1 on GitHub for the tracked work on a real
//! ingestion + retrieval backend that would eventually cover this page too.
//! There is also no persistence or backend, and the snapshot link is a
//! placeholder string that is never generated server side.

use honba_kao_ui_core::{
    HfButton, HfKpi, HfPageHeader, HfPanel, HfPanelHeader, HfTable, HfTableBody, HfTableCell,
    HfTableHeader, HfTableHeaderCell, HfTableRow,
};
use leptos::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ClaimTag {
    Supporting,
    Opposing,
    NeedsReview,
}

impl ClaimTag {
    /// Label shown on the tag.
    fn label(self) -> &'static str {
        match self {
            ClaimTag::Supporting => "Supporting",
            ClaimTag::Opposing => "Opposing",
            ClaimTag::NeedsReview => "Needs Review",
        }
    }

    /// Colour token, reusing the `.hfui-badge` state colours
    /// (gain / loss / warning).
    fn badge_class(self) -> &'static str {
        match self {
            ClaimTag::Supporting => "hfui-badge running",
            ClaimTag::Opposing => "hfui-badge stopped",
            ClaimTag::NeedsReview => "hfui-badge paused",
        }
    }
}

struct Claim {
    tag: ClaimTag,
    claim: &'static str,
    quote: &'static str,
    date: &'static str,
    source: &'static str,
}

const CLAIMS: &[Claim] = &[
    Claim {
        tag: ClaimTag::Supporting,
        claim: "Studded mix shift is lifting realisation per gram.",
        quote: "Studded share rose to 41% of jewellery sales this quarter, the highest in the company's reported history.",
        date: "2026-08-07",
        source: "Q2 FY27 Earnings Call, p.8",
    },
    Claim {
        tag: ClaimTag::Supporting,
        claim: "Wedding-season demand is holding despite high gold prices.",
        quote: "Bridal demand remained resilient through the festive window even as gold traded at record highs.",
        date: "2026-08-09",
        source: "Q2 FY27 Investor Presentation, p.5",
    },
    Claim {
        tag: ClaimTag::Opposing,
        claim: "Gold-loan competition may cap same-store growth.",
        quote: "Several competitors have cut lending rates on secured gold products, and we expect pricing pressure to persist through FY27.",
        date: "2026-09-18",
        source: "Analyst call transcript, Sep 2026",
    },
    Claim {
        tag: ClaimTag::NeedsReview,
        claim: "Payback assumptions for new small-format stores are untested.",
        quote: "Payback on the new small-format stores is expected within 30 months, subject to the pilot cohort's maturity.",
        date: "2026-08-09",
        source: "Q2 FY27 Investor Presentation, p.17",
    },
    Claim {
        tag: ClaimTag::NeedsReview,
        claim: "Lab-grown diamond pricing could compress the premium segment.",
        quote: "Lab-grown price points fell again in the quarter; we are monitoring cannibalisation in entry-level bridal.",
        date: "2026-10-03",
        source: "Industry note, Oct 2026",
    },
];

struct Revision {
    version: &'static str,
    date: &'static str,
    summary: &'static str,
}

const REVISIONS: &[Revision] = &[
    Revision {
        version: "v1.0",
        date: "2026-07-02",
        summary: "Initial thesis drawn from the FY26 annual report.",
    },
    Revision {
        version: "v1.1",
        date: "2026-08-11",
        summary: "Added Q2 FY27 call evidence and raised the studded-mix assumption.",
    },
    Revision {
        version: "v1.2",
        date: "2026-10-14",
        summary: "Added opposing gold-loan evidence and flagged store-capex claim for review.",
    },
];

const ASSUMPTIONS: &[&str] = &[
    "Studded share of jewellery sales keeps rising above 40%.",
    "Gold-loan competition does not pull same-store growth below 12%.",
    "New small-format stores reach breakeven within 30 months.",
];

#[component]
fn ClaimCard(claim: &'static Claim) -> impl IntoView {
    view! {
        <HfPanel class="dossiers__claim">
            <div class="dossiers__claim-head">
                <span class=claim.tag.badge_class()>{claim.tag.label()}</span>
                <span class="dossiers__claim-text">{claim.claim}</span>
            </div>
            <blockquote class="dossiers__quote">{claim.quote}</blockquote>
            <div class="dossiers__claim-meta">
                <span class="tnum">{claim.date}</span>
                <span>{claim.source}</span>
            </div>
        </HfPanel>
    }
}

#[component]
pub fn DossiersPage() -> impl IntoView {
    let snapshot_open = RwSignal::new(false);

    view! {
        <div class="console dossiers">
            <HfPageHeader title="Research Dossiers" subtitle="Thesis, evidence and revision history" />

            <p class="placeholder__note">"Mock data · no dossier backend connected"</p>

            <HfPanel class="dossiers__identity">
                <div class="dossiers__identity-name">
                    <span class="mono">"NSE:TITAN-EQ"</span>
                    <span>"Titan Company · Dossier v1.2"</span>
                </div>
                <div class="dossiers__kpis">
                    <HfKpi label="Status">"Draft"</HfKpi>
                    <HfKpi label="Evidence">"5 claims"</HfKpi>
                    <HfKpi label="Conviction">"Medium"</HfKpi>
                </div>
            </HfPanel>

            <HfPanel>
                <HfPanelHeader title="Thesis" />
                <div class="hfui-panel__content dossiers__thesis">
                    <div class="summary-row">
                        <span>"Core thesis"</span>
                        <strong>
                            "Mix-led margin expansion in studded jewellery should compound EBIT margins for three years despite gold price volatility."
                        </strong>
                    </div>
                    <div class="dossiers__assumptions">
                        <span class="dossiers__label">"Assumptions"</span>
                        <ul class="dossiers__list">
                            {ASSUMPTIONS
                                .iter()
                                .map(|a| view! { <li>{*a}</li> })
                                .collect_view()}
                        </ul>
                    </div>
                    <div class="summary-row">
                        <span>"Catalyst"</span>
                        <strong>"Q3 FY27 results (January 2027) showing studded mix above 42%."</strong>
                    </div>
                    <div class="summary-row">
                        <span>"Invalidation"</span>
                        <strong>
                            "Two consecutive quarters of EBIT margin contraction driven by a studded-mix reversal."
                        </strong>
                    </div>
                </div>
            </HfPanel>

            <section class="dossiers__section">
                <h3 class="dossiers__heading">"Evidence"</h3>
                {CLAIMS
                    .iter()
                    .map(|claim| view! { <ClaimCard claim=claim /> })
                    .collect_view()}
            </section>

            <HfPanel>
                <HfPanelHeader title="Shareable Snapshot" />
                <div class="dossiers__snapshot">
                    <HfButton on_click=Callback::new(move |_| snapshot_open.update(|open| *open = !*open))>
                        "Generate Shareable Snapshot"
                    </HfButton>
                    {move || {
                        snapshot_open
                            .get()
                            .then(|| {
                                view! {
                                    <span class="mono dossiers__snapshot-link">
                                        "Snapshot link: https://mock.invalid/dossiers/snap-7f3a (revocable)"
                                    </span>
                                }
                            })
                    }}
                </div>
            </HfPanel>

            <HfPanel>
                <HfPanelHeader title="Revision History" />
                <HfTable class="dossiers__revisions">
                    <HfTableHeader>
                        <HfTableRow>
                            <HfTableHeaderCell>"Version"</HfTableHeaderCell>
                            <HfTableHeaderCell>"Date"</HfTableHeaderCell>
                            <HfTableHeaderCell>"Summary of change"</HfTableHeaderCell>
                        </HfTableRow>
                    </HfTableHeader>
                    <HfTableBody>
                        {REVISIONS
                            .iter()
                            .map(|rev| {
                                view! {
                                    <HfTableRow>
                                        <HfTableCell class="mono">{rev.version}</HfTableCell>
                                        <HfTableCell class="tnum">{rev.date}</HfTableCell>
                                        <HfTableCell>{rev.summary}</HfTableCell>
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
