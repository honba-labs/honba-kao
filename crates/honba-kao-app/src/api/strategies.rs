//! Strategies API namespace. Mirrors `web/src/lib/api/strategies.ts`.

use crate::data::demo_data;
use crate::types::StrategySummary;

pub fn list_strategies() -> Vec<StrategySummary> {
    demo_data::demo_strategies()
}
