//! Domain + presentational components.
//!
//! * [`common`] — app chrome shared across screens (header, footer, rail,
//!   overlays).
//! * [`charts`], [`strategy`] — charting and strategy-studio domain pieces.
//! * [`auth`], [`landing`], [`modals`] — auth shells, marketing hero, dialogs.

pub mod auth;
pub mod charts;
pub mod common;
pub mod landing;
pub mod modals;
pub mod screener;
pub mod simulator;
pub mod strategy;
pub mod workbench;
