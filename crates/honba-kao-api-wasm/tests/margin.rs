//! Integration tests for `honba-kao-api-wasm::margin`.
//!
//! Covers asset-class haircuts, F&O margin requirements, position sizing
//! (Kelly / vol-target), liquidation buffer, and margin utilization.
//! All monetary math uses `rust_decimal::Decimal` (Zero Floats policy, #54).

use honba_api_wasm::margin::*;
use rust_decimal_macros::dec;

#[test]
fn haircut_ranking() {
    assert!(get_standard_haircut(AssetClass::Cash) < get_standard_haircut(AssetClass::GovernmentBonds));
    assert!(get_standard_haircut(AssetClass::GovernmentBonds) < get_standard_haircut(AssetClass::LargeCapEquity));
    assert!(get_standard_haircut(AssetClass::LargeCapEquity) < get_standard_haircut(AssetClass::SmallCapEquity));
    assert!(get_standard_haircut(AssetClass::Crypto) > get_standard_haircut(AssetClass::SmallCapEquity));
}

#[test]
fn cash_zero_haircut() {
    assert_eq!(get_standard_haircut(AssetClass::Cash), dec!(0.0));
}

#[test]
fn collateral_value_applies_haircut() {
    // #55: haircut is now a fraction consistent with get_standard_haircut.
    // 100 units @ ₹100, 20% haircut (0.20) => 10000 * 0.8 = 8000
    assert_eq!(
        calculate_collateral_value(dec!(100), dec!(100), dec!(0.20)),
        dec!(8000)
    );
    // 100% haircut => 0
    assert_eq!(calculate_collateral_value(dec!(100), dec!(100), dec!(1.0)), dec!(0));
    // clamped: never negative
    assert!(calculate_collateral_value(dec!(100), dec!(100), dec!(1.5)) >= dec!(0));
}

#[test]
fn fno_margin_spread_discount() {
    let normal = calculate_fno_margin(dec!(1_000_000), false);
    let spread = calculate_fno_margin(dec!(1_000_000), true);
    assert_eq!(spread.span_margin, normal.span_margin * dec!(0.30));
    assert_eq!(spread.initial_margin, normal.initial_margin * dec!(0.30));
    assert_eq!(spread.maintenance_margin, normal.maintenance_margin * dec!(0.30));
}

#[test]
fn kelly_clamped() {
    // win_rate=0.6, win_loss_ratio=2.0 => kelly = (0.6*2 - 0.4)/2 = 0.4
    assert_eq!(calculate_kelly_criterion(dec!(0.6), dec!(2.0)), dec!(0.4));
    // extreme -> clamped to [0,1]
    assert_eq!(calculate_kelly_criterion(dec!(1.0), dec!(100.0)), dec!(1.0));
    assert_eq!(calculate_kelly_criterion(dec!(0.0), dec!(1.0)), dec!(0.0));
}

#[test]
fn vol_target_size() {
    assert_eq!(
        calculate_volatility_target_size(dec!(1_000_000), dec!(0.10), dec!(0.20)),
        dec!(500_000)
    );
    // zero asset vol -> 0
    assert_eq!(
        calculate_volatility_target_size(dec!(1_000_000), dec!(0.10), dec!(0.0)),
        dec!(0)
    );
}

#[test]
fn position_sizing_recommends_min() {
    let ps = calculate_position_sizing(dec!(1_000_000), dec!(0.6), dec!(2.0), dec!(0.10), dec!(0.20));
    // kelly=0.4, half=0.2 => 200k; vol target=500k; recommended=min=200k
    assert_eq!(ps.kelly_fraction, dec!(0.4));
    assert_eq!(ps.recommended_notional, dec!(200_000));
}

#[test]
fn liquidation_buffer_long() {
    let lb = calculate_liquidation_buffer(dec!(100), dec!(10), dec!(500), dec!(0.25), true);
    // notional=1000, margin req=250, util=250/500*100=50%
    assert_eq!(lb.current_utilization_ratio, dec!(50));
    assert_eq!(lb.buffer_pct, dec!(35)); // 85 - 50
    assert!(lb.est_liquidation_price.is_some());
    // #56: long liquidation price must be <= entry price (75 <= 100)
    assert!(lb.est_liquidation_price.unwrap() <= dec!(100));
}

#[test]
fn liquidation_buffer_zero_size() {
    let lb = calculate_liquidation_buffer(dec!(100), dec!(0), dec!(500), dec!(0.25), true);
    assert_eq!(lb.current_utilization_ratio, dec!(0));
    assert!(lb.est_liquidation_price.is_none());
}

// #56: an undercollateralized long is immediately liquidatable; buffer collapses
// to zero and the estimated liquidation price is the entry price (never above it).
#[test]
fn liquidation_buffer_undercollateralized_long() {
    let lb = calculate_liquidation_buffer(dec!(100), dec!(10), dec!(100), dec!(0.50), true);
    // notional=1000, maintenance_req=500 > collateral=100 => active breach
    assert_eq!(lb.buffer_pct, dec!(0));
    assert_eq!(lb.est_liquidation_price, Some(dec!(100)));
}

#[test]
fn margin_utilization_warning_thresholds() {
    let m = calculate_margin_utilization(dec!(1_000_000), dec!(800_000));
    assert_eq!(m.utilization_ratio, dec!(80));
    assert!(m.is_warning);
    assert!(!m.is_critical);
    let m2 = calculate_margin_utilization(dec!(1_000_000), dec!(900_000));
    assert!(m2.is_critical);
    let m3 = calculate_margin_utilization(dec!(1_000_000), dec!(500_000));
    assert!(!m3.is_warning);
    assert!(!m3.is_critical);
}
