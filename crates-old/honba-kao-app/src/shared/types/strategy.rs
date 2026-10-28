//! Strategy types. Mirrors `web/src/types/strategy.ts`.

/// A strategy listed in the strategy market / studio.
#[derive(Clone, Debug, PartialEq)]
pub struct StrategySummary {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub win_rate: f64,
    pub author: &'static str,
}

/// A planning leg used by the trading planner.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanLeg {
    pub label: &'static str,
    pub trigger: &'static str,
    pub size_pct: f64,
    pub enabled: bool,
}
