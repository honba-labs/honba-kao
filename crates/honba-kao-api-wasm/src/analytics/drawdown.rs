use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DrawdownSeries {
    pub max_drawdown: f64,
    pub max_drawdown_pct: f64,
    pub peak_index: usize,
    pub trough_index: usize,
    pub recovery_duration_bars: Option<usize>,
    pub underwater_curve: Vec<f64>,
}

pub fn calculate_drawdown(equity_curve: &[f64]) -> DrawdownSeries {
    if equity_curve.is_empty() {
        return DrawdownSeries {
            max_drawdown: 0.0,
            max_drawdown_pct: 0.0,
            peak_index: 0,
            trough_index: 0,
            recovery_duration_bars: None,
            underwater_curve: Vec::new(),
        };
    }

    let mut peak = equity_curve[0];
    let mut peak_idx = 0;
    let mut max_dd = 0.0;
    let mut max_dd_pct = 0.0;
    let mut best_peak_idx = 0;
    let mut best_trough_idx = 0;
    let mut underwater = Vec::with_capacity(equity_curve.len());

    for (i, &equity) in equity_curve.iter().enumerate() {
        if equity > peak {
            peak = equity;
            peak_idx = i;
        }

        let dd = peak - equity;
        let dd_pct = if peak > 0.0 { dd / peak } else { 0.0 };
        underwater.push(-dd_pct);

        if dd > max_dd {
            max_dd = dd;
            max_dd_pct = dd_pct;
            best_peak_idx = peak_idx;
            best_trough_idx = i;
        }
    }

    DrawdownSeries {
        max_drawdown: max_dd,
        max_drawdown_pct: max_dd_pct,
        peak_index: best_peak_idx,
        trough_index: best_trough_idx,
        recovery_duration_bars: None,
        underwater_curve: underwater,
    }
}
