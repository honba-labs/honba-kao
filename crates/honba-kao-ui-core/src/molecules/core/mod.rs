//! Tier 2: MOLECULES — Universal trading building blocks.
//!
//! NSH port (Phase 2, see `nsh-port.md` §4): still Leptos, not yet ported.
//! Uncomment each line as its file is converted.

// pub mod alert_stack;
pub mod estimation_fields;
pub mod price_snap_input;
pub mod quantity_chip_input;
// pub mod spread_indicator;
// pub mod stats_bar;
// pub mod tape_row;

// pub use alert_stack::{AlertSeverity, HfAlertBanner, HfAlertBannerStack};
pub use estimation_fields::HfEstimationFields;
pub use price_snap_input::HfPriceSnapInput;
pub use quantity_chip_input::HfQuantityChipInput;
// pub use spread_indicator::HfSpreadIndicator;
// pub use stats_bar::{HfStatsBar, StatItem};
// pub use tape_row::HfTapeRow;
