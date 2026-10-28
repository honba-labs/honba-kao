use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Candle (OHLCV) data, keyed by UNIX timestamp.
///
/// `open`/`high`/`low`/`close`/`volume` are `Decimal` (zero-floats invariant):
/// they feed candle arithmetic that must be exact, not an `f64` approximation.
/// They are converted to `f64` only at the wasm wire boundary — i.e. in this
/// struct's own `Serialize` impl (see `serialize_decimal`/`serialize_decimal_opt`
/// below) — so the JSON shape pushed to `window.honbaCharts` is unchanged
/// (plain numbers).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CandleData {
    pub time: i64,
    #[serde(
        serialize_with = "serialize_decimal",
        deserialize_with = "deserialize_decimal"
    )]
    pub open: Decimal,
    #[serde(
        serialize_with = "serialize_decimal",
        deserialize_with = "deserialize_decimal"
    )]
    pub high: Decimal,
    #[serde(
        serialize_with = "serialize_decimal",
        deserialize_with = "deserialize_decimal"
    )]
    pub low: Decimal,
    #[serde(
        serialize_with = "serialize_decimal",
        deserialize_with = "deserialize_decimal"
    )]
    pub close: Decimal,
    #[serde(
        serialize_with = "serialize_decimal_opt",
        deserialize_with = "deserialize_decimal_opt",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub volume: Option<Decimal>,
}

/// Serialise a `Decimal` as an `f64` JSON number — the wasm wire boundary.
fn serialize_decimal<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_f64(value.to_f64().unwrap_or_default())
}

/// Deserialise a `Decimal` from a JSON number (the inverse of `serialize_decimal`).
fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = f64::deserialize(deserializer)?;
    Decimal::from_f64_retain(value).ok_or_else(|| serde::de::Error::custom("invalid decimal"))
}

/// Serialise an `Option<Decimal>` as an `Option<f64>` JSON number/null.
fn serialize_decimal_opt<S>(value: &Option<Decimal>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(v) => serializer.serialize_some(&v.to_f64().unwrap_or_default()),
        None => serializer.serialize_none(),
    }
}

/// Deserialise an `Option<Decimal>` from an optional JSON number.
fn deserialize_decimal_opt<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<f64>::deserialize(deserializer)?;
    Ok(value.and_then(Decimal::from_f64_retain))
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimeSeriesPoint {
    pub time: i64,
    pub value: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimeSeries {
    pub name: String,
    pub points: Vec<TimeSeriesPoint>,
    pub color: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ViewportState {
    pub from_time: i64,
    pub to_time: i64,
    pub min_price: f64,
    pub max_price: f64,
}
