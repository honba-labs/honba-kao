use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ValueAtRisk {
    pub parametric_var_95: f64,
    pub parametric_var_99: f64,
    pub historical_var_95: f64,
    pub historical_var_99: f64,
    pub expected_shortfall_95: f64,
    pub expected_shortfall_99: f64,
}

pub fn calculate_var_and_cvar(daily_returns: &[f64]) -> ValueAtRisk {
    if daily_returns.is_empty() {
        return ValueAtRisk {
            parametric_var_95: 0.0,
            parametric_var_99: 0.0,
            historical_var_95: 0.0,
            historical_var_99: 0.0,
            expected_shortfall_95: 0.0,
            expected_shortfall_99: 0.0,
        };
    }

    let n = daily_returns.len() as f64;
    let mean = daily_returns.iter().sum::<f64>() / n;
    let variance = daily_returns
        .iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>()
        / n.max(1.0);
    let std = variance.sqrt();

    // Parametric VaR (normal distribution: 1.645 for 95%, 2.326 for 99%)
    let param_95 = (1.645 * std - mean).max(0.0);
    let param_99 = (2.326 * std - mean).max(0.0);

    // Historical VaR
    let mut sorted = daily_returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let idx_95 = ((sorted.len() as f64 * 0.05).floor() as usize).min(sorted.len() - 1);
    let idx_99 = ((sorted.len() as f64 * 0.01).floor() as usize).min(sorted.len() - 1);

    let hist_95 = (-sorted[idx_95]).max(0.0);
    let hist_99 = (-sorted[idx_99]).max(0.0);

    // Expected Shortfall (CVaR)
    let tail_95 = &sorted[..=idx_95];
    let es_95 = if !tail_95.is_empty() {
        (-tail_95.iter().sum::<f64>() / tail_95.len() as f64).max(0.0)
    } else {
        hist_95
    };

    let tail_99 = &sorted[..=idx_99];
    let es_99 = if !tail_99.is_empty() {
        (-tail_99.iter().sum::<f64>() / tail_99.len() as f64).max(0.0)
    } else {
        hist_99
    };

    ValueAtRisk {
        parametric_var_95: param_95,
        parametric_var_99: param_99,
        historical_var_95: hist_95,
        historical_var_99: hist_99,
        expected_shortfall_95: es_95,
        expected_shortfall_99: es_99,
    }
}
