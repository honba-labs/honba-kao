//! Multi-leg payoff engine for derivatives strategies.
//!
//! Pure Rust computation of strategy P&L across a spot range at expiry,
//! breakeven points, and maximum profit/loss. Zero UI or network dependencies.

pub mod leg;
pub mod payoff;

pub use leg::{Leg, OptionType, Side};
pub use payoff::{PayoffConfig, PayoffEngine, PayoffPoint, PayoffResult};
