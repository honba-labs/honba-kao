use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MarginUtilization {
    pub total_collateral: Decimal,
    pub margin_used: Decimal,
    pub margin_available: Decimal,
    pub utilization_ratio: Decimal,
    pub is_warning: bool,
    pub is_critical: bool,
}

pub fn calculate_margin_utilization(total_collateral: Decimal, margin_used: Decimal) -> MarginUtilization {
    let margin_available = (total_collateral - margin_used).max(Decimal::ZERO);
    let ratio = if total_collateral > Decimal::ZERO {
        (margin_used / total_collateral) * dec!(100.0)
    } else if margin_used > Decimal::ZERO {
        dec!(100.0)
    } else {
        Decimal::ZERO
    };

    let warning = dec!(70.0);
    let critical = dec!(85.0);

    MarginUtilization {
        total_collateral,
        margin_used,
        margin_available,
        utilization_ratio: ratio,
        is_warning: ratio >= warning && ratio < critical,
        is_critical: ratio >= critical,
    }
}
