//! Pure indicator-series compute, free of any wasm-bindgen types.
//!
//! Everything here runs natively, so it is unit- and conformance-tested without a wasm runtime.
//! The `#[wasm_bindgen]` layer in [`crate`] only forwards to these functions.
//!
//! The indicators are the streaming implementations from `honba-indicators`, driven over a whole
//! series: `out[i]` is the indicator after consuming `closes[0..=i]`, and is `NaN` while the
//! indicator is warming up. The output always has the same length as the input.

use std::fmt;

use honba_indicators::{
    Atr, BollingerBands, Ema, Indicator, Macd, Rsi, Sma, Stochastic, WilliamsR,
};
use serde::Deserialize;
use serde_json::{Value, json};

/// Largest accepted period; bounds the memory an untrusted `params_json` can request.
pub const MAX_PERIOD: usize = 1_000_000;

/// Why an indicator request was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndicatorError {
    /// The indicator name is not in the catalog.
    UnknownIndicator(String),
    /// `params_json` is malformed, incomplete or out of range.
    InvalidParams(String),
    /// The input contains a `NaN` or infinity at this index.
    NonFiniteInput(usize),
    /// Finite input overflowed: the output is infinite, or `NaN` after warm-up, at this index.
    NonFiniteOutput(usize),
    /// `high` is below `low` at this index.
    InvertedRange(usize),
    /// The `high`, `low` and `close` series do not have the same length.
    LengthMismatch {
        /// Length of `high`.
        high: usize,
        /// Length of `low`.
        low: usize,
        /// Length of `close`.
        close: usize,
    },
}

impl IndicatorError {
    /// A stable machine-readable reason in a `reason: message` pair (ADR 0012 rule 4):
    /// `unknown_indicator`, `invalid_params`, `non_finite_input`, `non_finite_output`,
    /// `inverted_range` or `length_mismatch`.
    #[must_use]
    pub const fn reason(&self) -> &'static str {
        match self {
            Self::UnknownIndicator(_) => "unknown_indicator",
            Self::InvalidParams(_) => "invalid_params",
            Self::NonFiniteInput(_) => "non_finite_input",
            Self::NonFiniteOutput(_) => "non_finite_output",
            Self::InvertedRange(_) => "inverted_range",
            Self::LengthMismatch { .. } => "length_mismatch",
        }
    }
}

impl fmt::Display for IndicatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownIndicator(n) => write!(f, "unknown indicator `{n}`"),
            Self::InvalidParams(m) => write!(f, "invalid params: {m}"),
            Self::NonFiniteInput(i) => write!(f, "non-finite input at index {i}"),
            Self::NonFiniteOutput(i) => {
                write!(f, "numeric overflow: non-finite output at index {i}")
            }
            Self::InvertedRange(i) => write!(f, "high is below low at index {i}"),
            Self::LengthMismatch { high, low, close } => write!(
                f,
                "series lengths differ: high={high}, low={low}, close={close}"
            ),
        }
    }
}

