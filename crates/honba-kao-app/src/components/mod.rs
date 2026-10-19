//! Domain + presentational components.
//!
//! * [`ui`] — framework-level primitives (buttons, panels, modals, …).
//! * [`common`] — app chrome shared across screens (header, footer, rail,
//!   overlays).
//! * [`charts`], [`strategy`] — charting and strategy-studio domain pieces.
//! * [`auth`], [`landing`], [`modals`] — auth shells, marketing hero, dialogs.

pub mod auth;
pub mod charts;
pub mod common;
pub mod landing;
pub mod modals;
pub mod strategy;
pub mod ui;
