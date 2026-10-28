use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PositionSizing {
    pub kelly_fraction: f64,
    pub volatility_target_size: f64,
    pub recommended_notional: f64,
}

pub fn calculate_kelly_criterion(win_rate: f64, win_loss_ratio: f64) -> f64 {
    if win_loss_ratio <= 0.0 {
        return 0.0;
    }
    let q = 1.0 - win_rate;
    let kelly = (win_rate * win_loss_ratio - q) / win_loss_ratio;
    kelly.clamp(0.0, 1.0)
}

pub fn calculate_volatility_target_size(
    portfolio_value: f64,
    target_vol_annual: f64,
    asset_vol_annual: f64,
) -> f64 {
    if asset_vol_annual <= 1e-9 {
        return 0.0;
    }
    portfolio_value * (target_vol_annual / asset_vol_annual)
}

pub fn calculate_position_sizing(
    portfolio_value: f64,
    win_rate: f64,
    win_loss_ratio: f64,
    target_vol_annual: f64,
    asset_vol_annual: f64,
) -> PositionSizing {
    let kelly = calculate_kelly_criterion(win_rate, win_loss_ratio);
    let half_kelly = kelly * 0.5; // Institutional conservative half-Kelly
    let vol_size =
        calculate_volatility_target_size(portfolio_value, target_vol_annual, asset_vol_annual);
    let rec = (portfolio_value * half_kelly).min(vol_size);

    PositionSizing {
        kelly_fraction: kelly,
        volatility_target_size: vol_size,
        recommended_notional: rec,
    }
}
