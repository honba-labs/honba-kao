//! SIP (Systematic Investment Plan) compound projection.
//!
//! Pure `Decimal` math (Zero Floats Policy) for the monthly compounding
//! projection used by the SIP calculator card. Lives in `api-wasm::analytics`
//! per restructure.md Section 5.2 so the app crate stays free of numerical
//! algorithms.

use num_traits::ToPrimitive;
use rust_decimal::Decimal;

/// Result of a SIP compound projection.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SipProjection {
    /// Total amount invested (monthly × months).
    pub invested: Decimal,
    /// Projected future value of the SIP at the end of the term.
    pub projected: Decimal,
    /// Total gains (projected − invested).
    pub gains: Decimal,
}

/// Project the future value of a monthly SIP with end-of-month compounding.
///
/// Formula: `FV = m × (((1 + r)^n − 1) / r)` where `r = annual_return / 12 / 100`
/// and `n = years × 12`. When `r == 0`, `FV = m × n`.
pub fn calculate_sip_projection(
    monthly: Decimal,
    years: Decimal,
    annual_return_pct: Decimal,
) -> SipProjection {
    let r = annual_return_pct / Decimal::from(100) / Decimal::from(12);
    let n: u32 = years.to_i64().map(|v| v.max(0) as u32).unwrap_or(0);
    let invested = monthly * Decimal::from(n);

    let projected = if r.is_zero() {
        invested
    } else {
        let one_plus_r = Decimal::ONE + r;
        let mut factor = Decimal::ONE;
        for _ in 0..n {
            factor = factor * one_plus_r;
        }
        monthly * (factor - Decimal::ONE) / r
    };

    SipProjection {
        invested,
        projected,
        gains: projected - invested,
    }
}
