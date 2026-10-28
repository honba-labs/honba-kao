//! Integration tests for `honba-kao-api-wasm::margin`.
//!
//! Covers asset-class haircuts, F&O margin requirements, position sizing
//! (Kelly / vol-target), liquidation buffer, and margin utilization.

use honba_api_wasm::margin::*;

#[test]
fn haircut_ranking() {
    assert!(get_standard_haircut(AssetClass::Cash) < get_standard_haircut(AssetClass::GovernmentBonds));
    assert!(get_standard_haircut(AssetClass::GovernmentBonds) < get_standard_haircut(AssetClass::LargeCapEquity));
    assert!(get_standard_haircut(AssetClass::LargeCapEquity) < get_standard_haircut(AssetClass::SmallCapEquity));
    assert!(get_standard_haircut(AssetClass::Crypto) > get_standard_haircut(AssetClass::SmallCapEquity));
}

#[test]
fn cash_zero_haircut() {
    assert!((get_standard_haircut(AssetClass::Cash) - 0.0).abs() < 1e-9);
}

#[test]
fn collateral_value_applies_haircut() {
    // 100 units @ ₹100, 20% haircut => 10000 * 0.8 = 8000
    assert!((calculate_collateral_value(100.0, 100.0, 20.0) - 8000.0).abs() < 1e-9);
    // 100% haircut => 0
    assert!((calculate_collateral_value(100.0, 100.0, 100.0) - 0.0).abs() < 1e-9);
    // clamped: never negative
    assert!(calculate_collateral_value(100.0, 100.0, 150.0) >= 0.0);
}

#[test]
fn fno_margin_spread_discount() {
    let normal = calculate_fno_margin(1_000_000.0, false);
    let spread = calculate_fno_margin(1_000_000.0, true);
    // spread multiplier is 0.30 (i.e. 70% discount on margin)
    assert!((spread.span_margin - normal.span_margin * 0.30).abs() < 1e-6);
    assert!((spread.initial_margin - normal.initial_margin * 0.30).abs() < 1e-6);
    assert!((spread.maintenance_margin - normal.maintenance_margin * 0.30).abs() < 1e-6);
}

#[test]
fn kelly_clamped() {
    // win_rate=0.6, win_loss_ratio=2.0 => kelly = (0.6*2 - 0.4)/2 = 0.4
    let k = calculate_kelly_criterion(0.6, 2.0);
    assert!((k - 0.4).abs() < 1e-9);
    // extreme -> clamped to [0,1]
    assert!((calculate_kelly_criterion(1.0, 100.0) - 1.0).abs() < 1e-9);
    assert!((calculate_kelly_criterion(0.0, 1.0) - 0.0).abs() < 1e-9);
}

#[test]
fn vol_target_size() {
    let size = calculate_volatility_target_size(1_000_000.0, 0.10, 0.20);
    assert!((size - 500_000.0).abs() < 1e-9);
    // zero asset vol -> 0
    assert!((calculate_volatility_target_size(1_000_000.0, 0.10, 0.0) - 0.0).abs() < 1e-9);
}

#[test]
fn position_sizing_recommends_min() {
    let ps = calculate_position_sizing(1_000_000.0, 0.6, 2.0, 0.10, 0.20);
    // kelly=0.4, half=0.2 => 200k; vol target=500k; recommended=min=200k
    assert!((ps.kelly_fraction - 0.4).abs() < 1e-9);
    assert!((ps.recommended_notional - 200_000.0).abs() < 1e-9);
}

#[test]
fn liquidation_buffer_long() {
    let lb = calculate_liquidation_buffer(100.0, 10.0, 500.0, 0.25, true);
    // notional=1000, margin req=250, util=250/500*100=50%
    assert!((lb.current_utilization_ratio - 50.0).abs() < 1e-9);
    assert!((lb.buffer_pct - 35.0).abs() < 1e-9); // 85 - 50
    assert!(lb.est_liquidation_price.is_some());
}

#[test]
fn liquidation_buffer_zero_size() {
    let lb = calculate_liquidation_buffer(100.0, 0.0, 500.0, 0.25, true);
    assert!((lb.current_utilization_ratio - 0.0).abs() < 1e-9);
    assert!(lb.est_liquidation_price.is_none());
}

#[test]
fn margin_utilization_warning_thresholds() {
    let m = calculate_margin_utilization(1_000_000.0, 800_000.0);
    assert!((m.utilization_ratio - 80.0).abs() < 1e-9);
    assert!(m.is_warning);
    assert!(!m.is_critical);
    let m2 = calculate_margin_utilization(1_000_000.0, 900_000.0);
    assert!(m2.is_critical);
    let m3 = calculate_margin_utilization(1_000_000.0, 500_000.0);
    assert!(!m3.is_warning);
    assert!(!m3.is_critical);
}
