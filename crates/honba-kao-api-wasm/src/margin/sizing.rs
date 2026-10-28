use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PositionSizing {
    pub kelly_fraction: Decimal,
    pub volatility_target_size: Decimal,
    pub recommended_notional: Decimal,
}

pub fn calculate_kelly_criterion(win_rate: Decimal, win_loss_ratio: Decimal) -> Decimal {
    if win_loss_ratio.is_zero() {
        return Decimal::ZERO;
    }
    let q = Decimal::ONE - win_rate;
    let kelly = (win_rate * win_loss_ratio - q) / win_loss_ratio;
    kelly.max(Decimal::ZERO).min(Decimal::ONE)
}

pub fn calculate_volatility_target_size(
    portfolio_value: Decimal,
    target_vol_annual: Decimal,
    asset_vol_annual: Decimal,
) -> Decimal {
    if asset_vol_annual.is_zero() {
        return Decimal::ZERO;
    }
    portfolio_value * (target_vol_annual / asset_vol_annual)
}

pub fn calculate_position_sizing(
    portfolio_value: Decimal,
    win_rate: Decimal,
    win_loss_ratio: Decimal,
    target_vol_annual: Decimal,
    asset_vol_annual: Decimal,
) -> PositionSizing {
    let kelly = calculate_kelly_criterion(win_rate, win_loss_ratio);
    let half_kelly = kelly * dec!(0.5); // Institutional conservative half-Kelly
    let vol_size =
        calculate_volatility_target_size(portfolio_value, target_vol_annual, asset_vol_annual);
    let rec = (portfolio_value * half_kelly).min(vol_size);

    PositionSizing {
        kelly_fraction: kelly,
        volatility_target_size: vol_size,
        recommended_notional: rec,
    }
}

/// Maximum risk budget for a trade: `account_equity * (risk_pct / 100)`.
pub fn calculate_risk_amount(account_equity: Decimal, risk_pct: Decimal) -> Decimal {
    account_equity * (risk_pct / Decimal::from(100))
}

/// Per-share risk distance: `|entry_price - stop_loss|`.
pub fn calculate_per_share_risk(entry_price: Decimal, stop_loss: Decimal) -> Decimal {
    (entry_price - stop_loss).abs()
}

/// Suggested whole-unit quantity: `(risk_amount / per_share_risk).trunc()`.
/// Returns zero when per-share risk is zero to avoid division by zero.
pub fn calculate_suggested_shares(risk_amount: Decimal, per_share_risk: Decimal) -> Decimal {
    if per_share_risk.is_zero() {
        return Decimal::ZERO;
    }
    (risk_amount / per_share_risk).trunc()
}
