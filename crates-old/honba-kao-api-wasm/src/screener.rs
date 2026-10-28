//! Pure screener evaluation, free of any wasm-bindgen types.
//!
//! Everything here runs natively, so it is conformance-tested without a wasm runtime (see
//! `tests/screener_conformance.rs`). Rust callers use the typed [`scan_predicate`] /
//! [`scan_group`] (and [`predicate_matches`] / [`group_matches`]) over parsed filters; the
//! `#[wasm_bindgen]` layer in [`crate`] forwards to the JSON-string wrappers
//! [`scan_predicate_json`] and [`scan_group_json`], which parse, call the typed form and render.
//!
//! The evaluator itself is `honba_indicators::screener` (L3); this module is the thin adaptation
//! from the frontend's column-oriented arrays (`open`/`high`/`low`/`close`/`volume`) to the
//! chronological [`Bar`] history the evaluator reads, plus the JSON encoding of the outcome and
//! the translation from the wire's flat predicate/group JSON (`{"key","op","value"}`,
//! `{"operator","items"}`) into the evaluator's own typed [`ScreenerFilterPredicate`] /
//! [`ScreenerFilterGroup`].
//!
//! # Wire contract
//!
//! A scan returns the JSON string `{"result": bool, "metrics": {key: number|null}}`. `null` is a
//! metric still in warm-up (the evaluator returns `None`), which is distinct from a missing key.
//!
//! A predicate is `{"key": "<metric>", "op": "<op>", "value": <number>}`; a group is
//! `{"operator": "AND"|"OR", "items": [predicate | group, ...]}`. The evaluator's own
//! [`ScreenerFilterPredicate::value`] is a plain numeric scalar, so operators whose wire form
//! needs a non-numeric operand (`like`/`has` against a string, `between`/`in`/`not_in` against an
//! array, or crossing against another metric's series) are accepted as valid JSON but always
//! evaluate against the operand's bare numeric reading — a known limitation of the evaluator
//! this module adapts, not reimplemented here.
//!
//! Invalid input is refused with a message rather than answered, matching the indicator exports: a
//! malformed filter JSON, unequal series lengths, a non-finite value, an inconsistent OHLC row, or
//! an unsupported/invalid filter all throw a JavaScript `Error` carrying the message.

use std::collections::BTreeMap;
use std::fmt;

use honba_api::wire::ErrorCode;
use honba_core::rust_decimal::Decimal;
use honba_core::{Bar, Exchange, InstrumentId, Price, Quantity};
use honba_indicators::screener::{
    FilterOp, MetricRef, ScreenerError, ScreenerFilterGroup, ScreenerFilterPredicate,
    evaluate_predicate, latest_metrics,
};
use serde::Deserialize;
use serde_json::{Value, json};

/// Why a screener request over column-oriented arrays was refused.
///
/// [`reason`](Self::reason) is the stable machine-readable half of the refusal.
#[derive(Clone, Debug, PartialEq)]
pub enum WasmScreenerError {
    /// A predicate or group JSON document is malformed (unknown fields, bad types, ...).
    InvalidJson {
        /// Which document failed to parse: `"predicate"` or `"group"`.
        what: &'static str,
        /// The underlying parser message.
        message: String,
    },
    /// The five input series do not all have the same length.
    LengthMismatch {
        /// Length of `open`.
        open: usize,
        /// Length of `high`.
        high: usize,
        /// Length of `low`.
        low: usize,
        /// Length of `close`.
        close: usize,
        /// Length of `volume`.
        volume: usize,
    },
    /// A series holds a `NaN` or infinity at this index.
    NonFiniteInput {
        /// Which series: `"open"`, `"high"`, `"low"`, `"close"` or `"volume"`.
        series: &'static str,
        /// Index of the offending value.
        index: usize,
    },
    /// A row cannot be a valid bar (`low > high`, `open`/`close` outside the range, negative
    /// volume).
    InvalidBar {
        /// Index of the offending row.
        index: usize,
        /// What is wrong.
        message: &'static str,
    },
    /// The filter itself cannot be evaluated; the evaluator's own message is reused.
    Screener(ScreenerError),
}

