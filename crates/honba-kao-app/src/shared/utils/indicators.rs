//! Technical indicators. Mirrors `web/src/utils/indicators.ts`.

/// Simple moving average of the last `period` values.
pub fn sma(values: &[f64], period: usize) -> Option<f64> {
    if period == 0 || values.len() < period {
        return None;
    }
    let window = &values[values.len() - period..];
    Some(window.iter().sum::<f64>() / period as f64)
}

/// Exponential moving average (seeded with an SMA).
pub fn ema(values: &[f64], period: usize) -> Option<f64> {
    if period == 0 || values.len() < period {
        return None;
    }
    let k = 2.0 / (period as f64 + 1.0);
    let mut acc = sma(&values[..period], period)?;
    for v in &values[period..] {
        acc = v * k + acc * (1.0 - k);
    }
    Some(acc)
}

/// Wilder's RSI over `period`.
pub fn rsi(values: &[f64], period: usize) -> Option<f64> {
    if period == 0 || values.len() <= period {
        return None;
    }
    let (mut gain, mut loss) = (0.0, 0.0);
    for w in values.windows(2).take(period) {
        let delta = w[1] - w[0];
        if delta >= 0.0 {
            gain += delta;
        } else {
            loss -= delta;
        }
    }
    let mut avg_gain = gain / period as f64;
    let mut avg_loss = loss / period as f64;
    for w in values.windows(2).skip(period) {
        let delta = w[1] - w[0];
        let (g, l) = if delta >= 0.0 {
            (delta, 0.0)
        } else {
            (0.0, -delta)
        };
        avg_gain = (avg_gain * (period as f64 - 1.0) + g) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + l) / period as f64;
    }
    if avg_loss == 0.0 {
        return Some(100.0);
    }
    let rs = avg_gain / avg_loss;
    Some(100.0 - (100.0 / (1.0 + rs)))
}
