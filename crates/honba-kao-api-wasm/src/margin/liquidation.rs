use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LiquidationBuffer {
    pub liquidation_threshold_ratio: Decimal,
    pub current_utilization_ratio: Decimal,
    pub buffer_pct: Decimal,
    pub est_liquidation_price: Option<Decimal>,
}

pub fn calculate_liquidation_buffer(
    entry_price: Decimal,
    position_size: Decimal,
    collateral: Decimal,
    maintenance_margin_rate: Decimal,
    is_long: bool,
) -> LiquidationBuffer {
    let threshold = dec!(85.0);
    let zero = Decimal::ZERO;

    // Guard against non-positive spot/size inputs (#59).
    if position_size.is_zero() || entry_price.is_zero() || entry_price.is_sign_negative() {
        return LiquidationBuffer {
            liquidation_threshold_ratio: threshold,
            current_utilization_ratio: zero,
            buffer_pct: threshold,
            est_liquidation_price: None,
        };
    }

    let notional = entry_price * position_size.abs();
    let maintenance_req = notional * maintenance_margin_rate;
    let current_util = if collateral > zero {
        (maintenance_req / collateral) * dec!(100.0)
    } else {
        dec!(100.0)
    };

    // #56: an active maintenance breach (collateral <= maintenance requirement)
    // means the position is immediately liquidatable; buffer collapses to zero
    // and the estimated liquidation price is the entry price (a long never
    // liquidates as price rises).
    let in_breach = collateral <= maintenance_req;
    let est_liq = if is_long {
        if in_breach {
            Some(entry_price)
        } else {
            let max_loss = collateral - maintenance_req;
            Some((entry_price - (max_loss / position_size.abs())).max(zero))
        }
    } else {
        if in_breach {
            Some(entry_price)
        } else {
            let max_loss = collateral - maintenance_req;
            Some(entry_price + (max_loss / position_size.abs()))
        }
    };

    let buffer = if in_breach {
        zero
    } else {
        (threshold - current_util).max(zero)
    };

    LiquidationBuffer {
        liquidation_threshold_ratio: threshold,
        current_utilization_ratio: current_util,
        buffer_pct: buffer,
        est_liquidation_price: est_liq,
    }
}
