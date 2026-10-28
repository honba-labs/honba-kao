//! Screener primitives — composite factor score badge, factor driver chips,
//! sparkline size presets, and 52-week range bar with trend-state label.
//!
//! Milestone 1 ("UI Primitives & Styling") of `plan-screener.md`.

pub mod factor_chips;
pub mod range_bar_cell;
pub mod score_badge;
// Named `sparkline_presets` (not `sparkline_cell`) to avoid an
// ambiguous-glob-reexport collision with `crate::home::sparkline_cell`,
// which is also re-exported at the crate root.
pub mod sparkline_presets;

pub use factor_chips::*;
pub use range_bar_cell::*;
pub use score_badge::*;
pub use sparkline_presets::*;
