use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssetClass {
    Cash,
    GovernmentBonds,
    LargeCapEquity,
    MidCapEquity,
    SmallCapEquity,
    Commodities,
    Crypto,
}

/// Standard collateral haircut as a decimal *fraction* (e.g. `0.15` = 15%).
///
/// Returning a fraction (not a percentage) keeps units consistent with
/// `calculate_collateral_value`, which computes `value * (1 - haircut)`.
/// This prevents the 100x overstatement of collateral described in #55.
pub fn get_standard_haircut(asset: AssetClass) -> Decimal {
    match asset {
        AssetClass::Cash => dec!(0.0),
        AssetClass::GovernmentBonds => dec!(0.05),
        AssetClass::LargeCapEquity => dec!(0.15),
        AssetClass::MidCapEquity => dec!(0.25),
        AssetClass::SmallCapEquity => dec!(0.40),
        AssetClass::Commodities => dec!(0.20),
        AssetClass::Crypto => dec!(0.50),
    }
}

/// Collateral value after applying the haircut.
///
/// `haircut_pct` is a fraction (e.g. `0.15` for 15%), consistent with
/// `get_standard_haircut`. The result is clamped to non-negative so an
/// over-collateralized haircut never inverts into negative value.
pub fn calculate_collateral_value(balance: Decimal, price: Decimal, haircut_pct: Decimal) -> Decimal {
    let raw_val = balance * price;
    let multiplier = (Decimal::ONE - haircut_pct).max(Decimal::ZERO);
    raw_val * multiplier
}
