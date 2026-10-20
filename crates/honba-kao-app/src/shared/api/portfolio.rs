//! Aggregated portfolio API namespace.
//!
//! One call returns every broker, holding and fund balance, plus the derived
//! totals. [`snapshot_for`] recomputes the aggregate for a client-side broker
//! list, which is what the connect/disconnect actions on the portfolio page
//! use — the same call the server would make after a token changes.

use crate::shared::data::demo_data;
use crate::shared::types::{BrokerAccount, BrokerFunds, Holding, PortfolioSnapshot};

/// Portfolio for the default connection state.
pub fn get_snapshot() -> PortfolioSnapshot {
    demo_data::demo_portfolio()
}

/// Portfolio restricted to the connected brokers in `brokers`.
pub fn snapshot_for(brokers: &[BrokerAccount]) -> PortfolioSnapshot {
    demo_data::build_portfolio(brokers)
}

pub fn get_holdings() -> Vec<Holding> {
    demo_data::demo_portfolio().holdings
}

pub fn get_funds() -> Vec<BrokerFunds> {
    demo_data::demo_portfolio().funds
}
