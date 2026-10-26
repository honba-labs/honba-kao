//! Sanity checks for the Black-Scholes-Merton pricer and IV solver (Phase 3.4b).

use honba_kao_app::features::fno::black_scholes::{bs_price_and_greeks, implied_vol, norm_cdf};

/// Hull reference case: S = K = 100, T = 1y, r = 5%, sigma = 20%.
const S: f64 = 100.0;
const K: f64 = 100.0;
const T: f64 = 1.0;
const R: f64 = 0.05;
const SIGMA: f64 = 0.20;

fn close(actual: f64, expected: f64, tol: f64) {
    assert!(
        (actual - expected).abs() <= tol,
        "expected {expected} +/- {tol}, got {actual}"
    );
}

#[test]
fn normal_cdf_reference_points() {
    close(norm_cdf(0.0), 0.5, 1e-7);
    close(norm_cdf(1.0), 0.841_344_746, 1e-6);
    close(norm_cdf(-1.96), 0.024_997_895, 1e-6);
}

#[test]
fn atm_call_matches_reference_price_and_greeks() {
    let g = bs_price_and_greeks(S, K, T, SIGMA, R, true);
    close(g.price, 10.4506, 1e-3);
    close(g.delta, 0.6368, 1e-3);
    close(g.gamma, 0.01876, 1e-4);
    // Vega per 1 vol point: 37.52 / 100.
    close(g.vega, 0.3752, 1e-3);
    // Theta per calendar day: -6.414 / 365.
    close(g.theta, -6.414 / 365.0, 1e-3);
    // Rho per 1 rate point: 53.23 / 100.
    close(g.rho, 0.5323, 1e-3);
}

#[test]
fn atm_put_matches_reference_price_and_greeks() {
    let g = bs_price_and_greeks(S, K, T, SIGMA, R, false);
    close(g.price, 5.5735, 1e-3);
    close(g.delta, 0.6368 - 1.0, 1e-3);
    close(g.rho, -0.4189, 1e-3);
}

#[test]
fn put_call_parity_holds() {
    for &(spot, strike, days, vol) in
        &[(24_850.0, 25_000.0, 30.0, 0.18), (100.0, 80.0, 180.0, 0.35)]
    {
        let t = days / 365.0;
        let c = bs_price_and_greeks(spot, strike, t, vol, R, true);
        let p = bs_price_and_greeks(spot, strike, t, vol, R, false);
        close(c.price - p.price, spot - strike * (-R * t).exp(), 1e-6);
        close(c.gamma, p.gamma, 1e-12);
        close(c.vega, p.vega, 1e-12);
    }
}

#[test]
fn expired_option_returns_intrinsic_value() {
    let itm = bs_price_and_greeks(110.0, 100.0, 0.0, SIGMA, R, true);
    close(itm.price, 10.0, 1e-12);
    assert_eq!(itm.delta, 0.0);
    let otm = bs_price_and_greeks(110.0, 100.0, 0.0, SIGMA, R, false);
    close(otm.price, 0.0, 1e-12);
}

#[test]
fn implied_vol_round_trips() {
    for &is_call in &[true, false] {
        let target = bs_price_and_greeks(S, 105.0, T, 0.27, R, is_call).price;
        let iv = implied_vol(target, S, 105.0, T, R, is_call);
        close(iv, 0.27, 1e-5);
    }
}

#[test]
fn implied_vol_rejects_unattainable_price() {
    // A call can never be worth more than the spot.
    assert!(implied_vol(150.0, S, K, T, R, true).is_nan());
}
