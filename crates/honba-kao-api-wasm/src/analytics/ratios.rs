use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PerformanceRatios {
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub profit_factor: f64,
    pub win_rate: f64,
}

pub fn calculate_performance_ratios(
    returns: &[f64],
    trades_pnl: &[f64],
    max_drawdown_pct: f64,
    annual_risk_free_rate: f64,
) -> PerformanceRatios {
    if returns.is_empty() {
        return PerformanceRatios {
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            calmar_ratio: 0.0,
            profit_factor: 0.0,
            win_rate: 0.0,
        };
    }

    let n = returns.len() as f64;
    let mean_ret = returns.iter().sum::<f64>() / n;
    let daily_rf = annual_risk_free_rate / 252.0;

    let variance = returns.iter().map(|r| (r - mean_ret).powi(2)).sum::<f64>() / n.max(1.0);
    let std_dev = variance.sqrt();

    let downside_variance = returns
        .iter()
        .filter(|&&r| r < daily_rf)
        .map(|r| (r - daily_rf).powi(2))
        .sum::<f64>()
        / n.max(1.0);
    let downside_dev = downside_variance.sqrt();

    let sharpe = if std_dev > 1e-9 {
        ((mean_ret - daily_rf) / std_dev) * (252.0_f64).sqrt()
    } else {
        0.0
    };

    let sortino = if downside_dev > 1e-9 {
        ((mean_ret - daily_rf) / downside_dev) * (252.0_f64).sqrt()
    } else {
        0.0
    };

    let annualized_ret = mean_ret * 252.0;
    let calmar = if max_drawdown_pct > 1e-9 {
        annualized_ret / max_drawdown_pct
    } else {
        0.0
    };

    let mut gross_profit = 0.0;
    let mut gross_loss = 0.0;
    let mut wins = 0;

    for &pnl in trades_pnl {
        if pnl > 0.0 {
            gross_profit += pnl;
            wins += 1;
        } else if pnl < 0.0 {
            gross_loss += pnl.abs();
        }
    }

    let profit_factor = if gross_loss > 1e-9 {
        gross_profit / gross_loss
    } else if gross_profit > 0.0 {
        100.0
    } else {
        0.0
    };

    let win_rate = if !trades_pnl.is_empty() {
        wins as f64 / trades_pnl.len() as f64
    } else {
        0.0
    };

    PerformanceRatios {
        sharpe_ratio: sharpe,
        sortino_ratio: sortino,
        calmar_ratio: calmar,
        profit_factor,
        win_rate,
    }
}