/// The stable `reason` for one [`ScreenerError`] (ADR 0012 rule 4); the evaluator itself has no
/// such method.
fn screener_error_reason(error: &ScreenerError) -> &'static str {
    match error {
        ScreenerError::UnsupportedMetric { .. } => "unsupported_metric",
        ScreenerError::InvalidOperand { .. } => "invalid_operand",
        ScreenerError::InvalidFilter { .. } => "invalid_filter",
    }
}

impl WasmScreenerError {
    /// A stable machine-readable reason in a `reason: message` pair (ADR 0012 rule 4).
    ///
    /// Caller-facing input failures (malformed predicate or group JSON, unequal series lengths,
    /// non-finite input, an inconsistent OHLC row) all reuse the REST
    /// [`ErrorCode::ValidationInvalidRequest`] code, matching `POST /screener/scan`; evaluator
    /// failures reuse the evaluator's own reason (`unsupported_metric`, `invalid_operand`,
    /// `invalid_filter`).
    #[must_use]
    pub fn reason(&self) -> &'static str {
        match self {
            Self::InvalidJson { .. }
            | Self::LengthMismatch { .. }
            | Self::NonFiniteInput { .. }
            | Self::InvalidBar { .. } => ErrorCode::ValidationInvalidRequest.as_str(),
            Self::Screener(error) => screener_error_reason(error),
        }
    }
}

impl fmt::Display for WasmScreenerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson { what, message } => write!(f, "invalid {what}: {message}"),
            Self::LengthMismatch {
                open,
                high,
                low,
                close,
                volume,
            } => write!(
                f,
                "series lengths differ: open={open}, high={high}, low={low}, close={close}, \
                 volume={volume}"
            ),
            Self::NonFiniteInput { series, index } => {
                write!(f, "non-finite {series} input at index {index}")
            }
            Self::InvalidBar { index, message } => {
                write!(f, "invalid bar at index {index}: {message}")
            }
            // The evaluator's own message ("unsupported metric: {key}") doesn't say why a
            // metric is unsupported; made explicit for the caller here.
            Self::Screener(ScreenerError::UnsupportedMetric { key }) => {
                write!(f, "metric {key:?} cannot be computed from bars")
            }
            Self::Screener(e) => write!(f, "{e}"),
        }
    }
}

impl From<ScreenerError> for WasmScreenerError {
    fn from(e: ScreenerError) -> Self {
        Self::Screener(e)
    }
}

/// Renders an error as a `reason: message` pair (ADR 0012 rule 4) for the `Result<_, String>`
/// adaptation fns below.
fn error_pair(error: impl Into<WasmScreenerError>) -> String {
    let error = error.into();
    format!("{}: {error}", error.reason())
}

/// The stable identity of the synthetic bars: instrument `X` on `NSE`.
///
/// The evaluator derives `price_52_week_high`/`price_52_week_low` from a 252-bar window count, so
/// the exact timestamps do not matter; only their relative order (one synthetic day apart) does.
fn synthetic_instrument() -> InstrumentId {
    InstrumentId::new(Exchange::Nse, "X")
}

/// An input value as an exact [`Decimal`], or `0` if it cannot be represented (unreachable for
/// anything [`finite_series`] has already let through).
fn decimal(x: f64) -> Decimal {
    Decimal::try_from(x).unwrap_or(Decimal::ZERO)
}

/// The synthetic bar timestamp at `index`: one UTC day apart, starting at the epoch.
fn bar_timestamp(index: usize) -> honba_core::chrono::DateTime<honba_core::chrono::Utc> {
    honba_core::chrono::DateTime::from_timestamp(index as i64 * 86_400, 0)
        .expect("in-range synthetic timestamp")
}

fn finite_series(name: &'static str, series: &[f64]) -> Result<(), WasmScreenerError> {
    match series.iter().position(|v| !v.is_finite()) {
        Some(index) => Err(WasmScreenerError::NonFiniteInput {
            series: name,
            index,
        }),
        None => Ok(()),
    }
}

