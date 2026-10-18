//! Traders API namespace. Mirrors `web/src/lib/api/traders.ts`.

use crate::data::demo_data;
use crate::types::{AccountInfo, DecisionRecord, Position, SystemStatus, TraderInfo};

/// All configured traders (dashboard pass requests only active ones).
pub fn get_traders() -> Vec<TraderInfo> {
    demo_data::demo_traders()
}

pub fn get_account(_trader_id: &str) -> AccountInfo {
    demo_data::demo_account()
}

pub fn get_positions(_trader_id: &str) -> Vec<Position> {
    demo_data::demo_positions()
}

pub fn get_status(trader_id: &str) -> SystemStatus {
    demo_data::demo_status(trader_id)
}

pub fn get_latest_decisions(_trader_id: &str, limit: usize) -> Vec<DecisionRecord> {
    let mut decisions = demo_data::demo_decisions();
    decisions.truncate(limit);
    decisions
}