impl std::error::Error for IndicatorError {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PeriodParams {
    period: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StochasticParams {
    period: usize,
    #[serde(default = "default_d_period")]
    d_period: usize,
    #[serde(default = "default_k_output")]
    output: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BollingerParams {
    period: usize,
    #[serde(default = "default_k")]
    k: f64,
    #[serde(default = "default_middle")]
    output: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MacdParams {
    #[serde(default = "default_fast")]
    fast: usize,
    #[serde(default = "default_slow")]
    slow: usize,
    #[serde(default = "default_signal")]
    signal: usize,
    #[serde(default = "default_macd")]
    output: String,
}

fn default_k() -> f64 {
    2.0
}
fn default_middle() -> String {
    "middle".into()
}
fn default_d_period() -> usize {
    3
}
fn default_k_output() -> String {
    "k".into()
}
fn default_fast() -> usize {
    12
}
fn default_slow() -> usize {
    26
}
fn default_signal() -> usize {
    9
}
fn default_macd() -> String {
    "macd".into()
}

fn parse<'a, T: Deserialize<'a>>(params_json: &'a str) -> Result<T, IndicatorError> {
    let text = if params_json.trim().is_empty() {
        "{}"
    } else {
        params_json
    };
    serde_json::from_str(text).map_err(|e| IndicatorError::InvalidParams(e.to_string()))
}

fn period(name: &str, v: usize) -> Result<usize, IndicatorError> {
    if v == 0 || v > MAX_PERIOD {
        return Err(IndicatorError::InvalidParams(format!(
            "`{name}` must be in 1..={MAX_PERIOD}, got {v}"
        )));
    }
    Ok(v)
}

fn pick(output: &str, allowed: &[&str]) -> Result<usize, IndicatorError> {
    allowed.iter().position(|a| *a == output).ok_or_else(|| {
        IndicatorError::InvalidParams(format!(
            "`output` must be one of {allowed:?}, got {output:?}"
        ))
    })
}

fn run<I>(mut ind: I, closes: &[f64], f: impl Fn(I::Output) -> f64) -> Vec<f64>
where
    I: for<'a> Indicator<Input<'a> = f64>,
{
    closes
        .iter()
        .map(|&c| ind.update(c).map_or(f64::NAN, &f))
        .collect()
}

/// Leading NaNs are warm-up. An infinity, or a NaN after the first value, means finite input
/// overflowed f64 (e.g. a running sum of 1e308 values); refuse rather than return garbage.
fn check_output(out: &[f64]) -> Result<(), IndicatorError> {
    let first = out.iter().position(|v| !v.is_nan()).unwrap_or(out.len());
    match out
        .iter()
        .enumerate()
        .position(|(i, v)| v.is_infinite() || (i > first && v.is_nan()))
    {
        Some(i) => Err(IndicatorError::NonFiniteOutput(i)),
        None => Ok(()),
    }
}

/// Which band a [`CloseIndicator::Bollinger`] series reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BollingerOutput {
    /// The moving average.
    Middle,
    /// `middle + k * stddev`.
    Upper,
    /// `middle - k * stddev`.
    Lower,
}

/// Which line a [`CloseIndicator::Macd`] series reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MacdOutput {
    /// `ema(fast) - ema(slow)`.
    Macd,
    /// The signal EMA of the MACD line.
    Signal,
    /// `macd - signal`.
    Histogram,
}

/// Which line an [`OhlcIndicator::Stochastic`] series reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StochasticOutput {
    /// `%K`.
    K,
    /// `%D`, the SMA of `%K` over `d_period`.
    D,
}

/// A close-only indicator request, typed: the Rust-facing form of `name` + `params_json`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CloseIndicator {
    /// Simple moving average.
    Sma {
        /// Window length, `1..=MAX_PERIOD`.
        period: usize,
    },
    /// Exponential moving average, seeded with the SMA.
    Ema {
        /// Window length, `1..=MAX_PERIOD`.
        period: usize,
    },
    /// Wilder RSI.
    Rsi {
        /// Window length, `1..=MAX_PERIOD`.
        period: usize,
    },
    /// Bollinger bands.
    Bollinger {
        /// Window length, `1..=MAX_PERIOD`.
        period: usize,
        /// Band width in standard deviations, finite and `>= 0`.
        k: f64,
        /// The band to report.
        output: BollingerOutput,
    },
    /// MACD.
    Macd {
        /// Fast EMA length, below `slow`.
        fast: usize,
        /// Slow EMA length.
        slow: usize,
        /// Signal EMA length.
        signal: usize,
        /// The line to report.
        output: MacdOutput,
    },
}

/// An OHLC-input indicator request, typed: the Rust-facing form of `name` + `params_json`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OhlcIndicator {
    /// Wilder ATR.
    Atr {
        /// Window length, `1..=MAX_PERIOD`.
        period: usize,
    },
    /// Williams %R.
    WilliamsR {
        /// Window length, `1..=MAX_PERIOD`.
        period: usize,
    },
    /// Stochastic oscillator.
    Stochastic {
        /// `%K` window length, `1..=MAX_PERIOD`.
        period: usize,
        /// `%D` smoothing length, `1..=MAX_PERIOD`.
        d_period: usize,
        /// The line to report.
        output: StochasticOutput,
    },
}

