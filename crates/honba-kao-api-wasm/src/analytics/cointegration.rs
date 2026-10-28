use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CointegrationResult {
    pub hedge_ratio: f64,
    pub intercept: f64,
    pub spread_mean: f64,
    pub spread_std: f64,
    pub z_scores: Vec<f64>,
    pub half_life: f64,
}

pub fn calculate_cointegration(series_y: &[f64], series_x: &[f64]) -> Option<CointegrationResult> {
    let n = series_y.len().min(series_x.len());
    if n < 10 {
        return None;
    }

    let y = &series_y[..n];
    let x = &series_x[..n];

    let mean_x = x.iter().sum::<f64>() / n as f64;
    let mean_y = y.iter().sum::<f64>() / n as f64;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    for i in 0..n {
        cov += (x[i] - mean_x) * (y[i] - mean_y);
        var_x += (x[i] - mean_x).powi(2);
    }

    let hedge_ratio = if var_x > 1e-9 { cov / var_x } else { 1.0 };
    let intercept = mean_y - hedge_ratio * mean_x;

    let spread: Vec<f64> = (0..n)
        .map(|i| y[i] - (hedge_ratio * x[i] + intercept))
        .collect();
    let spread_mean = spread.iter().sum::<f64>() / n as f64;
    let spread_var = spread
        .iter()
        .map(|s| (s - spread_mean).powi(2))
        .sum::<f64>()
        / n as f64;
    let spread_std = spread_var.sqrt();

    let z_scores: Vec<f64> = if spread_std > 1e-9 {
        spread
            .iter()
            .map(|s| (s - spread_mean) / spread_std)
            .collect()
    } else {
        vec![0.0; n]
    };

    // Calculate half-life of mean reversion via AR(1) lag regression
    let mut dy = Vec::with_capacity(n - 1);
    let mut y_lag = Vec::with_capacity(n - 1);
    for i in 1..n {
        dy.push(spread[i] - spread[i - 1]);
        y_lag.push(spread[i - 1]);
    }
    let m_yl = y_lag.iter().sum::<f64>() / y_lag.len() as f64;
    let m_dy = dy.iter().sum::<f64>() / dy.len() as f64;
    let mut c_lag = 0.0;
    let mut v_lag = 0.0;
    for i in 0..dy.len() {
        c_lag += (y_lag[i] - m_yl) * (dy[i] - m_dy);
        v_lag += (y_lag[i] - m_yl).powi(2);
    }
    let lambda = if v_lag > 1e-9 { c_lag / v_lag } else { 0.0 };
    let half_life = if lambda < -1e-9 {
        -2.0_f64.ln() / lambda
    } else {
        f64::INFINITY
    };

    Some(CointegrationResult {
        hedge_ratio,
        intercept,
        spread_mean,
        spread_std,
        z_scores,
        half_life,
    })
}
