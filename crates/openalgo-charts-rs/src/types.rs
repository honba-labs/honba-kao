//! Core data structures, series definitions, themes, and configuration options.

use serde::{Deserialize, Serialize};

/// One OHLCV bar. `time` is UTC seconds, matching openalgo-charts' `Bar`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bar {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    #[serde(default)]
    pub volume: f64,
}

impl Bar {
    pub const fn new(time: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
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

/// A single timestamped value for line, area, and histogram series.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LineData {
    pub time: i64,
    pub value: f64,
}

impl LineData {
    pub const fn new(time: i64, value: f64) -> Self {
        Self { time, value }
    }
}

/// Series renderer kinds supported by openalgo-charts.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SeriesKind {
    #[default]
    Candlestick,
    Line,
    Area,
    Bar,
    Histogram,
    Baseline,
    HollowCandle,
    VolumeCandle,
    HighLow,
    HlcArea,
    HeikinAshi,
    Renko,
    Kagi,
    PointFigure,
}

impl SeriesKind {
    /// The string identifier expected by `createChart.addSeries(kind)`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Candlestick => "candlestick",
            Self::Line => "line",
            Self::Area => "area",
            Self::Bar => "bar",
            Self::Histogram => "histogram",
            Self::Baseline => "baseline",
            Self::HollowCandle => "hollow-candle",
            Self::VolumeCandle => "volume-candle",
            Self::HighLow => "high-low",
            Self::HlcArea => "hlc-area",
            Self::HeikinAshi => "heikin-ashi",
            Self::Renko => "renko",
            Self::Kagi => "kagi",
            Self::PointFigure => "point-figure",
        }
    }

    pub fn from_str_name(name: &str) -> Self {
        match name {
            "candlestick" | "candles" => Self::Candlestick,
            "line" => Self::Line,
            "area" => Self::Area,
            "bar" | "bars" => Self::Bar,
            "histogram" => Self::Histogram,
            "baseline" => Self::Baseline,
            "hollow-candle" => Self::HollowCandle,
            "volume-candle" => Self::VolumeCandle,
            "high-low" => Self::HighLow,
            "hlc-area" => Self::HlcArea,
            "heikin-ashi" => Self::HeikinAshi,
            "renko" => Self::Renko,
            "kagi" => Self::Kagi,
            "point-figure" => Self::PointFigure,
            _ => Self::Candlestick,
        }
    }
}

/// One series to draw, with style metadata and bar rows.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartSeries {
    pub kind: SeriesKind,
    /// Literal hex/rgb color or CSS variable name (e.g. `"--primary-8"`).
    pub color: String,
    pub line_width: f64,
    pub bars: Vec<Bar>,
}

impl ChartSeries {
    pub fn candles(bars: Vec<Bar>) -> Self {
        Self {
            kind: SeriesKind::Candlestick,
            color: String::new(),
            line_width: 0.0,
            bars,
        }
    }

    pub fn line(bars: Vec<Bar>, color: impl Into<String>) -> Self {
        Self {
            kind: SeriesKind::Line,
            color: color.into(),
            line_width: 2.0,
            bars,
        }
    }

    pub fn area(bars: Vec<Bar>) -> Self {
        Self {
            kind: SeriesKind::Area,
            color: String::new(),
            line_width: 2.0,
            bars,
        }
    }

    pub fn histogram(bars: Vec<Bar>, color: impl Into<String>) -> Self {
        Self {
            kind: SeriesKind::Histogram,
            color: color.into(),
            line_width: 1.0,
            bars,
        }
    }
}

/// Stylistic overrides when configuring a series.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub up_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub down_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_up_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_down_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wick_up_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wick_down_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_scale_id: Option<String>,
}

/// Options passed into `createChart(container, options)`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<LayoutOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid: Option<GridOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_scale: Option<TimeScaleOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right_price_scale: Option<PriceScaleOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crosshair: Option<CrosshairOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<ChartTheme>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vert_lines: Option<GridLineOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horz_lines: Option<GridLineOptions>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridLineOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeScaleOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right_offset: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar_spacing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_visible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds_visible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_visible: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceScaleOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_scale: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_top: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_bottom: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_visible: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrosshairOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vert_line: Option<GridLineOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horz_line: Option<GridLineOptions>,
}

/// Complete theme palette expected by openalgo-charts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartTheme {
    pub background: String,
    pub grid: String,
    pub axis_text: String,
    pub axis_line: String,
    pub pane_separator: String,
    pub crosshair: String,

    pub up_color: String,
    pub down_color: String,
    pub wick_up_color: String,
    pub wick_down_color: String,

    pub line_color: String,
    pub area_top_color: String,
    pub area_bottom_color: String,

    pub buy: String,
    pub sell: String,
    pub profit: String,
    pub loss: String,
}

impl Default for ChartTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl ChartTheme {
    pub fn dark() -> Self {
        Self {
            background: "#131722".to_string(),
            grid: "rgba(42, 46, 57, 0.5)".to_string(),
            axis_text: "#787B86".to_string(),
            axis_line: "#2A2E39".to_string(),
            pane_separator: "#2A2E39".to_string(),
            crosshair: "#758696".to_string(),

            up_color: "#089981".to_string(),
            down_color: "#F23645".to_string(),
            wick_up_color: "#089981".to_string(),
            wick_down_color: "#F23645".to_string(),

            line_color: "#2962FF".to_string(),
            area_top_color: "rgba(41, 98, 255, 0.28)".to_string(),
            area_bottom_color: "rgba(41, 98, 255, 0.05)".to_string(),

            buy: "#089981".to_string(),
            sell: "#F23645".to_string(),
            profit: "#089981".to_string(),
            loss: "#F23645".to_string(),
        }
    }

    pub fn light() -> Self {
        Self {
            background: "#FFFFFF".to_string(),
            grid: "#F0F3FA".to_string(),
            axis_text: "#787B86".to_string(),
            axis_line: "#E0E3EB".to_string(),
            pane_separator: "#E0E3EB".to_string(),
            crosshair: "#9598A1".to_string(),

            up_color: "#089981".to_string(),
            down_color: "#F23645".to_string(),
            wick_up_color: "#089981".to_string(),
            wick_down_color: "#F23645".to_string(),

            line_color: "#2962FF".to_string(),
            area_top_color: "rgba(41, 98, 255, 0.2)".to_string(),
            area_bottom_color: "rgba(41, 98, 255, 0.02)".to_string(),

            buy: "#089981".to_string(),
            sell: "#F23645".to_string(),
            profit: "#089981".to_string(),
            loss: "#F23645".to_string(),
        }
    }
}
