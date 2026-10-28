use super::black_scholes::{norm_cdf, norm_pdf};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OptionGreeks {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
    pub vanna: f64,
    pub charm: f64,
}

pub fn calculate_greeks(s: f64, k: f64, t: f64, r: f64, sigma: f64, is_call: bool) -> OptionGreeks {
    if t <= 1e-6 || sigma <= 1e-6 {
        let delta = if is_call {
            if s > k { 1.0 } else { 0.0 }
        } else {
            if s < k { -1.0 } else { 0.0 }
        };
        return OptionGreeks {
            delta,
            gamma: 0.0,
            theta: 0.0,
            vega: 0.0,
            rho: 0.0,
            vanna: 0.0,
            charm: 0.0,
        };
    }

    let sqrt_t = t.sqrt();
    let d1 = ((s / k).ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    let n_prime_d1 = norm_pdf(d1);

    let delta = if is_call {
        norm_cdf(d1)
    } else {
        norm_cdf(d1) - 1.0
    };
    let gamma = n_prime_d1 / (s * sigma * sqrt_t);
    let vega = s * sqrt_t * n_prime_d1 / 100.0; // 1% change

    let theta = if is_call {
        (-(s * sigma * n_prime_d1) / (2.0 * sqrt_t) - r * k * (-r * t).exp() * norm_cdf(d2)) / 365.0
    } else {
        (-(s * sigma * n_prime_d1) / (2.0 * sqrt_t) + r * k * (-r * t).exp() * norm_cdf(-d2))
            / 365.0
    };

    let rho = if is_call {
        (k * t * (-r * t).exp() * norm_cdf(d2)) / 100.0
    } else {
        (-k * t * (-r * t).exp() * norm_cdf(-d2)) / 100.0
    };

    let vanna = -n_prime_d1 * d2 / sigma;
    let charm = if is_call {
        -n_prime_d1 * (2.0 * r * t - d2 * sigma * sqrt_t) / (2.0 * t * sigma * sqrt_t)
    } else {
        -n_prime_d1 * (2.0 * r * t - d2 * sigma * sqrt_t) / (2.0 * t * sigma * sqrt_t)
    };

    OptionGreeks {
        delta,
        gamma,
        theta,
        vega,
        rho,
        vanna,
        charm,
    }
}
