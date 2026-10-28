use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MarginRequirement {
    pub initial_margin: Decimal,
    pub maintenance_margin: Decimal,
    pub span_margin: Decimal,
    pub exposure_margin: Decimal,
}

pub fn calculate_fno_margin(notional: Decimal, is_spread: bool) -> MarginRequirement {
    // If it's a hedged spread, margin requirement is significantly reduced (up to 70% discount)
    let discount = if is_spread { dec!(0.30) } else { Decimal::ONE };
    let span = notional * dec!(0.12) * discount;
    let exposure = notional * dec!(0.03) * discount;
    let initial = span + exposure;
    let maintenance = initial * dec!(0.80);

    MarginRequirement {
        initial_margin: initial,
        maintenance_margin: maintenance,
        span_margin: span,
        exposure_margin: exposure,
    }
}
