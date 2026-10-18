//! Domain + presentational components. Mirrors `web/src/components/`.
//!
//! * [`ui`] — framework-level primitives (buttons, panels, modals, …).
//! * [`common`] — app chrome shared across screens (header, footer, overlays).
//! * one module per domain: [`auth`], [`terminal`], [`trader`], [`charts`],
//!   [`strategy`], [`planning`], [`modals`], [`faq`], [`landing`].

pub mod auth;
pub mod charts;
pub mod common;
pub mod faq;
pub mod landing;
pub mod modals;
pub mod planning;
pub mod strategy;
pub mod terminal;
pub mod trader;
pub mod ui;
