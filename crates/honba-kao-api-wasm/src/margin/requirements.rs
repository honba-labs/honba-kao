use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MarginRequirement {
    pub initial_margin: f64,
    pub maintenance_margin: f64,
    pub span_margin: f64,
    pub exposure_margin: f64,
}

pub fn calculate_fno_margin(notional: f64, is_spread: bool) -> MarginRequirement {
    // If it's a hedged spread, margin requirement is significantly reduced (up to 70% discount)
    let discount = if is_spread { 0.30 } else { 1.0 };
    let span = notional * 0.12 * discount;
    let exposure = notional * 0.03 * discount;
    let initial = span + exposure;
    let maintenance = initial * 0.80;

    MarginRequirement {
        initial_margin: initial,
        maintenance_margin: maintenance,
        span_margin: span,
        exposure_margin: exposure,
    }
}
