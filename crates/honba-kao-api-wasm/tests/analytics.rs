//! Integration tests for `honba-kao-api-wasm::analytics`.
//!
//! Covers drawdown, performance ratios, return metrics, VaR/CVaR, and
//! cointegration. Exercises the public API surface end-to-end.

use honba_api_wasm::analytics::*;

#[test]
fn drawdown_monotonic_curve_no_loss() {
    let curve = vec![100.0, 110.0, 120.0, 130.0];
    let dd = calculate_drawdown(&curve);
    assert!((dd.max_drawdown - 0.0).abs() < 1e-9);
    assert!((dd.max_drawdown_pct - 0.0).abs() < 1e-9);
    assert_eq!(dd.underwater_curve.len(), 4);
}

#[test]
fn drawdown_known_peak_trough() {
    let curve = vec![100.0, 120.0, 90.0, 110.0];
    let dd = calculate_drawdown(&curve);
    assert!((dd.max_drawdown - 30.0).abs() < 1e-9);
    assert!((dd.max_drawdown_pct - 0.25).abs() < 1e-9);
    assert_eq!(dd.peak_index, 1);
    assert_eq!(dd.trough_index, 2);
    assert_eq!(dd.underwater_curve.len(), curve.len());
}

#[test]
fn drawdown_empty() {
    let dd = calculate_drawdown(&[]);
    assert!((dd.max_drawdown - 0.0).abs() < 1e-9);
    assert_eq!(dd.recovery_duration_bars, None);
}

#[test]
fn ratios_empty_input() {
    let r = calculate_performance_ratios(&[], &[], 0.0, 0.05);
    assert!((r.sharpe_ratio - 0.0).abs() < 1e-9);
    assert!((r.sortino_ratio - 0.0).abs() < 1e-9);
    assert!((r.calmar_ratio - 0.0).abs() < 1e-9);
    assert!((r.profit_factor - 0.0).abs() < 1e-9);
    assert!((r.win_rate - 0.0).abs() < 1e-9);
}

#[test]
fn ratios_win_rate_profit_factor() {
    let returns = vec![0.01, 0.02, -0.005, 0.015];
    let pnl = vec![100.0, -50.0, 200.0];
    let r = calculate_performance_ratios(&returns, &pnl, 0.0, 0.05);
    assert!((r.win_rate - (2.0 / 3.0)).abs() < 1e-9);
    assert!((r.profit_factor - 6.0).abs() < 1e-9); // (100+200)/50 = 6.0
}

#[test]
fn ratios_all_losses() {
    let r = calculate_performance_ratios(&[0.01, 0.02], &[-100.0, -50.0], 0.0, 0.05);
    assert!((r.profit_factor - 0.0).abs() < 1e-9);
    assert!((r.win_rate - 0.0).abs() < 1e-9);
}

#[test]
fn returns_daily() {
    let prices = vec![100.0, 110.0, 121.0];
    let rets = calculate_daily_returns(&prices);
    assert_eq!(rets.len(), 2);
    assert!((rets[0] - 0.1).abs() < 1e-9);
}

#[test]
fn returns_single_price_empty() {
    assert!(calculate_daily_returns(&[100.0]).is_empty());
}

#[test]
fn returns_metrics_total() {
    let prices = vec![100.0, 150.0];
    let m = calculate_return_metrics(&prices, None, 252.0);
    assert!((m.total_return - 0.5).abs() < 1e-9);
    assert_eq!(m.daily_returns.len(), 1);
}

#[test]
fn returns_metrics_with_benchmark() {
    let p = vec![100.0, 110.0, 121.0];
    let b = vec![100.0, 105.0, 110.25];
    let m = calculate_return_metrics(&p, Some(&b), 252.0);
    assert!((m.beta - 1.0).abs() < 0.2);
}

#[test]
fn var_ordering_99_ge_95() {
    let rets: Vec<f64> = (0..100).map(|i| (i as f64 - 50.0) / 1000.0).collect();
    let v = calculate_var_and_cvar(&rets);
    assert!(v.parametric_var_99 >= v.parametric_var_95);
    assert!(v.historical_var_99 >= v.historical_var_95);
    assert!(v.expected_shortfall_95 >= v.historical_var_95);
}

#[test]
fn var_empty_returns_zeros() {
    let v = calculate_var_and_cvar(&[]);
    assert!((v.parametric_var_95 - 0.0).abs() < 1e-9);
    assert!((v.expected_shortfall_99 - 0.0).abs() < 1e-9);
}

#[test]
fn cointegration_exactly_ten_points() {
    let y: Vec<f64> = (1..=10).map(|i| i as f64 * 2.0 + 1.0).collect();
    let x: Vec<f64> = (1..=10).map(|i| i as f64).collect();
    let r = calculate_cointegration(&y, &x).unwrap();
    assert!((r.hedge_ratio - 2.0).abs() < 1e-6);
    assert!((r.intercept - 1.0).abs() < 1e-6);
    assert_eq!(r.z_scores.len(), 10);
}

#[test]
fn cointegration_short_series_none() {
    assert!(calculate_cointegration(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]).is_none());
}

#[test]
fn cointegration_unequal_length_truncates() {
    let y: Vec<f64> = (1..=12).map(|i| i as f64).collect();
    let x: Vec<f64> = (1..=10).map(|i| i as f64 * 2.0).collect();
    let r = calculate_cointegration(&y, &x).unwrap();
    assert_eq!(r.z_scores.len(), 10);
}