/// Builds a chronological bar history from the column-oriented input arrays.
///
/// Rejects unequal lengths, any non-finite value (in `open`, `high`, `low`, `close` order, then
/// `volume`), and rows that violate the `Bar` invariants (`low <= high`, `open`/`close` inside
/// `[low, high]`, non-negative volume). Empty arrays are allowed and give an empty history.
fn bars_from_arrays(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<Vec<Bar>, WasmScreenerError> {
    let lengths = [open.len(), high.len(), low.len(), close.len(), volume.len()];
    if lengths.iter().any(|&n| n != open.len()) {
        return Err(WasmScreenerError::LengthMismatch {
            open: open.len(),
            high: high.len(),
            low: low.len(),
            close: close.len(),
            volume: volume.len(),
        });
    }
    for (name, series) in [
        ("open", open),
        ("high", high),
        ("low", low),
        ("close", close),
        ("volume", volume),
    ] {
        finite_series(name, series)?;
    }
    let instrument = synthetic_instrument();
    let mut bars = Vec::with_capacity(open.len());
    for i in 0..open.len() {
        let (o, h, l, c, v) = (open[i], high[i], low[i], close[i], volume[i]);
        let message = if l > h {
            Some("low is above high")
        } else if o < l || o > h {
            Some("open is outside the high/low range")
        } else if c < l || c > h {
            Some("close is outside the high/low range")
        } else if v < 0.0 {
            Some("volume is negative")
        } else {
            None
        };
        if let Some(message) = message {
            return Err(WasmScreenerError::InvalidBar { index: i, message });
        }
        bars.push(Bar {
            instrument: instrument.clone(),
            open: Price(decimal(o)),
            high: Price(decimal(h)),
            low: Price(decimal(l)),
            close: Price(decimal(c)),
            volume: Quantity(decimal(v)),
            timestamp: bar_timestamp(i),
        });
    }
    Ok(bars)
}

/// The wire spelling of a [`FilterOp`] (lower snake case), the reverse of what the evaluator's
/// own `Deserialize` derive expects (its Rust variant names).
fn parse_filter_op(op: &str) -> Result<FilterOp, WasmScreenerError> {
    Ok(match op {
        "eq" => FilterOp::Eq,
        "ne" => FilterOp::Ne,
        "neq" => FilterOp::Neq,
        "gt" => FilterOp::Gt,
        "gte" => FilterOp::Gte,
        "lt" => FilterOp::Lt,
        "lte" => FilterOp::Lte,
        "between" => FilterOp::Between,
        "in" => FilterOp::In,
        "not_in" => FilterOp::NotIn,
        "like" => FilterOp::Like,
        "has" => FilterOp::Has,
        "crosses_above" => FilterOp::CrossesAbove,
        "crosses_below" => FilterOp::CrossesBelow,
        other => {
            return Err(WasmScreenerError::InvalidJson {
                what: "predicate",
                message: format!("unknown op {other:?}"),
            });
        }
    })
}

/// Ordering operators: the only ones [`operand_value`] requires a genuine JSON number for,
/// mirroring the evaluator's own `operand()` check. `eq`/`ne`/`neq` don't call it at all — a
/// non-numeric operand just compares unequal, never errors.
fn is_ordering_op(op: FilterOp) -> bool {
    matches!(
        op,
        FilterOp::Gt | FilterOp::Gte | FilterOp::Lt | FilterOp::Lte
    )
}

/// A predicate's operand as the evaluator reads it: a bare numeric scalar (see the module doc on
/// the evaluator's non-numeric-operand limitation).
///
/// An ordering operator (`eq`/`ne`/`gt`/`gte`/`lt`/`lte`) needs a genuine JSON number or boolean,
/// matching the evaluator's own refusal message for anything else — a numeral *string* is not a
/// number here any more than it is there. The remaining operators (`between`/`in`/`not_in`/
/// `like`/`has`/`crosses_*`) accept a numeral string as a best-effort scalar reading, since their
/// non-numeric operand shapes (an array, a free-text string, a cross-metric reference) cannot be
/// represented by the evaluator's `value: f64` at all; an object `{"key": "<metric>"}` is reported
/// as that metric being unsupported rather than silently read as `0.0`.
fn operand_value(value: &Value, op: FilterOp) -> Result<f64, WasmScreenerError> {
    match value {
        Value::Number(n) => Ok(n.as_f64().unwrap_or(0.0)),
        Value::Bool(b) => Ok(f64::from(u8::from(*b))),
        Value::String(s) if !is_ordering_op(op) => Ok(s.parse().unwrap_or(0.0)),
        Value::Object(map) => {
            let key = map.get("key").and_then(Value::as_str).unwrap_or("?");
            Err(WasmScreenerError::Screener(
                ScreenerError::UnsupportedMetric {
                    key: key.to_owned(),
                },
            ))
        }
        other => Err(WasmScreenerError::Screener(ScreenerError::InvalidOperand {
            op,
            message: format!("cannot order a number against {other}"),
        })),
    }
}

/// One wire predicate: `{"key", "op", "value"}`. `value` is required: an absent one is a
/// malformed predicate (`validation_invalid_request`), not an operand to evaluate.
#[derive(Deserialize)]
struct WirePredicate {
    key: String,
    op: String,
    value: Value,
}

/// One wire group item: a predicate or a nested group, distinguished structurally (`key`/`op`
/// marks a predicate, `operator`/`items` a group).
#[derive(Deserialize)]
#[serde(untagged)]
enum WireItem {
    Predicate(WirePredicate),
    Group(WireGroup),
}

/// One wire group: `{"operator": "AND"|"OR", "items": [...]}`.
#[derive(Deserialize)]
struct WireGroup {
    operator: String,
    items: Vec<WireItem>,
}

/// Rejects an `sma0` (period `0`) key before it reaches the evaluator: `Sma::new(0)` panics
/// rather than erroring, so this crate's wasm boundary never calls it with one.
fn reject_zero_period_sma(key: &str) -> Result<(), WasmScreenerError> {
    let lower = key.to_ascii_lowercase();
    let is_zero_period = lower
        .strip_prefix("sma")
        .is_some_and(|digits| digits.parse::<u32>() == Ok(0));
    if is_zero_period {
        return Err(WasmScreenerError::Screener(
            ScreenerError::UnsupportedMetric {
                key: key.to_owned(),
            },
        ));
    }
    Ok(())
}

fn translate_predicate(wire: WirePredicate) -> Result<ScreenerFilterPredicate, WasmScreenerError> {
    reject_zero_period_sma(&wire.key)?;
    let op = parse_filter_op(&wire.op)?;
    let value = operand_value(&wire.value, op)?;
    Ok(ScreenerFilterPredicate {
        metric: MetricRef {
            key: wire.key,
            period: None,
        },
        op,
        value,
    })
}

fn translate_item(item: WireItem) -> Result<ScreenerFilterGroup, WasmScreenerError> {
    match item {
        WireItem::Predicate(p) => Ok(ScreenerFilterGroup::Predicate(translate_predicate(p)?)),
        WireItem::Group(g) => translate_group(g),
    }
}

fn translate_group(wire: WireGroup) -> Result<ScreenerFilterGroup, WasmScreenerError> {
    let items = wire
        .items
        .into_iter()
        .map(translate_item)
        .collect::<Result<Vec<_>, _>>()?;
    match wire.operator.as_str() {
        "AND" => Ok(ScreenerFilterGroup::And(items)),
        "OR" => Ok(ScreenerFilterGroup::Or(items)),
        other => Err(WasmScreenerError::InvalidJson {
            what: "group",
            message: format!("operator {other:?} is not AND or OR"),
        }),
    }
}

fn parse_predicate(predicate_json: &str) -> Result<ScreenerFilterPredicate, WasmScreenerError> {
    let wire: WirePredicate =
        serde_json::from_str(predicate_json).map_err(|e| WasmScreenerError::InvalidJson {
            what: "predicate",
            message: e.to_string(),
        })?;
    translate_predicate(wire)
}

fn parse_group(group_json: &str) -> Result<ScreenerFilterGroup, WasmScreenerError> {
    let wire: WireGroup =
        serde_json::from_str(group_json).map_err(|e| WasmScreenerError::InvalidJson {
            what: "group",
            message: e.to_string(),
        })?;
    translate_group(wire)
}

/// The one metric key a predicate reads.
fn predicate_keys(pred: &ScreenerFilterPredicate) -> Vec<String> {
    vec![pred.metric.key.clone()]
}

/// Every metric key a group reads, in first-seen order, de-duplicated. The evaluator has no
/// equivalent of its own.
fn group_metric_keys(group: &ScreenerFilterGroup) -> Vec<String> {
    let mut keys = Vec::new();
    collect_group_keys(group, &mut keys);
    keys
}

fn collect_group_keys(group: &ScreenerFilterGroup, keys: &mut Vec<String>) {
    match group {
        ScreenerFilterGroup::And(groups) | ScreenerFilterGroup::Or(groups) => {
            for g in groups {
                collect_group_keys(g, keys);
            }
        }
        ScreenerFilterGroup::Predicate(pred) => {
            for key in predicate_keys(pred) {
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
        }
    }
}

/// [`evaluate_group`], except an empty `OR` matches everything rather than nothing.
///
/// The evaluator's own `Or(groups) => groups.iter()...any(|x| x)` is vacuously `false` on an
/// empty `groups` (there is no element that matches); the wire contract wants an empty group to
/// mean "no filter", which is vacuously `true` for `OR` just as it already is for `AND`. Mirrors
/// the evaluator's own recursion (including its no-short-circuit error propagation, so an
/// unsupported branch is never hidden behind an already-decided result) so a nested empty `OR`
/// gets the same fix, not just a top-level one.
fn evaluate_group_compat(group: &ScreenerFilterGroup, bars: &[Bar]) -> Result<bool, ScreenerError> {
    match group {
        ScreenerFilterGroup::And(groups) => {
            let mut all_true = true;
            for g in groups {
                if !evaluate_group_compat(g, bars)? {
                    all_true = false;
                }
            }
            Ok(all_true)
        }
        ScreenerFilterGroup::Or(groups) => {
            if groups.is_empty() {
                return Ok(true);
            }
            let mut any_true = false;
            for g in groups {
                if evaluate_group_compat(g, bars)? {
                    any_true = true;
                }
            }
            Ok(any_true)
        }
        ScreenerFilterGroup::Predicate(pred) => evaluate_predicate(pred, bars),
    }
}

fn f64_value(x: f64) -> Value {
    serde_json::Number::from_f64(x).map_or(Value::Null, Value::Number)
}

/// The outcome of a scan: whether the filter matched, and the latest value of every metric it
/// reads (`None` is a metric still in warm-up, distinct from a missing key).
#[derive(Clone, Debug, PartialEq)]
pub struct ScanOutcome {
    /// Whether the filter matched the latest bar.
    pub result: bool,
    /// Latest value per metric key read by the filter.
    pub metrics: BTreeMap<String, Option<f64>>,
}

impl ScanOutcome {
    /// Renders `{"result": bool, "metrics": {key: number|null}}`.
    #[must_use]
    pub fn to_json(&self) -> String {
        let metrics: serde_json::Map<String, Value> = self
            .metrics
            .iter()
            .map(|(key, value)| (key.clone(), value.map_or(Value::Null, f64_value)))
            .collect();
        json!({ "result": self.result, "metrics": Value::Object(metrics) }).to_string()
    }
}

/// Evaluates one typed predicate against a bar history built from the input arrays.
///
/// # Errors
///
/// [`WasmScreenerError`] for unequal lengths, non-finite input, an inconsistent OHLC row, or a
/// [`ScreenerError`] (unknown/unsupported metric, bad operand).
pub fn predicate_matches(
    predicate: &ScreenerFilterPredicate,
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<bool, WasmScreenerError> {
    let bars = bars_from_arrays(open, high, low, close, volume)?;
    Ok(evaluate_predicate(predicate, &bars)?)
}

/// Evaluates one typed filter group (AND/OR, possibly nested) against the input arrays.
///
/// # Errors
///
/// As [`predicate_matches`], plus group shape/size/operator errors.
pub fn group_matches(
    group: &ScreenerFilterGroup,
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<bool, WasmScreenerError> {
    let bars = bars_from_arrays(open, high, low, close, volume)?;
    Ok(evaluate_group_compat(group, &bars)?)
}

/// Evaluates a typed predicate and reports both its `result` and the latest `metrics` it reads.
///
/// # Errors
///
/// As [`predicate_matches`].
pub fn scan_predicate(
    predicate: &ScreenerFilterPredicate,
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<ScanOutcome, WasmScreenerError> {
    let bars = bars_from_arrays(open, high, low, close, volume)?;
    let result = evaluate_predicate(predicate, &bars)?;
    let metrics = latest_metrics(&predicate_keys(predicate), &bars)?;
    Ok(ScanOutcome { result, metrics })
}

/// Evaluates a typed group and reports both its `result` and the latest value of every metric it
/// reads (in first-seen order, without repeats).
///
/// # Errors
///
/// As [`group_matches`].
pub fn scan_group(
    group: &ScreenerFilterGroup,
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<ScanOutcome, WasmScreenerError> {
    let bars = bars_from_arrays(open, high, low, close, volume)?;
    let result = evaluate_group_compat(group, &bars)?;
    let keys = group_metric_keys(group);
    let metrics = latest_metrics(&keys, &bars)?;
    Ok(ScanOutcome { result, metrics })
}

/// Evaluates one predicate JSON document: the wire form of [`predicate_matches`].
///
/// `predicate_json` is `{"key":"close","op":"gt","value":10}`.
///
/// # Errors
///
/// A `reason: message` pair for malformed predicate JSON, unequal lengths, non-finite input, an
/// inconsistent OHLC row, or a [`ScreenerError`] (unknown/unsupported metric, bad operand, bad
/// filter tree). The `reason` is the stable [`WasmScreenerError::reason`] string.
pub fn evaluate_predicate_json(
    predicate_json: &str,
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<bool, String> {
    let pred = parse_predicate(predicate_json).map_err(error_pair)?;
    predicate_matches(&pred, open, high, low, close, volume).map_err(error_pair)
}

/// Evaluates one filter group JSON document: the wire form of [`group_matches`].
///
/// # Errors
///
/// As [`evaluate_predicate_json`], plus group shape/size/operator errors.
pub fn evaluate_group_json(
    group_json: &str,
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<bool, String> {
    let group = parse_group(group_json).map_err(error_pair)?;
    group_matches(&group, open, high, low, close, volume).map_err(error_pair)
}

/// Evaluates a predicate JSON document: the wire form of [`scan_predicate`].
///
/// Returns the JSON string `{"result": bool, "metrics": {key: number|null}}`, where a `null`
/// metric is still in warm-up.
///
/// # Errors
///
/// As [`evaluate_predicate_json`].
pub fn scan_predicate_json(
    predicate_json: &str,
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<String, String> {
    let pred = parse_predicate(predicate_json).map_err(error_pair)?;
    scan_predicate(&pred, open, high, low, close, volume)
        .map(|outcome| outcome.to_json())
        .map_err(error_pair)
}

/// Evaluates a group JSON document: the wire form of [`scan_group`].
///
/// Returns the same JSON shape as [`scan_predicate_json`].
///
/// # Errors
///
/// As [`evaluate_group_json`].
pub fn scan_group_json(
    group_json: &str,
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
) -> Result<String, String> {
    let group = parse_group(group_json).map_err(error_pair)?;
    scan_group(&group, open, high, low, close, volume)
        .map(|outcome| outcome.to_json())
        .map_err(error_pair)
}
