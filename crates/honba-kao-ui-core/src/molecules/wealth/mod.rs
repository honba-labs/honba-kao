//! Tier 2: MOLECULES — Mutual Funds & Wealth specific.
//!
//! NSH port (Phase 2, see `nsh-port.md` §4): still Leptos, not yet ported.
//! Uncomment each line as its file is converted.

pub mod asset_allocation_bar;
// pub mod overlap_pill;
pub mod risk_o_meter;
pub mod rolling_returns;
pub mod sip_calculator;

pub use asset_allocation_bar::{Allocation, HfAssetAllocationBar};
// pub use overlap_pill::HfPortfolioOverlapPill;
pub use risk_o_meter::{HfRiskOMeter, RiskLevel};
pub use rolling_returns::HfRollingReturnsVisualizer;
pub use sip_calculator::HfSipCalculatorCard;
