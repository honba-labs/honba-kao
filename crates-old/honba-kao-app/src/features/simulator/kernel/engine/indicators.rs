//! Fast client-side indicator series for the replay engine (P7).
//!
//! The evaluator needs whole arrays per indicator node (`out[i]` is the
//! value after consuming `closes[0..=i]`, `NaN` while warming up — the
//! exact fe `indicator-calculator.ts` contract). Sources, pinned:
//!
//! - `sma`, `rsi`, `bollinger` — the streaming implementations from
//!   `honba-me-indicators` (the crates honba-kao-api-wasm drives), which
//!   numerically match fe's formulas (sliding-window SMA, Wilder-seeded
//!   RSI, population-stddev Bollinger). Tests pin the parity against an
//!   inline fe reference, not just warm-up shapes. One sub-point-of-tick
//!   deviation is pinned: `bollinger` accumulates running sums (variance =
//!   `sum_sq/n − mean²`) while fe recomputes each window naively, so
//!   cancellation can open the gap at price scale — parity is asserted at
//!   1e-9 (order of a picopaisa on a ₹100 band, invisible to any
//!   condition it feeds).
//! - `ema` — fe's own recursion is kept verbatim: fe seeds the EMA with
//!   `bars[0].close` and emits at index 0 (unmasked!) then NaN-masks
//!   `1..period-1`; `honba-me-indicators::Ema` seeds with the window SMA
//!   and emits from `period - 1`. Doc P7's rule is "where a formula diff
//!   exists, match fe's semantics" — so the first-close seed is pinned
//!   here, including fe's asymmetry of a finite value at index 0.
//! - `macd` — builds on the pinned EMA because fe anchors the signal line
//!   at index `slow - 1`, which depends on that seed.
//!
//! Degenerate params (period < 1) are clamped to 1 instead of reproducing
//! fe's `sum / 0` NaN behaviour; the evaluator supplies its defaults first,
//! so this only rejects garbage, never honest input.

use honba_indicators::{BollingerBands, Indicator, Rsi, Sma};

/// Sliding-window SMA, fe `calculateSMA`.
pub fn sma_series(closes: &[f64], period: usize) -> Vec<f64> {
    let mut result = vec![f64::NAN; closes.len()];
    let mut sma = Sma::new(period.max(1));
    for (i, &close) in closes.iter().enumerate() {
        if let Some(value) = sma.update(close) {
            result[i] = value;
        }
    }
    result
}

/// fe-EMA recursion, verbatim (see module doc for the pinned seed).
pub fn ema_series(closes: &[f64], period: usize) -> Vec<f64> {
    let mut result = vec![f64::NAN; closes.len()];
    if closes.is_empty() {
        return result;
    }
    let period = period.max(1);
    let k = 2.0 / (period as f64 + 1.0);
    let mut ema = closes[0];
    result[0] = ema;
    for i in 1..closes.len() {
        ema = closes[i] * k + ema * (1.0 - k);
        result[i] = if i + 1 >= period { ema } else { f64::NAN };
    }
    result
}

/// Wilder RSI over closes, driven by the `honba-me-indicators` stream
/// (simple-average seed + `(avg·(n−1)+x)/n` smoothing, `100` when no
/// losses) — numerically fe's `calculateRSI`.
pub fn rsi_series(closes: &[f64], period: usize) -> Vec<f64> {
    let mut result = vec![f64::NAN; closes.len()];
    let mut rsi = Rsi::new(period.max(1));
    for (i, &close) in closes.iter().enumerate() {
        if let Some(value) = rsi.update(close) {
            result[i] = value;
        }
    }
    result
}

/// fe `MACDResult`.
#[derive(Clone, Debug, PartialEq)]
pub struct MacdSeries {
    /// `macd` line.
    pub macd: Vec<f64>,
    /// `signal` line.
    pub signal: Vec<f64>,
    /// `histogram`.
    pub histogram: Vec<f64>,
}

