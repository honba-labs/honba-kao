//! Strategies API namespace. Mirrors `web/src/lib/api/strategies.ts`.

use crate::shared::data::demo_data;
use crate::shared::types::StrategySummary;

pub fn list_strategies() -> Vec<StrategySummary> {
    demo_data::demo_strategies()
}