impl CloseIndicator {
    /// Parses the wire form: indicator `name` plus a `params_json` object (blank means `{}`).
    ///
    /// Unknown fields are rejected and every value is range-checked, in the same order
    /// [`indicator_series`] has always reported them.
    ///
    /// # Errors
    ///
    /// [`IndicatorError::UnknownIndicator`] or [`IndicatorError::InvalidParams`].
    pub fn from_json(name: &str, params_json: &str) -> Result<Self, IndicatorError> {
        Ok(match name {
            "sma" => Self::Sma {
                period: period("period", parse::<PeriodParams>(params_json)?.period)?,
            },
            "ema" => Self::Ema {
                period: period("period", parse::<PeriodParams>(params_json)?.period)?,
            },
            "rsi" => Self::Rsi {
                period: period("period", parse::<PeriodParams>(params_json)?.period)?,
            },
            "bollinger" => {
                let p: BollingerParams = parse(params_json)?;
                bollinger_k(p.k)?;
                let output = [
                    BollingerOutput::Middle,
                    BollingerOutput::Upper,
                    BollingerOutput::Lower,
                ][pick(&p.output, &["middle", "upper", "lower"])?];
                Self::Bollinger {
                    period: period("period", p.period)?,
                    k: p.k,
                    output,
                }
            }
            "macd" => {
                let p: MacdParams = parse(params_json)?;
                let (fast, slow, signal) = macd_periods(p.fast, p.slow, p.signal)?;
                let output = [MacdOutput::Macd, MacdOutput::Signal, MacdOutput::Histogram]
                    [pick(&p.output, &["macd", "signal", "histogram"])?];
                Self::Macd {
                    fast,
                    slow,
                    signal,
                    output,
                }
            }
            other => return Err(IndicatorError::UnknownIndicator(other.to_string())),
        })
    }

    /// Range-checks the request exactly as [`Self::from_json`] does.
    fn validate(&self) -> Result<(), IndicatorError> {
        match *self {
            Self::Sma { period: n } | Self::Ema { period: n } | Self::Rsi { period: n } => {
                period("period", n)?;
            }
            Self::Bollinger { period: n, k, .. } => {
                bollinger_k(k)?;
                period("period", n)?;
            }
            Self::Macd {
                fast, slow, signal, ..
            } => {
                macd_periods(fast, slow, signal)?;
            }
        }
        Ok(())
    }
}

impl OhlcIndicator {
    /// Parses the wire form: indicator `name` plus a `params_json` object (blank means `{}`).
    ///
    /// # Errors
    ///
    /// [`IndicatorError::UnknownIndicator`] (including a close-only name) or
    /// [`IndicatorError::InvalidParams`].
    pub fn from_json(name: &str, params_json: &str) -> Result<Self, IndicatorError> {
        Ok(match name {
            "atr" => Self::Atr {
                period: period("period", parse::<PeriodParams>(params_json)?.period)?,
            },
            "williams_r" => Self::WilliamsR {
                period: period("period", parse::<PeriodParams>(params_json)?.period)?,
            },
            "stochastic" => {
                let p: StochasticParams = parse(params_json)?;
                let output =
                    [StochasticOutput::K, StochasticOutput::D][pick(&p.output, &["k", "d"])?];
                Self::Stochastic {
                    period: period("period", p.period)?,
                    d_period: period("d_period", p.d_period)?,
                    output,
                }
            }
            other => return Err(IndicatorError::UnknownIndicator(other.to_string())),
        })
    }

    /// Range-checks the request exactly as [`Self::from_json`] does.
    fn validate(&self) -> Result<(), IndicatorError> {
        match *self {
            Self::Atr { period: n } | Self::WilliamsR { period: n } => {
                period("period", n)?;
            }
            Self::Stochastic {
                period: n,
                d_period,
                ..
            } => {
                period("period", n)?;
                period("d_period", d_period)?;
            }
        }
        Ok(())
    }
}

fn bollinger_k(k: f64) -> Result<(), IndicatorError> {
    if !k.is_finite() || k < 0.0 {
        return Err(IndicatorError::InvalidParams(format!(
            "`k` must be finite and >= 0, got {k}"
        )));
    }
    Ok(())
}

fn macd_periods(
    fast: usize,
    slow: usize,
    signal: usize,
) -> Result<(usize, usize, usize), IndicatorError> {
    let (fast, slow, signal) = (
        period("fast", fast)?,
        period("slow", slow)?,
        period("signal", signal)?,
    );
    if fast >= slow {
        return Err(IndicatorError::InvalidParams(format!(
            "`fast` ({fast}) must be less than `slow` ({slow})"
        )));
    }
    Ok((fast, slow, signal))
}

