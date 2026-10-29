//! Tier 2: MOLECULES — Derivatives & F&O specific.
//!
//! NSH port (Phase 2, see `nsh-port.md` §4): siblings below are still
//! Leptos, not yet ported — uncomment each line as its file is converted.

pub mod payoff_chart;
// pub mod greeks_group;
// pub mod max_pain_badge;
// pub mod option_chain_row;
// pub mod pcr_meter;
// pub mod strategy_leg_row;

pub use payoff_chart::{HfOptionPayoffChart, PayoffPoint};
// pub use greeks_group::{Greeks, HfGreeksGroup};
// pub use max_pain_badge::HfMaxPainBadge;
// pub use option_chain_row::{HfOptionChainRow, OptionLegQuote};
// pub use pcr_meter::HfPcrMeter;
// pub use strategy_leg_row::{HfStrategyLegRow, StrategyLeg};
