//! Black-Scholes-Merton pricing, Greeks and an implied-volatility solver
//! (Phase 3.4b). Pure `f64` math with no external crate: the normal CDF uses
//! the Abramowitz-Stegun 7.1.26 erf approximation (absolute error < 1.5e-7).
//!
//! Units: `spot`, `strike` and the returned `price` share one currency unit.
//! `time_years` is year fraction, `vol` and `rate` are decimals (0.18 = 18%).
//! Greeks are reported in conventional trader units:
//! * `theta` per calendar day,
//! * `vega` per 1 vol point (1% absolute change in vol),
//! * `rho` per 1 rate point (1% absolute change in rate).
//!
//! Degenerate inputs (expired, zero vol, non-positive spot or strike) return
//! the discounted intrinsic value with zeroed sensitivities.

use std::f64::consts::{FRAC_1_SQRT_2, PI};

/// Calendar days per year used to convert annual theta to per-day theta.
const DAYS_PER_YEAR: f64 = 365.0;

/// Price and first-order Greeks for one European option.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Greeks {
    pub price: f64,
    pub delta: f64,
    pub gamma: f64,
    /// Per calendar day.
    pub theta: f64,
    /// Per 1 vol point.
    pub vega: f64,
    /// Per 1 rate point.
    pub rho: f64,
}

/// Error function, Abramowitz-Stegun 7.1.26 (max absolute error 1.5e-7).
pub fn erf(x: f64) -> f64 {
    const P: f64 = 0.327_591_1;
    const A1: f64 = 0.254_829_592;
    const A2: f64 = -0.284_496_736;
    const A3: f64 = 1.421_413_741;
    const A4: f64 = -1.453_152_027;
    const A5: f64 = 1.061_405_429;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let ax = x.abs();
    let t = 1.0 / (1.0 + P * ax);
    let poly = ((((A5 * t + A4) * t + A3) * t + A2) * t + A1) * t;
    sign * (1.0 - poly * (-ax * ax).exp())
}

/// Standard normal cumulative distribution function.
pub fn norm_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x * FRAC_1_SQRT_2))
}

/// Standard normal probability density function.
pub fn norm_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * PI).sqrt()
}

fn intrinsic(spot: f64, strike: f64, is_call: bool) -> f64 {
    if is_call {
        (spot - strike).max(0.0)
    } else {
        (strike - spot).max(0.0)
    }
}

/// Black-Scholes-Merton price and Greeks for a European call or put.
pub fn bs_price_and_greeks(
    spot: f64,
    strike: f64,
    time_years: f64,
    vol: f64,
    rate: f64,
    is_call: bool,
) -> Greeks {
    let valid = spot > 0.0 && strike > 0.0 && time_years > 0.0 && vol > 0.0;
    if !valid || !spot.is_finite() || !strike.is_finite() {
        // With time left but zero vol the payoff is the discounted forward
        // intrinsic; with no time left it is plain intrinsic.
        let price = if time_years > 0.0 && spot > 0.0 && strike > 0.0 {
            let forward = spot - strike * (-rate * time_years).exp();
            if is_call {
                forward.max(0.0)
            } else {
                (-forward).max(0.0)
            }
        } else {
            intrinsic(spot, strike, is_call)
        };
        return Greeks {
            price,
            delta: 0.0,
            gamma: 0.0,
            theta: 0.0,
            vega: 0.0,
            rho: 0.0,
        };
    }

    let sqrt_t = time_years.sqrt();
    let vol_sqrt_t = vol * sqrt_t;
    let d1 = ((spot / strike).ln() + (rate + 0.5 * vol * vol) * time_years) / vol_sqrt_t;
    let d2 = d1 - vol_sqrt_t;
    let discount = (-rate * time_years).exp();
    let pdf_d1 = norm_pdf(d1);

    let (price, delta, theta_annual, rho_raw) = if is_call {
        let nd1 = norm_cdf(d1);
        let nd2 = norm_cdf(d2);
        (
            spot * nd1 - strike * discount * nd2,
            nd1,
            -spot * pdf_d1 * vol / (2.0 * sqrt_t) - rate * strike * discount * nd2,
            strike * time_years * discount * nd2,
        )
    } else {
        let nmd1 = norm_cdf(-d1);
        let nmd2 = norm_cdf(-d2);
        (
            strike * discount * nmd2 - spot * nmd1,
            norm_cdf(d1) - 1.0,
            -spot * pdf_d1 * vol / (2.0 * sqrt_t) + rate * strike * discount * nmd2,
            -strike * time_years * discount * nmd2,
        )
    };

    Greeks {
        price,
        delta,
        gamma: pdf_d1 / (spot * vol_sqrt_t),
        theta: theta_annual / DAYS_PER_YEAR,
        vega: spot * pdf_d1 * sqrt_t / 100.0,
        rho: rho_raw / 100.0,
    }
}

/// Implied volatility (decimal, 0.18 = 18%) that reproduces `target_price`,
/// found by bisection over `[1e-4, 5.0]` in 30 iterations (bracket width
/// shrinks by 2^30, roughly 5e-9 precision). Returns `f64::NAN` when the
/// target lies outside the attainable price range for the bracket.
pub fn implied_vol(
    target_price: f64,
    spot: f64,
    strike: f64,
    time_years: f64,
    rate: f64,
    is_call: bool,
) -> f64 {
    const LO: f64 = 1e-4;
    const HI: f64 = 5.0;
    const ITERATIONS: usize = 30;

    let price_at =
        |vol: f64| bs_price_and_greeks(spot, strike, time_years, vol, rate, is_call).price;

    let (mut lo, mut hi) = (LO, HI);
    if !target_price.is_finite() || price_at(lo) > target_price || price_at(hi) < target_price {
        return f64::NAN;
    }
    for _ in 0..ITERATIONS {
        let mid = 0.5 * (lo + hi);
        if price_at(mid) < target_price {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}