/// Computes the full series of a typed close-only indicator over `closes`.
///
/// The request is validated before the data is inspected, so a bad request fails the same way
/// on any input.
///
/// # Errors
///
/// [`IndicatorError`] for bad params, non-finite input, or finite input whose result overflows
/// `f64` (infinite output, or `NaN` after warm-up).
pub fn close_series(indicator: &CloseIndicator, closes: &[f64]) -> Result<Vec<f64>, IndicatorError> {
    indicator.validate()?;
    if let Some(i) = closes.iter().position(|c| !c.is_finite()) {
        return Err(IndicatorError::NonFiniteInput(i));
    }
    let out = match *indicator {
        CloseIndicator::Sma { period: n } => run(Sma::new(n), closes, |v| v),
        CloseIndicator::Ema { period: n } => run(Ema::new(n), closes, |v| v),
        CloseIndicator::Rsi { period: n } => run(Rsi::new(n), closes, |v| v),
        CloseIndicator::Bollinger {
            period: n,
            k,
            output,
        } => run(BollingerBands::new(n, k), closes, |v| match output {
            BollingerOutput::Middle => v.middle,
            BollingerOutput::Upper => v.upper,
            BollingerOutput::Lower => v.lower,
        }),
        CloseIndicator::Macd {
            fast,
            slow,
            signal,
            output,
        } => run(Macd::new(fast, slow, signal), closes, |v| match output {
            MacdOutput::Macd => v.macd,
            MacdOutput::Signal => v.signal,
            MacdOutput::Histogram => v.histogram,
        }),
    };
    check_output(&out)?;
    Ok(out)
}

/// Computes the full series of indicator `name` over `closes`: the wire form of
/// [`close_series`].
///
/// `params_json` is a JSON object (see [`list_indicators_json`]); blank means `{}`. Unknown
/// fields are rejected. Multi-output indicators (`macd`, `bollinger`) select one output with the
/// `output` param.
///
/// # Errors
///
/// [`IndicatorError`] for an unknown name, bad params, non-finite input, or finite input whose
/// result overflows `f64` (infinite output, or `NaN` after warm-up).
pub fn indicator_series(
    name: &str,
    params_json: &str,
    closes: &[f64],
) -> Result<Vec<f64>, IndicatorError> {
    close_series(&CloseIndicator::from_json(name, params_json)?, closes)
}

