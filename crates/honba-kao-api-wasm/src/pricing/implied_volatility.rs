use super::black_scholes::{black_scholes_call, black_scholes_put};
use super::greeks::calculate_greeks;

pub fn calculate_implied_volatility(
    price: f64,
    s: f64,
    k: f64,
    t: f64,
    r: f64,
    is_call: bool,
) -> Option<f64> {
    if price <= 0.0 || t <= 0.0 || s <= 0.0 || k <= 0.0 {
        return None;
    }

    // Intrinsic value check
    let intrinsic = if is_call {
        (s - k).max(0.0)
    } else {
        (k - s).max(0.0)
    };
    if price < intrinsic {
        return None;
    }

    // Newton-Raphson iteration with fallback to bisection
    let mut sigma = 0.30;
    let max_iter = 100;
    let tol = 1e-4;

    for _ in 0..max_iter {
        let model_price = if is_call {
            black_scholes_call(s, k, t, r, sigma)
        } else {
            black_scholes_put(s, k, t, r, sigma)
        };

        let diff = model_price - price;
        if diff.abs() < tol {
            return Some(sigma);
        }

        let vega = calculate_greeks(s, k, t, r, sigma, is_call).vega * 100.0;
        if vega > 1e-6 {
            let next_sigma = sigma - diff / vega;
            if next_sigma > 0.001 && next_sigma < 5.0 {
                sigma = next_sigma;
                continue;
            }
        }

        // Bisection fallback
        break;
    }

    // Brent / Bisection fallback
    let mut low = 0.001;
    let mut high = 5.0;
    for _ in 0..50 {
        let mid = (low + high) * 0.5;
        let p = if is_call {
            black_scholes_call(s, k, t, r, mid)
        } else {
            black_scholes_put(s, k, t, r, mid)
        };

        if (p - price).abs() < tol {
            return Some(mid);
        }

        if p > price {
            high = mid;
        } else {
            low = mid;
        }
    }

    Some((low + high) * 0.5)
}
