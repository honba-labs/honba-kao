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

pub fn get_standard_haircut(asset: AssetClass) -> f64 {
    match asset {
        AssetClass::Cash => 0.0,
        AssetClass::GovernmentBonds => 0.05,
        AssetClass::LargeCapEquity => 0.15,
        AssetClass::MidCapEquity => 0.25,
        AssetClass::SmallCapEquity => 0.40,
        AssetClass::Commodities => 0.20,
        AssetClass::Crypto => 0.50,
    }
}

pub fn calculate_collateral_value(balance: f64, price: f64, haircut_pct: f64) -> f64 {
    let raw_val = balance * price;
    let multiplier = (1.0 - (haircut_pct / 100.0)).clamp(0.0, 1.0);
    raw_val * multiplier
}
