//! Security Hub — single-page company intelligence at `/equity/security/:ticker`.
//!
//! * [`security_hub_page`] — the page shell: DES header, 11-tab router driven by
//!   the `?tab=` query parameter, and the per-tab bodies.

pub mod security_hub_page;

pub use security_hub_page::SecurityHubPage;
