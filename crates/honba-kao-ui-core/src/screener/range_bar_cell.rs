//! RangeBarCell — 52-week range bar with a classified trend-state label.
//!
//! Wraps `crate::trading::range_bar::HfMiniRangeBar` (track + marker) and
//! appends a trend label derived from the position within the range and the
//! recent change percentage. No track/marker markup is duplicated here.

use leptos::prelude::*;

use crate::trading::range_bar::{HfMiniRangeBar, range_position_pct};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RangeTrend {
    NearHigh,
    NearLow,
    Uptrend,
    Downtrend,
    RangeBound,
}

impl RangeTrend {
    pub fn as_label(&self) -> &'static str {
        match self {
            RangeTrend::NearHigh => "near high",
            RangeTrend::NearLow => "near low",
            RangeTrend::Uptrend => "uptrend",
            RangeTrend::Downtrend => "downtrend",
            RangeTrend::RangeBound => "range bound",
        }
    }

    pub fn as_data_attr(&self) -> &'static str {
        match self {
            RangeTrend::NearHigh => "near-high",
            RangeTrend::NearLow => "near-low",
            RangeTrend::Uptrend => "uptrend",
            RangeTrend::Downtrend => "downtrend",
            RangeTrend::RangeBound => "range-bound",
        }
    }
}

/// Classify the trend state of a 52-week range position.
///
/// - Degenerate range (`high <= low`) -> `RangeBound`.
/// - `pct >= 90.0` -> `NearHigh`; `pct <= 10.0` -> `NearLow` (checked before
///   `change_pct`, regardless of its sign).
/// - Otherwise: `change_pct > 0.5` -> `Uptrend`, `change_pct < -0.5` ->
///   `Downtrend`, else -> `RangeBound`.
pub fn classify_range_trend(low: f64, high: f64, last: f64, change_pct: f64) -> RangeTrend {
    if high <= low {
        return RangeTrend::RangeBound;
    }
    let pct = range_position_pct(low, high, last);
    if pct >= 90.0 {
        RangeTrend::NearHigh
    } else if pct <= 10.0 {
        RangeTrend::NearLow
    } else if change_pct > 0.5 {
        RangeTrend::Uptrend
    } else if change_pct < -0.5 {
        RangeTrend::Downtrend
    } else {
        RangeTrend::RangeBound
    }
}

#[component]
pub fn RangeBarCell(
    low: f64,
    high: f64,
    last: f64,
    #[prop(optional, default = 0.0)] change_pct: f64,
    low_label: String,
    high_label: String,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let trend = classify_range_trend(low, high, last, change_pct);

    view! {
        <div
            class=format!("sc-rangebar-cell {class}").trim().to_string()
            data-trend=trend.as_data_attr()
        >
            <HfMiniRangeBar low_label=low_label high_label=high_label low=low high=high last=last />
            <span class="sc-rangebar-cell__trend">{trend.as_label()}</span>
        </div>
    }
}
