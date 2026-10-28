use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LiquidationBuffer {
    pub liquidation_threshold_ratio: f64,
    pub current_utilization_ratio: f64,
    pub buffer_pct: f64,
    pub est_liquidation_price: Option<f64>,
}

pub fn calculate_liquidation_buffer(
    entry_price: f64,
    position_size: f64,
    collateral: f64,
    maintenance_margin_rate: f64,
    is_long: bool,
) -> LiquidationBuffer {
    if position_size.abs() < 1e-9 || entry_price <= 0.0 {
        return LiquidationBuffer {
            liquidation_threshold_ratio: 85.0,
            current_utilization_ratio: 0.0,
            buffer_pct: 85.0,
            est_liquidation_price: None,
        };
    }

    let notional = entry_price * position_size.abs();
    let current_util = if collateral > 0.0 {
        ((notional * maintenance_margin_rate) / collateral) * 100.0
    } else {
        100.0
    };

    let est_liq = if is_long {
        let max_loss = collateral - (notional * maintenance_margin_rate);
        (entry_price - (max_loss / position_size.abs())).max(0.0)
    } else {
        let max_loss = collateral - (notional * maintenance_margin_rate);
        entry_price + (max_loss / position_size.abs())
    };

    let threshold = 85.0;
    let buffer = (threshold - current_util).max(0.0);

    LiquidationBuffer {
        liquidation_threshold_ratio: threshold,
        current_utilization_ratio: current_util,
        buffer_pct: buffer,
        est_liquidation_price: Some(est_liq),
    }
}