/// fe `calculateMACD` over the two pinned-EMA lines: the MACD line is the
/// finite-pair difference; the signal line anchors at `slow − 1` and is
/// NaN-masked until `slow + signal − 2`; histogram fills where the signal
/// does.
pub fn macd_from_emas(
    fast_ema: &[f64],
    slow_ema: &[f64],
    slow_period: usize,
    signal_period: usize,
) -> MacdSeries {
    let n = fast_ema.len().min(slow_ema.len());
    let mut macd = vec![f64::NAN; n];
    for i in 0..n {
        if fast_ema[i].is_finite() && slow_ema[i].is_finite() {
            macd[i] = fast_ema[i] - slow_ema[i];
        }
    }
    let mut signal = vec![f64::NAN; n];
    let mut histogram = vec![f64::NAN; n];
    // fe: `Number(params.signal ?? 9)` reaches this as ≥ 1 via the
    // evaluator defaults; `signal_period.min(…)` guards degeneracy.
    let signal_period = signal_period.max(1);
    let valid_index = slow_period.max(1) - 1;
    if n > valid_index + signal_period {
        let k = 2.0 / (signal_period as f64 + 1.0);
        let mut sig = macd[valid_index];
        signal[valid_index] = sig;
        for i in valid_index + 1..n {
            if macd[i].is_finite() {
                sig = macd[i] * k + sig * (1.0 - k);
                signal[i] = if i + 1 >= valid_index + signal_period {
                    sig
                } else {
                    f64::NAN
                };
                if signal[i].is_finite() {
                    histogram[i] = macd[i] - signal[i];
                }
            }
        }
    }
    MacdSeries {
        macd,
        signal,
        histogram,
    }
}

/// fe `BollingerResult`.
#[derive(Clone, Debug, PartialEq)]
pub struct BollingerSeries {
    /// `upper` band.
    pub upper: Vec<f64>,
    /// `middle` band (the SMA).
    pub middle: Vec<f64>,
    /// `lower` band.
    pub lower: Vec<f64>,
}

/// Bollinger bands driven by the `honba-me-indicators` stream (population
/// stddev — fe's convention too). A negative multiplier clamps to 0 where
/// fe would silently mirror the bands; documented validation, not a
/// formula change.
pub fn bollinger_series(closes: &[f64], period: usize, std_mult: f64) -> BollingerSeries {
    let n = closes.len();
    let mut series = BollingerSeries {
        upper: vec![f64::NAN; n],
        middle: vec![f64::NAN; n],
        lower: vec![f64::NAN; n],
    };
    if n == 0 {
        return series;
    }
    let std_mult = if std_mult.is_finite() && std_mult >= 0.0 {
        std_mult
    } else {
        0.0
    };
    let mut bands = BollingerBands::new(period.max(1), std_mult);
    for (i, &close) in closes.iter().enumerate() {
        if let Some(value) = bands.update(close) {
            series.upper[i] = value.upper;
            series.middle[i] = value.middle;
            series.lower[i] = value.lower;
        }
    }
    series
}

#[cfg(test)]
mod tests {
    use super::*;

    /// fe `engine.test.ts` `mockBars` closes (same sine generator), as f64.
    fn sine_closes(count: usize) -> Vec<f64> {
        let mut closes = Vec::with_capacity(count);
        let mut price = 100.0;
        for i in 0..count {
            let delta = (i as f64 / 3.0).sin() * 2.0 + if i.is_multiple_of(2) { 1.0 } else { -0.5 };
            price += delta;
            closes.push(price);
        }
        closes
    }

