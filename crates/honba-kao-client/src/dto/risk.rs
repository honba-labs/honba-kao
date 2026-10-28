use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MarginSummary {
    pub total_collateral: f64,
    pub margin_utilized: f64,
    pub margin_available: f64,
    pub utilization_pct: f64,
    pub haircut_discount: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LiquidationWarning {
    pub symbol: String,
    pub position_size: f64,
    pub liquidation_price: f64,
    pub mark_price: f64,
    pub buffer_pct: f64,
    pub is_critical: bool,
}
