//! Checked-in sample filing excerpts used by the [`super::search`] lookup.
//!
//! These are **representative sample excerpts written for this local demo**,
//! composed in the style of NSE/BSE quarterly results commentary and annual
//! report MD&A sections for three well-known Indian listed companies
//! (Reliance Industries, Infosys, TCS). They are **not** scraped, copied, or
//! reconstructed from any actual filing, earnings call, or disclosure
//! document — they are faithful-in-style fiction written so the citation
//! search module has real, checked-in text to run deterministic lookups
//! against. Do not present this corpus as verified or licensed disclosure
//! text; see `shared/filings/mod.rs` for the scope of what the search module
//! does and does not claim.

use super::FilingPage;

pub const RELIANCE_Q1_FY26: &[FilingPage] = &[
    FilingPage {
        document: "Reliance Industries - Q1 FY26 Results Commentary",
        page_number: 4,
        text: "Jio's digital services ARPU rose to INR 203 this quarter, up from INR 181 a year ago, \
            as the subscriber base continued to migrate onto higher-tariff postpaid and True-5G data \
            plans. Premium subscriber additions outpaced overall net adds for the fourth consecutive \
            quarter, with the mix shift expected to continue supporting ARPU expansion through the \
            rest of the fiscal year.",
    },
    FilingPage {
        document: "Reliance Industries - Q1 FY26 Results Commentary",
        page_number: 11,
        text: "Oil-to-chemicals segment EBITDA margin contracted as crude-linked feedstock costs rose \
            faster than product cracks, with only part of the increase passed through to customers \
            this quarter. Finance costs increased on higher average borrowings drawn to fund the new \
            Jamnagar capacity expansion, which remains on schedule for commissioning next year.",
    },
];

pub const INFOSYS_Q1_FY26: &[FilingPage] = &[
    FilingPage {
        document: "Infosys - Q1 FY26 Earnings Release",
        page_number: 2,
        text: "Digital revenue contributed 62.3% of total revenue this quarter, with large deal total \
            contract value of USD 2.7 billion signed across BFSI, retail and manufacturing clients. \
            Generative AI-linked engagements more than doubled sequentially, and management expects \
            this momentum to continue into the second half.",
    },
    FilingPage {
        document: "Infosys - Q1 FY26 Earnings Release",
        page_number: 7,
        text: "Discretionary technology spending in the BFSI segment remained muted, and clients \
            continued to prioritise cost takeout programmes over fresh transformation budgets this \
            quarter. Voluntary attrition ticked up slightly to 13.1% on a trailing twelve month basis, \
            which management attributed to a tightening lateral hiring market.",
    },
];

pub const TCS_FY25_ANNUAL: &[FilingPage] = &[
    FilingPage {
        document: "TCS - FY25 Annual Report MD&A",
        page_number: 46,
        text: "The order book remained healthy, led by continued demand for cloud migration and \
            cybersecurity modernisation programmes across North American and European clients. Net \
            new bookings for the year crossed USD 34 billion, with BFSI and manufacturing verticals \
            contributing the largest share of incremental signings.",
    },
    FilingPage {
        document: "TCS - FY25 Annual Report MD&A",
        page_number: 58,
        text: "Operating margin was impacted by the annual wage revision effective in the first \
            quarter, partially offset by currency tailwinds and continued pyramid optimisation. The \
            company has disclosed a contingent liability of INR 412 crore relating to a service tax \
            matter pending appeal before the appellate tribunal.",
    },
];

/// All checked-in pages across the sample corpus, used by [`super::search`].
pub fn all_pages() -> Vec<FilingPage> {
    RELIANCE_Q1_FY26
        .iter()
        .chain(INFOSYS_Q1_FY26.iter())
        .chain(TCS_FY25_ANNUAL.iter())
        .copied()
        .collect()
}
