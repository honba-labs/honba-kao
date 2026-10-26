//! Security Hub — single-page company intelligence at `/equity/security/:ticker`.
//!
//! * [`security_hub_page`] — the page shell: DES header, 11-tab router driven by
//!   the `?tab=` query parameter, and the per-tab bodies.

pub mod filings_tab;
pub mod financials_tab;
pub mod security_hub_page;
pub mod value_chain_tab;

pub use filings_tab::FilingsTab;
pub use financials_tab::FinancialsTab;
pub use value_chain_tab::ValueChainTab;

pub use security_hub_page::SecurityHubPage;
