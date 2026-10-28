//! Integration tests for `honba-kao-api-wasm::pricing`.
//!
//! Covers Black-Scholes pricing, option Greeks, implied volatility inversion,
//! and yield term structure.

use honba_api_wasm::pricing::*;

#[test]
fn call_at_expiry_intrinsic() {
    assert!((black_scholes_call(110.0, 100.0, 0.0, 0.05, 0.2) - 10.0).abs() < 1e-9);
    assert!((black_scholes_call(90.0, 100.0, 0.0, 0.05, 0.2) - 0.0).abs() < 1e-9);
}

#[test]
fn put_at_expiry_intrinsic() {
    assert!((black_scholes_put(90.0, 100.0, 0.0, 0.05, 0.2) - 10.0).abs() < 1e-9);
    assert!((black_scholes_put(110.0, 100.0, 0.0, 0.05, 0.2) - 0.0).abs() < 1e-9);
}

#[test]
fn call_price_increases_with_volatility() {
    let low = black_scholes_call(100.0, 100.0, 1.0, 0.05, 0.1);
    let high = black_scholes_call(100.0, 100.0, 1.0, 0.05, 0.5);
    assert!(high > low);
}

#[test]
fn put_call_parity() {
    let s = 100.0;
    let k = 100.0;
    let t = 1.0;
    let r = 0.05;
    let sigma = 0.2;
    let call = black_scholes_call(s, k, t, r, sigma);
    let put = black_scholes_put(s, k, t, r, sigma);
    assert!((call + k * (-r * t).exp() - (put + s)).abs() < 1e-6);
}

#[test]
fn norm_cdf_monotonic_bounded() {
    assert!((norm_cdf(-10.0) - 0.0).abs() < 1e-6);
    assert!((norm_cdf(0.0) - 0.5).abs() < 1e-9);
    assert!((norm_cdf(10.0) - 1.0).abs() < 1e-6);
    assert!(norm_cdf(1.0) > norm_cdf(0.0));
}

#[test]
fn greeks_call_delta_range() {
    let g = calculate_greeks(100.0, 100.0, 1.0, 0.05, 0.2, true);
    assert!(g.delta > 0.0 && g.delta < 1.0);
    assert!(g.gamma > 0.0);
    assert!(g.vega > 0.0);
}

#[test]
fn greeks_put_delta_negative() {
    let g = calculate_greeks(100.0, 100.0, 1.0, 0.05, 0.2, false);
    assert!(g.delta < 0.0 && g.delta > -1.0);
}

#[test]
fn greeks_call_put_symmetry_atm() {
    let gc = calculate_greeks(100.0, 100.0, 1.0, 0.05, 0.2, true);
    let gp = calculate_greeks(100.0, 100.0, 1.0, 0.05, 0.2, false);
    assert!((gc.gamma - gp.gamma).abs() < 1e-9);
    assert!((gc.vega - gp.vega).abs() < 1e-9);
}

#[test]
fn greeks_at_expiry_call_itm() {
    let g = calculate_greeks(110.0, 100.0, 0.0, 0.05, 0.2, true);
    assert!((g.delta - 1.0).abs() < 1e-9);
    assert_eq!(g.gamma, 0.0);
    assert_eq!(g.vega, 0.0);
}

#[test]
fn iv_round_trip_call() {
    let s = 100.0;
    let k = 100.0;
    let t = 1.0;
    let r = 0.05;
    let sigma = 0.25;
    let price = black_scholes_call(s, k, t, r, sigma);
    let implied = calculate_implied_volatility(price, s, k, t, r, true);
    assert!(implied.is_some());
    assert!((implied.unwrap() - sigma).abs() < 1e-3);
}

#[test]
fn iv_round_trip_put() {
    let s = 100.0;
    let k = 100.0;
    let t = 1.0;
    let r = 0.05;
    let sigma = 0.20;
    let price = black_scholes_put(s, k, t, r, sigma);
    let implied = calculate_implied_volatility(price, s, k, t, r, false);
    assert!(implied.is_some());
    assert!((implied.unwrap() - sigma).abs() < 1e-3);
}

#[test]
fn iv_below_intrinsic_none() {
    assert!(calculate_implied_volatility(1.0, 100.0, 50.0, 1.0, 0.05, true).is_none());
}

#[test]
fn iv_zero_price_none() {
    assert!(calculate_implied_volatility(0.0, 100.0, 100.0, 1.0, 0.05, true).is_none());
}

#[test]
fn discount_factor_zero_rate() {
    assert!((discount_factor(0.0, 10.0) - 1.0).abs() < 1e-9);
    assert!((discount_factor(0.05, 0.0) - 1.0).abs() < 1e-9);
}

#[test]
fn pv_fv_inverse() {
    let cf = 1000.0;
    let pv = present_value(cf, 0.05, 2.0);
    assert!((future_value(pv, 0.05, 2.0) - cf).abs() < 1e-6);
}

#[test]
fn interpolate_yield_bounds_and_midpoint() {
    let curve = vec![
        YieldPoint { tenor_months: 1.0, rate: 0.05 },
        YieldPoint { tenor_months: 13.0, rate: 0.07 },
    ];
    assert!((interpolate_yield(&curve, 0.5) - 0.05).abs() < 1e-9);
    assert!((interpolate_yield(&curve, 24.0) - 0.07).abs() < 1e-9);
    assert!((interpolate_yield(&curve, 7.0) - 0.06).abs() < 1e-9);
}

#[test]
fn interpolate_yield_empty() {
    assert!((interpolate_yield(&[], 6.0) - 0.0).abs() < 1e-9);
}
