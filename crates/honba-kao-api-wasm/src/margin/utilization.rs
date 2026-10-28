use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MarginUtilization {
    pub total_collateral: f64,
    pub margin_used: f64,
    pub margin_available: f64,
    pub utilization_ratio: f64,
    pub is_warning: bool,
    pub is_critical: bool,
}

pub fn calculate_margin_utilization(total_collateral: f64, margin_used: f64) -> MarginUtilization {
    let margin_available = (total_collateral - margin_used).max(0.0);
    let ratio = if total_collateral > 0.0 {
        (margin_used / total_collateral) * 100.0
    } else if margin_used > 0.0 {
        100.0
    } else {
        0.0
    };

    MarginUtilization {
        total_collateral,
        margin_used,
        margin_available,
        utilization_ratio: ratio,
        is_warning: ratio >= 70.0 && ratio < 85.0,
        is_critical: ratio >= 85.0,
    }
}
