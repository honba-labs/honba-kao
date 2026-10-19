//! Chart types shared by the demo data and the OpenAlgo chart bridge.

use serde::Serialize;

/// One OHLCV bar. `time` is UTC seconds, matching openalgo-charts' `Bar`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Bar {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Bar {
    pub fn new(time: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Self {
            time,
            open,
            high,
            low,
            close,
            volume,
        }
    }
}

/// Series renderers exposed by openalgo-charts that the app uses.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SeriesKind {
    #[default]
    Candles,
    Line,
    Area,
    Bar,
}

impl SeriesKind {
    /// The `SeriesType` string openalgo-charts expects.
    pub const fn as_str(self) -> &'static str {
        match self {
            SeriesKind::Candles => "candlestick",
            SeriesKind::Line => "line",
            SeriesKind::Area => "area",
            SeriesKind::Bar => "bar",
        }
    }
}
