use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReturnMetrics {
    pub total_return: f64,
    pub cagr: f64,
    pub daily_returns: Vec<f64>,
    pub alpha: f64,
    pub beta: f64,
}

pub fn calculate_daily_returns(prices: &[f64]) -> Vec<f64> {
    if prices.len() < 2 {
        return Vec::new();
    }
    prices.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect()
}

pub fn calculate_return_metrics(
    portfolio_prices: &[f64],
    benchmark_prices: Option<&[f64]>,
    days_per_year: f64,
) -> ReturnMetrics {
    if portfolio_prices.len() < 2 {
        return ReturnMetrics {
            total_return: 0.0,
            cagr: 0.0,
            daily_returns: Vec::new(),
            alpha: 0.0,
            beta: 1.0,
        };
    }

    let daily_returns = calculate_daily_returns(portfolio_prices);
    let total_return = (portfolio_prices.last().unwrap() - portfolio_prices.first().unwrap())
        / portfolio_prices.first().unwrap();

    let years = portfolio_prices.len() as f64 / days_per_year;
    let cagr = if years > 0.0 && portfolio_prices.first().unwrap() > &0.0 {
        (portfolio_prices.last().unwrap() / portfolio_prices.first().unwrap()).powf(1.0 / years)
            - 1.0
    } else {
        total_return
    };

    let (alpha, beta) = if let Some(bm_prices) = benchmark_prices {
        let bm_returns = calculate_daily_returns(bm_prices);
        let min_len = daily_returns.len().min(bm_returns.len());
        if min_len > 2 {
            let p_rets = &daily_returns[..min_len];
            let b_rets = &bm_returns[..min_len];
            let mean_p = p_rets.iter().sum::<f64>() / min_len as f64;
            let mean_b = b_rets.iter().sum::<f64>() / min_len as f64;

            let mut cov = 0.0;
            let mut var_b = 0.0;
            for i in 0..min_len {
                cov += (p_rets[i] - mean_p) * (b_rets[i] - mean_b);
                var_b += (b_rets[i] - mean_b).powi(2);
            }
            let beta = if var_b > 1e-9 { cov / var_b } else { 1.0 };
            let alpha = (mean_p - beta * mean_b) * 252.0;
            (alpha, beta)
        } else {
            (0.0, 1.0)
        }
    } else {
        (0.0, 1.0)
    };

    ReturnMetrics {
        total_return,
        cagr,
        daily_returns,
        alpha,
        beta,
    }
}
