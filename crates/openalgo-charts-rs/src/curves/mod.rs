//! Yield curve / term structure canvas component.
//!
//! Renders the interest-rate term structure: one or more yield curves (e.g.
//! G-Sec, corporate AAA), optional historical overlays, and spread bands
//! between tenor pairs. Points are expressed as (tenor in days, yield in %).

use crate::canvas::engine::{CanvasEngine, SeriesStyle};
use crate::types::{TimeSeries, TimeSeriesPoint};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use wasm_bindgen::JsValue;

/// A single point on a yield curve: tenor in days, yield in percent.
///
/// `yield_pct` is a `Decimal` (zero-floats invariant): it feeds interpolation
/// and spread arithmetic that must be exact, not an `f64` approximation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct YieldPoint {
    pub tenor_days: u32,
    pub yield_pct: Decimal,
}

/// One yield curve (a series of tenors -> yields) with a name and colour.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct YieldCurve {
    pub name: String,
    pub color: String,
    pub points: Vec<YieldPoint>,
}

impl YieldCurve {
    pub fn new(name: impl Into<String>, color: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color: color.into(),
            points: Vec::new(),
        }
    }

    pub fn with_point(mut self, tenor_days: u32, yield_pct: Decimal) -> Self {
        self.points.push(YieldPoint { tenor_days, yield_pct });
        self
    }

    /// Linearly interpolate the yield at `tenor_days`, or `None` when the curve
    /// has no points or the tenor lies outside the observed range.
    pub fn interpolate(&self, tenor_days: u32) -> Option<Decimal> {
        if self.points.is_empty() {
            return None;
        }
        let mut pts = self.points.clone();
        pts.sort_by_key(|p| p.tenor_days);
        // Outside the observed range: cannot interpolate.
        if tenor_days < pts[0].tenor_days || tenor_days > pts[pts.len() - 1].tenor_days {
            return None;
        }
        for i in 0..pts.len() - 1 {
            let (a, b) = (&pts[i], &pts[i + 1]);
            if a.tenor_days <= tenor_days && tenor_days <= b.tenor_days {
                if b.tenor_days == a.tenor_days {
                    return Some(a.yield_pct);
                }
                let t = Decimal::from(tenor_days - a.tenor_days) / Decimal::from(b.tenor_days - a.tenor_days);
                return Some(a.yield_pct + t * (b.yield_pct - a.yield_pct));
            }
        }
        None
    }

    /// Convert to an engine-friendly line series (tenor days -> x, yield -> y).
    ///
    /// `yield_pct` is converted from `Decimal` to `f64` only here, at the wasm
    /// wire boundary (`TimeSeriesPoint` is the JS charting engine's wire type).
    pub fn to_series(&self) -> TimeSeries {
        let mut pts: Vec<TimeSeriesPoint> = self
            .points
            .iter()
            .map(|p| TimeSeriesPoint {
                time: p.tenor_days as i64,
                value: p.yield_pct.to_f64().unwrap_or_default(),
            })
            .collect();
        pts.sort_by_key(|p| p.time);
        TimeSeries {
            name: self.name.clone(),
            points: pts,
            color: self.color.clone(),
        }
    }
}

/// A spread band: the yield difference between a long and a short tenor.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SpreadBand {
    pub name: String,
    pub short_tenor_days: u32,
    pub long_tenor_days: u32,
    pub color: String,
}

impl SpreadBand {
    /// Render a computed spread as a flat two-point line spanning this band's
    /// tenor range, so it can be pushed to the chart as its own series
    /// alongside the curves/overlays instead of being silently dropped.
    ///
    /// `spread` is converted from `Decimal` to `f64` only here, at the wasm
    /// wire boundary.
    fn to_series(&self, spread: Decimal) -> TimeSeries {
        let value = spread.to_f64().unwrap_or_default();
        TimeSeries {
            name: self.name.clone(),
            points: vec![
                TimeSeriesPoint { time: self.short_tenor_days as i64, value },
                TimeSeriesPoint { time: self.long_tenor_days as i64, value },
            ],
            color: self.color.clone(),
        }
    }
}

/// The full term structure view: curves, historical overlays, and spread bands.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TermStructure {
    pub curves: Vec<YieldCurve>,
    pub overlays: Vec<YieldCurve>,
    pub spread_bands: Vec<SpreadBand>,
}

