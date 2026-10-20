//! Indicator node catalogue (fe `core/simulator/catalogue.ts`).
//!
//! The graph designer names indicator nodes by `kind` (`sma`, `rsi`, …) and
//! the engine knows each kind's inputs/outputs/parameters. This port keeps
//! the same table fe does — currently the nine kinds the simulator's
//! built-in graphs actually use — so [`crate::sim::chart_series`] labels
//! specs in the catalogue's parameter order and future controls (= P8) can
//! generate parameter editors from the same source. fe derives `summary`
//! and the numeric ranges from the default value (`rangeFor`); both rules
//! are ported verbatim.

use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

/// The default value of one indicator parameter (fe's typed `defaultValue`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParamDefault {
    /// Integer-typed number (`typeName: 'int'`).
    Int(i64),
    /// Non-integer-typed number (`typeName: 'float'`).
    Float(f64),
    /// Any other default (`typeName: 'str'`).
    Str(String),
}

impl ParamDefault {
    /// fe's `typeName` derivation: `typeof number` → `int`/`float`, else `str`.
    pub const fn type_name(&self) -> &'static str {
        match self {
            ParamDefault::Int(_) => "int",
            ParamDefault::Float(_) => "float",
            ParamDefault::Str(_) => "str",
        }
    }

    /// fe's `rangeFor`: an editor range derived from the default value.
    ///
    /// - Strings and non-positive integers: unconstrained.
    /// - Positive integers `[1, max(d×3, d+1)]` step 1.
    /// - Other numbers `[0, max(0, d×3)]` step `d/10` (rounded like fe,
    ///   falling to 1.0 when the rounding yields zero).
    pub fn range(&self) -> (Option<f64>, Option<f64>, Option<f64>) {
        match *self {
            ParamDefault::Int(n) if n <= 0 => (None, None, None),
            ParamDefault::Int(n) => {
                (Some(1.0), Some((n as f64 * 3.0).max(n as f64 + 1.0)), Some(1.0))
            }
            ParamDefault::Float(d) if d <= 0.0 => (Some(0.0), Some(1.0), Some(1.0)),
            ParamDefault::Float(d) => {
                let step = (d / 10.0 * 1.0e6).round() / 1.0e6;
                let step = if step > 0.0 { step } else { 1.0 };
                (Some(0.0), Some(d * 3.0), Some(step))
            }
            ParamDefault::Str(_) => (None, None, None),
        }
    }
}

/// One tunable parameter of an indicator kind (fe `IndicatorParam`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorParam {
    pub name: String,
    pub default_value: ParamDefault,
    pub type_name: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
}

/// One indicator kind as the graph catalogue exposes it (fe `IndicatorEntry`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorEntry {
    pub kind: String,
    pub family: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub params: Vec<IndicatorParam>,
    pub summary: String,
}

/// fe's `FAMILIES` — the designer's node palette buckets.
pub const FAMILIES: &[&str] = &[
    "moving_average",
    "trend",
    "momentum",
    "volatility",
    "volume",
    "support_resistance",
    "breadth",
    "statistical",
];

/// fe's `RAW` table row (`kind → family/inputs/outputs/params`).
type RawParams = &'static [(&'static str, ParamDefault)];
type RawSpec = (
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
    RawParams,
);

const RAW: &[RawSpec] = &[
    ("sma", "moving_average", &["close"], &["value"], &[("period", ParamDefault::Int(5))]),
    ("ema", "moving_average", &["close"], &["value"], &[("period", ParamDefault::Int(5))]),
    ("rma", "moving_average", &["close"], &["value"], &[("period", ParamDefault::Int(14))]),
    ("wma", "moving_average", &["close"], &["value"], &[("period", ParamDefault::Int(5))]),
    ("rsi", "momentum", &["close"], &["value"], &[("period", ParamDefault::Int(14))]),
    ("atr", "volatility", &["high", "low"], &["value"], &[("period", ParamDefault::Int(14))]),
    (
        "macd",
        "trend",
        &["close"],
        &["macd", "signal", "histogram"],
        &[
            ("fast", ParamDefault::Int(12)),
            ("slow", ParamDefault::Int(26)),
            ("signal", ParamDefault::Int(9)),
        ],
    ),
    (
        "bollinger",
        "volatility",
        &["close"],
        &["upper", "middle", "lower"],
        &[("length", ParamDefault::Int(20)), ("std", ParamDefault::Float(2.0))],
    ),
    (
        "donchian",
        "volatility",
        &["high", "low"],
        &["upper", "lower"],
        &[("period", ParamDefault::Int(20))],
    ),
];

static CATALOGUE: LazyLock<Vec<IndicatorEntry>> = LazyLock::new(|| {
    RAW.iter()
        .map(|&(kind, family, inputs, outputs, params)| IndicatorEntry {
            kind: kind.to_string(),
            family: family.to_string(),
            inputs: inputs.iter().map(|s| (*s).to_string()).collect(),
            outputs: outputs.iter().map(|s| (*s).to_string()).collect(),
            params: params
                .iter()
                .map(|(name, default)| {
                    let (min, max, step) = default.range();
                    IndicatorParam {
                        name: (*name).to_string(),
                        default_value: default.clone(),
                        type_name: default.type_name().to_string(),
                        min,
                        max,
                        step,
                    }
                })
                .collect(),
            summary: format!("{kind} ({family})"),
        })
        .collect()
});

/// The full catalogue, built once (fe `catalogue()`).
pub fn catalogue() -> &'static [IndicatorEntry] {
    &CATALOGUE
}

/// Look up a single indicator entry by its `kind` (e.g. `sma`, `rsi`) —
/// fe `cataloguelookup` (`catalogueLookup` after the solver's rename).
pub fn catalogue_lookup(kind: &str) -> Option<&'static IndicatorEntry> {
    catalogue().iter().find(|entry| entry.kind == kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_reads_the_raw_table_verbatim() {
        let sma = catalogue_lookup("sma").expect("sma is in the table");
        assert_eq!(sma.family, "moving_average");
        assert_eq!(sma.inputs, ["close"]);
        assert_eq!(sma.params.len(), 1);
        assert_eq!(sma.params[0].name, "period");
        assert_eq!(sma.params[0].default_value, ParamDefault::Int(5));
        assert_eq!(sma.params[0].type_name, "int");
        assert_eq!(sma.summary, "sma (moving_average)");
        assert!(catalogue_lookup("not_a_real_indicator").is_none());
    }

    #[test]
    fn macd_keeps_the_parameter_order_the_graph_edges_read() {
        let macd = catalogue_lookup("macd").expect("macd is in the table");
        let names: Vec<&str> = macd.params.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["fast", "slow", "signal"]);
        assert_eq!(macd.outputs, ["macd", "signal", "histogram"]);
    }

    #[test]
    fn ranges_follow_the_fe_derivation() {
        let boll = catalogue_lookup("bollinger").expect("bollinger is in the table");
        let length = boll.params.iter().find(|p| p.name == "length").unwrap();
        assert_eq!(length.default_value.range(), (Some(1.0), Some(60.0), Some(1.0)));
        let std = boll.params.iter().find(|p| p.name == "std").unwrap();
        assert_eq!(std.type_name, "float");
        // `2.0 / 10 → 0.2` exactly divisible, keeps 6 decimals.
        assert_eq!(std.default_value.range(), (Some(0.0), Some(6.0), Some(0.2)));
    }
}