/// Computes the full series of a typed OHLC-input indicator over equal-length `high`, `low` and
/// `close` series (`atr`: Wilder; `williams_r`; `stochastic`).
///
/// The request is validated first, then lengths, then finiteness; the output has the input's
/// length with `period - 1` leading `NaN`s (the first bar's true range is `high - low`); empty
/// input gives empty output; infinite output from finite input is [`IndicatorError::NonFiniteOutput`].
///
/// # Errors
///
/// [`IndicatorError`] for bad params, unequal lengths, non-finite input, `high < low` at any index
/// ([`IndicatorError::InvertedRange`]), or overflow.
pub fn ohlc_series(
    indicator: &OhlcIndicator,
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, IndicatorError> {
    indicator.validate()?;
    if high.len() != low.len() || high.len() != close.len() {
        return Err(IndicatorError::LengthMismatch {
            high: high.len(),
            low: low.len(),
            close: close.len(),
        });
    }
    for series in [high, low, close] {
        if let Some(i) = series.iter().position(|v| !v.is_finite()) {
            return Err(IndicatorError::NonFiniteInput(i));
        }
    }
    if let Some(i) = high.iter().zip(low).position(|(h, l)| h < l) {
        return Err(IndicatorError::InvertedRange(i));
    }
    let bars = high
        .iter()
        .zip(low)
        .zip(close)
        .map(|((&h, &l), &c)| (h, l, c));
    let out: Vec<f64> = match *indicator {
        OhlcIndicator::Atr { period: n } => {
            let mut ind = Atr::new(n);
            bars.map(|(h, l, c)| ind.update_hlc(h, l, c).unwrap_or(f64::NAN))
                .collect()
        }
        OhlcIndicator::WilliamsR { period: n } => {
            let mut ind = WilliamsR::new(n);
            bars.map(|(h, l, c)| ind.update_hlc(h, l, c).unwrap_or(f64::NAN))
                .collect()
        }
        OhlcIndicator::Stochastic {
            period: n,
            d_period,
            output,
        } => {
            let mut ind = Stochastic::new(n, d_period);
            bars.map(|(h, l, c)| {
                ind.update_hlc(h, l, c)
                    .and_then(|v| match output {
                        StochasticOutput::K => Some(v.k),
                        StochasticOutput::D => v.d,
                    })
                    .unwrap_or(f64::NAN)
            })
            .collect()
        }
    };
    check_output(&out)?;
    Ok(out)
}

/// Computes the full series of OHLC-input indicator `name`: the wire form of [`ohlc_series`].
///
/// Same conventions as [`indicator_series`]: params are validated first (unknown fields rejected,
/// `period` in `1..=MAX_PERIOD`, `period` required), then lengths, then finiteness.
///
/// # Errors
///
/// [`IndicatorError`] for an unknown (or close-only) name, bad params, unequal lengths,
/// non-finite input, `high < low` at any index ([`IndicatorError::InvertedRange`]), or overflow.
pub fn ohlc_indicator_series(
    name: &str,
    params_json: &str,
    high: &[f64],
    low: &[f64],
    close: &[f64],
) -> Result<Vec<f64>, IndicatorError> {
    ohlc_series(&OhlcIndicator::from_json(name, params_json)?, high, low, close)
}

/// Describes the available indicators: names, inputs, params, outputs, warm-up and the render
/// contract (E6-S5).
///
/// Shape: `{"warmup_value":"NaN","indicators":[{"name","input","params":[{"name","type",
/// "required","default"?,"min"?}],"outputs":[..],"warmup":"..","presentation":{"pane","plots",
/// "zones"}}]}`. `input` is `close` for [`indicator_series`] indicators and `ohlc` for
/// [`ohlc_indicator_series`] ones (those also carry `"export":"ohlc_indicator_series"`). The
/// `warmup` text is the number of leading `NaN` values. `presentation` is the render contract the
/// chart consumes: the pane, one plot per output and any zones.
#[must_use]
pub fn indicator_catalog() -> Value {
    let period = json!({"name": "period", "type": "integer", "required": true, "min": 1,
        "max": MAX_PERIOD});
    let int = |n: &str, d: usize| {
        json!({"name": n, "type": "integer", "required": false, "default": d, "min": 1,
            "max": MAX_PERIOD})
    };
    let output = |opts: &[&str]| {
        json!({"name": "output", "type": "string", "required": false, "default": opts[0],
            "values": opts})
    };
    // The render contract is owned by `honba-indicators`; serialize it here so the chart has a
    // single source of truth (a missing entry is a programming error, caught by the catalog test).
    let pres = |name: &str| {
        serde_json::to_value(honba_indicators::render::presentation(name).expect("catalog"))
            .expect("presentation serializes")
    };
    json!({
        "warmup_value": "NaN",
        "indicators": [
            {"name": "sma", "input": "close", "params": [period], "outputs": ["value"],
             "warmup": "period - 1", "presentation": pres("sma")},
            {"name": "ema", "input": "close", "params": [period], "outputs": ["value"],
             "warmup": "period - 1", "presentation": pres("ema")},
            {"name": "rsi", "input": "close", "params": [period], "outputs": ["value"],
             "warmup": "period", "presentation": pres("rsi")},
            {"name": "macd", "input": "close",
             "params": [int("fast", 12), int("slow", 26), int("signal", 9),
                        output(&["macd", "signal", "histogram"])],
             "outputs": ["macd", "signal", "histogram"], "warmup": "slow + signal - 2",
             "presentation": pres("macd")},
            {"name": "bollinger", "input": "close",
             "params": [period, {"name": "k", "type": "number", "required": false,
                                 "default": 2.0, "min": 0},
                        output(&["middle", "upper", "lower"])],
             "outputs": ["middle", "upper", "lower"], "warmup": "period - 1",
             "presentation": pres("bollinger")},
            {"name": "atr", "input": "ohlc", "export": "ohlc_indicator_series",
             "params": [period], "outputs": ["value"], "warmup": "period - 1",
             "presentation": pres("atr")},
            {"name": "williams_r", "input": "ohlc", "export": "ohlc_indicator_series",
             "params": [period], "outputs": ["value"], "warmup": "period - 1",
             "presentation": pres("williams_r")},
            {"name": "stochastic", "input": "ohlc", "export": "ohlc_indicator_series",
             "params": [period, int("d_period", 3), output(&["k", "d"])],
             "outputs": ["k", "d"], "warmup": "period - 1 (k); period + d_period - 2 (d)",
             "presentation": pres("stochastic")},
        ],
    })
}

/// [`indicator_catalog`] as a JSON string, for the wasm boundary.
#[must_use]
pub fn list_indicators_json() -> String {
    indicator_catalog().to_string()
}
