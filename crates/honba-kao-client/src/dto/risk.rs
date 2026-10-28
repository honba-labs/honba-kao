use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MarginSummary {
    pub total_collateral: Decimal,
    pub margin_utilized: Decimal,
    pub margin_available: Decimal,
    pub utilization_pct: Decimal,
    pub haircut_discount: Decimal,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LiquidationWarning {
    pub symbol: String,
    pub position_size: Decimal,
    pub liquidation_price: Decimal,
    pub mark_price: Decimal,
    pub buffer_pct: Decimal,
    pub is_critical: bool,
}