    /// Inline fe reference for RSI, used to pin the stream's parity.
    fn fe_rsi(closes: &[f64], period: usize) -> Vec<f64> {
        let mut result = vec![f64::NAN; closes.len()];
        if closes.len() <= period {
            return result;
        }
        let mut gains = 0.0;
        let mut losses = 0.0;
        for i in 1..=period {
            let change = closes[i] - closes[i - 1];
            if change >= 0.0 {
                gains += change;
            } else {
                losses += change.abs();
            }
        }
        let mut avg_gain = gains / period as f64;
        let mut avg_loss = losses / period as f64;
        result[period] = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
        };
        for i in period + 1..closes.len() {
            let change = closes[i] - closes[i - 1];
            let gain = if change >= 0.0 { change } else { 0.0 };
            let loss = if change < 0.0 { change.abs() } else { 0.0 };
            avg_gain = (avg_gain * (period as f64 - 1.0) + gain) / period as f64;
            avg_loss = (avg_loss * (period as f64 - 1.0) + loss) / period as f64;
            result[i] = if avg_loss == 0.0 {
                100.0
            } else {
                100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
            };
        }
        result
    }

    #[test]
    fn sma_warms_up_then_slides_like_fe() {
        let out = sma_series(&[100.0, 102.0, 101.0, 103.0], 2);
        assert!(out[0].is_nan());
        assert_eq!(out[1], 101.0);
        assert_eq!(out[2], 101.5);
        assert_eq!(out[3], 102.0);
        // The honba stream numerically equals fe's rolling formula on the
        // whole sine fixture (rolling sum, subtract-oldest, bit-exact).
        let closes = sine_closes(30);
        let stream = sma_series(&closes, 5);
        let mut sum = 0.0;
        for (i, &close) in closes.iter().enumerate() {
            sum += close;
            if i >= 5 {
                sum -= closes[i - 5];
            }
            if i + 1 >= 5 {
                assert_eq!(stream[i], sum / 5.0, "sma mismatch at {i}");
            } else {
                assert!(stream[i].is_nan(), "sma warmup at {i}");
            }
        }
    }

    #[test]
    fn rsi_matches_fes_formula_across_the_whole_fixture() {
        let closes = sine_closes(25);
        let stream = rsi_series(&closes, 14);
        let reference = fe_rsi(&closes, 14);
        for i in 0..closes.len() {
            if i + 1 < 14 {
                assert!(stream[i].is_nan(), "warmup must stay NaN at {i}");
            }
            if reference[i].is_nan() {
                assert!(stream[i].is_nan(), "NaN parity broken at {i}");
            } else {
                assert_eq!(stream[i], reference[i], "rsi mismatch at {i}");
            }
        }
        // All-up closes clamp to 100, like fe's avgLoss === 0 branch.
        let up = rsi_series(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 3);
        assert_eq!(up[3], 100.0);
        assert_eq!(up[5], 100.0);
    }

    #[test]
    fn ema_keeps_fes_first_close_seed_and_mask_quirk() {
        let out = ema_series(&[100.0, 102.0, 101.0, 103.0], 3);
        // fe quirk, pinned: index 0 is finite (the raw seed), 1 is masked,
        // 2..on carry the recursion (k = 0.5 here).
        assert_eq!(out[0], 100.0);
        assert!(out[1].is_nan());
        assert_eq!(out[2], 101.0);
        assert_eq!(out[3], 102.0);
    }

    #[test]
    fn macd_anchors_the_signal_at_slow_minus_one_like_fe() {
        let closes = sine_closes(12);
        let fast = ema_series(&closes, 3);
        let slow = ema_series(&closes, 5);
        let macd = macd_from_emas(&fast, &slow, 5, 2);
        // MACD line finite from the slow warm-up (index 4).
        assert!(macd.macd[3].is_nan());
        assert!(macd.macd[4].is_finite());
        // Signal seeded at validIndex = 4, masked until index 5
        // (validIndex + signalPeriod - 1).
        assert_eq!(macd.signal[4], macd.macd[4]);
        assert!(macd.signal[5].is_finite());
        assert_eq!(macd.histogram[5], macd.macd[5] - macd.signal[5]);
        assert!(macd.histogram[4].is_nan());
    }

    #[test]
    fn bollinger_is_population_stddev_over_the_sma_window() {
        let out = bollinger_series(&[1.0, 2.0, 3.0, 4.0, 5.0], 5, 2.0);
        assert!(out.middle[3].is_nan());
        assert_eq!(out.middle[4], 3.0);
        assert!((out.upper[4] - (3.0 + 2.0 * 2.0_f64.sqrt())).abs() < 1e-9);
        assert!((out.lower[4] - (3.0 - 2.0 * 2.0_f64.sqrt())).abs() < 1e-9);
        // honba stream vs fe reference across the whole sine fixture.
        let closes = sine_closes(30);
        let stream = bollinger_series(&closes, 10, 2.0);
        for i in 0..closes.len() {
            if i + 1 >= 10 {
                let window = &closes[i - 9..=i];
                let mean = window.iter().sum::<f64>() / 10.0;
                let variance = window.iter().map(|c| (c - mean) * (c - mean)).sum::<f64>() / 10.0;
                let std = variance.sqrt();
                // The honba stream accumulates running sums (and derives
                // variance as sum_sq/n − mean²), fe recomputes each window
                // naively — identical math, different summation order, so
                // cancellation can widen the gap at price scale; parity is
                // pinned at 1e-9 (≈ a picopaisa on a ₹100 band — far below
                // anything the confluence condition can read differently).
                assert!((stream.middle[i] - mean).abs() < 1e-9, "middle at {i}");
                assert!(
                    (stream.upper[i] - (mean + 2.0 * std)).abs() < 1e-9,
                    "upper at {i}"
                );
                assert!(
                    (stream.lower[i] - (mean - 2.0 * std)).abs() < 1e-9,
                    "lower at {i}"
                );
            } else {
                assert!(stream.upper[i].is_nan());
            }
        }
    }
}
