use super::moving_averages::calculate_sma;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BollingerBands {
    pub upper: Vec<Option<f64>>,
    pub middle: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
}

pub fn calculate_bollinger_bands(prices: &[f64], period: usize, num_std: f64) -> BollingerBands {
    let middle = calculate_sma(prices, period);
    let mut upper = Vec::with_capacity(prices.len());
    let mut lower = Vec::with_capacity(prices.len());

    for (i, mid_opt) in middle.iter().enumerate() {
        if let Some(mid) = *mid_opt {
            let slice = &prices[(i + 1 - period)..=i];
            let variance = slice.iter().map(|p| (p - mid).powi(2)).sum::<f64>() / period as f64;
            let std = variance.sqrt();
            upper.push(Some(mid + num_std * std));
            lower.push(Some(mid - num_std * std));
        } else {
            upper.push(None);
            lower.push(None);
        }
    }

    BollingerBands {
        upper,
        middle,
        lower,
    }
}

pub fn calculate_atr(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    period: usize,
) -> Vec<Option<f64>> {
    let n = highs.len().min(lows.len()).min(closes.len());
    if n < 2 || period == 0 {
        return vec![None; n];
    }

    let mut trs = Vec::with_capacity(n);
    trs.push(highs[0] - lows[0]);

    for i in 1..n {
        let hl = highs[i] - lows[i];
        let hc = (highs[i] - closes[i - 1]).abs();
        let lc = (lows[i] - closes[i - 1]).abs();
        trs.push(hl.max(hc).max(lc));
    }

    calculate_sma(&trs, period)
}