impl TermStructure {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_curve(&mut self, curve: YieldCurve) {
        self.curves.push(curve);
    }

    pub fn add_overlay(&mut self, overlay: YieldCurve) {
        self.overlays.push(overlay);
    }

    pub fn add_spread_band(&mut self, band: SpreadBand) {
        self.spread_bands.push(band);
    }

    /// The spread (long yield minus short yield) for a band, interpolated at the
    /// band's tenor pair. Curves that cannot interpolate either side of the band
    /// are skipped; `None` only when no curve covers both tenors.
    pub fn spread_at(&self, band: &SpreadBand) -> Option<Decimal> {
        // Evaluate each curve at both tenors and average across curves that
        // cover both tenors — a composite spread when multiple curves exist.
        let mut spreads = Vec::new();
        for curve in &self.curves {
            let Some(short) = curve.interpolate(band.short_tenor_days) else {
                continue;
            };
            let Some(long) = curve.interpolate(band.long_tenor_days) else {
                continue;
            };
            spreads.push(long - short);
        }
        if spreads.is_empty() {
            return None;
        }
        let sum: Decimal = spreads.iter().sum();
        Some(sum / Decimal::from(spreads.len()))
    }

    /// All curves (live + overlays) as engine line series.
    pub fn to_series(&self) -> Vec<TimeSeries> {
        let mut out: Vec<TimeSeries> = self.curves.iter().map(|c| c.to_series()).collect();
        out.extend(self.overlays.iter().map(|c| c.to_series()));
        out
    }

    /// Spread bands that could be computed (`spread_at` returned `Some`),
    /// each rendered as a flat two-point line spanning its tenor range. Bands
    /// with no coverage are omitted rather than pushed as an empty series.
    pub fn spread_band_series(&self) -> Vec<TimeSeries> {
        self.spread_bands
            .iter()
            .filter_map(|band| self.spread_at(band).map(|spread| band.to_series(spread)))
            .collect()
    }

    /// Every series this term structure renders — each curve, each overlay,
    /// and each spread band with coverage — one entry per series. This is
    /// exactly the payload `render` pushes to the bridge, exposed so the
    /// "every curve/overlay/band gets its own series" invariant can be
    /// asserted without a live chart.
    pub fn render_series(&self) -> Vec<TimeSeries> {
        let mut out = self.to_series();
        out.extend(self.spread_band_series());
        out
    }

    /// Render every curve, overlay, and computable spread band as its own
    /// line series on `engine`, then return the computed spread values.
    ///
    /// Each series is pushed through its own `bridge_set_series` call against
    /// the same chart id. The underlying bridge's `setSeries` *adds* a new
    /// series to the chart on every call (`window.honbaCharts` keeps a
    /// per-chart series list and never clears it before adding) rather than
    /// replacing the chart's series wholesale, so pushing one call per curve,
    /// overlay, and spread band renders all of them simultaneously instead of
    /// only the last one.
    pub fn render(&self, engine: &CanvasEngine) -> Result<Vec<(SpreadBand, Option<Decimal>)>, JsValue> {
        for series in self.render_series() {
            self.push_series(engine, &series)?;
        }
        let spreads: Vec<(SpreadBand, Option<Decimal>)> = self
            .spread_bands
            .iter()
            .map(|b| (b.clone(), self.spread_at(b)))
            .collect();
        Ok(spreads)
    }

    /// Push one series to the bridge as its own `setSeries` call.
    fn push_series(&self, engine: &CanvasEngine, series: &TimeSeries) -> Result<(), JsValue> {
        let points_json = serde_json::to_string(&series.points)
            .map_err(|e| JsValue::from_str(&format!("points serialise: {e}")))?;
        let style_json = serde_json::to_string(&SeriesStyle::line(&series.color))
            .map_err(|e| JsValue::from_str(&format!("style serialise: {e}")))?;
        let id = engine
            .id()
            .ok_or_else(|| JsValue::from_str("chart not mounted"))?
            .0;
        bridge_set_series(id, "line", &style_json, &points_json)?;
        Ok(())
    }
}

// Thin helper so `render` stays on `TermStructure` without importing bridge at
// crate root in a way that creates a cycle.
fn bridge_set_series(
    id: u64,
    kind: &str,
    style_json: &str,
    bars_json: &str,
) -> Result<bool, JsValue> {
    crate::canvas::bridge::set_series(id as f64, kind, style_json, bars_json)
}
