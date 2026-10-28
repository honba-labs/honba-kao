//! Yield curve / term structure canvas component.
//!
//! Renders the interest-rate term structure: one or more yield curves (e.g.
//! G-Sec, corporate AAA), optional historical overlays, and spread bands
//! between tenor pairs. Points are expressed as (tenor in days, yield in %).

use crate::canvas::engine::{CanvasEngine, SeriesStyle};
use crate::types::{TimeSeries, TimeSeriesPoint};
use wasm_bindgen::JsValue;

/// A single point on a yield curve: tenor in days, yield in percent.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct YieldPoint {
    pub tenor_days: u32,
    pub yield_pct: f64,
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

    pub fn with_point(mut self, tenor_days: u32, yield_pct: f64) -> Self {
        self.points.push(YieldPoint { tenor_days, yield_pct });
        self
    }

    /// Linearly interpolate the yield at `tenor_days`, or `None` when the curve
    /// has no points or the tenor lies outside the observed range.
    pub fn interpolate(&self, tenor_days: u32) -> Option<f64> {
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
                let t = (tenor_days - a.tenor_days) as f64 / (b.tenor_days - a.tenor_days) as f64;
                return Some(a.yield_pct + t * (b.yield_pct - a.yield_pct));
            }
        }
        None
    }

    /// Convert to an engine-friendly line series (tenor days -> x, yield -> y).
    pub fn to_series(&self) -> TimeSeries {
        let mut pts: Vec<TimeSeriesPoint> = self
            .points
            .iter()
            .map(|p| TimeSeriesPoint {
                time: p.tenor_days as i64,
                value: p.yield_pct,
            })
            .collect();
        pts.sort_by(|a, b| a.time.cmp(&b.time));
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
    /// band's tenor pair; `None` when either side cannot be interpolated.
    pub fn spread_at(&self, band: &SpreadBand) -> Option<f64> {
        // Evaluate each curve at both tenors and average across curves that
        // cover both tenors — a composite spread when multiple curves exist.
        let mut spreads = Vec::new();
        for curve in &self.curves {
            let short = curve.interpolate(band.short_tenor_days)?;
            let long = curve.interpolate(band.long_tenor_days)?;
            spreads.push(long - short);
        }
        if spreads.is_empty() {
            return None;
        }
        Some(spreads.iter().sum::<f64>() / spreads.len() as f64)
    }

    /// All curves (live + overlays) as engine line series.
    pub fn to_series(&self) -> Vec<TimeSeries> {
        let mut out: Vec<TimeSeries> = self.curves.iter().map(|c| c.to_series()).collect();
        out.extend(self.overlays.iter().map(|c| c.to_series()));
        out
    }

    /// Render every curve and overlay as line series on `engine`, then compute
    /// and return the spread bands.
    pub fn render(&self, engine: &CanvasEngine) -> Result<Vec<(SpreadBand, Option<f64>)>, JsValue> {
        for series in self.to_series() {
            let points_json = serde_json::to_string(&series.points)
                .map_err(|e| JsValue::from_str(&format!("points serialise: {e}")))?;
            let style_json = serde_json::to_string(&SeriesStyle::line(&series.color))
                .map_err(|e| JsValue::from_str(&format!("style serialise: {e}")))?;
            let id = engine
                .id()
                .ok_or_else(|| JsValue::from_str("chart not mounted"))?
                .0;
            bridge_set_series(id, "line", &style_json, &points_json)?;
        }
        let spreads: Vec<(SpreadBand, Option<f64>)> = self
            .spread_bands
            .iter()
            .map(|b| (b.clone(), self.spread_at(b)))
            .collect();
        Ok(spreads)
    }
}

// Thin helper so `render` stays on `TermStructure` without importing bridge at
// crate root in a way that creates a cycle.
fn bridge_set_series(
    id: f64,
    kind: &str,
    style_json: &str,
    bars_json: &str,
) -> Result<bool, JsValue> {
    crate::canvas::bridge::set_series(id, kind, style_json, bars_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolate_within_range() {
        let curve = YieldCurve::new("G-Sec", "#10b981")
            .with_point(1, 6.5)
            .with_point(30, 6.75)
            .with_point(90, 7.0);
        // Exactly on a point.
        assert!((curve.interpolate(30).unwrap() - 6.75).abs() < f64::EPSILON);
        assert!((curve.interpolate(90).unwrap() - 7.0).abs() < f64::EPSILON);
        // Midpoint-ish between 1 and 30: 6.5 + (15/29)*0.25.
        let expected = 6.5 + ((15f64 / 29.0) * 0.25);
        assert!((curve.interpolate(16).unwrap() - expected).abs() < 1e-6);
        // Outside the observed range -> None (no clamping).
        assert!(curve.interpolate(0).is_none());
        assert!(curve.interpolate(100).is_none());
        // Empty curve -> None.
        let empty: YieldCurve = YieldCurve::new("empty", "#000000");
        assert!(empty.interpolate(30).is_none());
    }

    #[test]
    fn to_series_is_sorted_by_tenor() {
        let curve = YieldCurve::new("C", "#10b981")
            .with_point(90, 7.0)
            .with_point(1, 6.5)
            .with_point(30, 6.75);
        let series = curve.to_series();
        let times: Vec<i64> = series.points.iter().map(|p| p.time).collect();
        assert_eq!(times, vec![1, 30, 90]);
    }

    #[test]
    fn spread_at_average_across_curves() {
        let mut ts = TermStructure::new();
        ts.add_curve(YieldCurve::new("A", "#10b981").with_point(30, 7.0).with_point(90, 7.5));
        ts.add_curve(YieldCurve::new("B", "#ef4444").with_point(30, 6.8).with_point(90, 7.2));
        ts.add_spread_band(SpreadBand {
            name: "30-90".into(),
            short_tenor_days: 30,
            long_tenor_days: 90,
            color: "#f59e0b".into(),
        });
        // Curve A spread: 0.5; Curve B spread: 0.4; average: 0.45.
        let band = ts.spread_bands[0].clone();
        assert!((ts.spread_at(&band).unwrap() - 0.45).abs() < 1e-6);
    }

    #[test]
    fn spread_at_returns_none_when_uninterpolatable() {
        let mut ts = TermStructure::new();
        ts.add_curve(YieldCurve::new("A", "#10b981").with_point(30, 7.0));
        ts.add_spread_band(SpreadBand {
            name: "30-90".into(),
            short_tenor_days: 30,
            long_tenor_days: 90,
            color: "#f59e0b".into(),
        });
        assert!(ts.spread_at(&ts.spread_bands[0]).is_none());
    }
}
