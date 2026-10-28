use openalgo_charts::curves::*;
use rust_decimal_macros::dec;
use std::collections::HashSet;

#[test]
fn interpolate_within_range() {
    let curve = YieldCurve::new("G-Sec", "#10b981")
        .with_point(1, dec!(6.5))
        .with_point(30, dec!(6.75))
        .with_point(90, dec!(7.0));
    // Exactly on a point.
    assert_eq!(curve.interpolate(30), Some(dec!(6.75)));
    assert_eq!(curve.interpolate(90), Some(dec!(7.0)));
    // Midpoint between 30 and 90 (t = 15/60 = 0.25 exactly):
    // 6.75 + 0.25 * (7.0 - 6.75) = 6.8125.
    assert_eq!(curve.interpolate(45), Some(dec!(6.8125)));
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
        .with_point(90, dec!(7.0))
        .with_point(1, dec!(6.5))
        .with_point(30, dec!(6.75));
    let series = curve.to_series();
    let times: Vec<i64> = series.points.iter().map(|p| p.time).collect();
    assert_eq!(times, vec![1, 30, 90]);
    // yield_pct is converted Decimal -> f64 only at this wire boundary.
    let values: Vec<f64> = series.points.iter().map(|p| p.value).collect();
    assert_eq!(values, vec![6.5, 6.75, 7.0]);
}

#[test]
fn spread_at_average_across_curves() {
    let mut ts = TermStructure::new();
    ts.add_curve(
        YieldCurve::new("A", "#10b981")
            .with_point(30, dec!(7.0))
            .with_point(90, dec!(7.5)),
    );
    ts.add_curve(
        YieldCurve::new("B", "#ef4444")
            .with_point(30, dec!(6.8))
            .with_point(90, dec!(7.2)),
    );
    ts.add_spread_band(SpreadBand {
        name: "30-90".into(),
        short_tenor_days: 30,
        long_tenor_days: 90,
        color: "#f59e0b".into(),
    });
    // Curve A spread: 0.5; Curve B spread: 0.4; average: 0.45.
    let band = ts.spread_bands[0].clone();
    assert_eq!(ts.spread_at(&band), Some(dec!(0.45)));
}

#[test]
fn spread_at_skips_tenors_without_coverage() {
    // Curve A covers both tenors of the 30-90 band; curve B only covers 30 and
    // cannot interpolate 90. The band must still yield Some, computed only from
    // the curve(s) where both tenors can be interpolated (A: 7.5 - 7.0 = 0.5).
    let mut ts = TermStructure::new();
    ts.add_curve(
        YieldCurve::new("A", "#10b981")
            .with_point(30, dec!(7.0))
            .with_point(90, dec!(7.5)),
    );
    ts.add_curve(YieldCurve::new("B", "#ef4444").with_point(30, dec!(6.8)));
    ts.add_spread_band(SpreadBand {
        name: "30-90".into(),
        short_tenor_days: 30,
        long_tenor_days: 90,
        color: "#f59e0b".into(),
    });
    let band = ts.spread_bands[0].clone();
    assert_eq!(
        ts.spread_at(&band),
        Some(dec!(0.5)),
        "band should not fail as a whole"
    );
}

#[test]
fn render_series_gives_each_curve_overlay_and_spread_band_its_own_series() {
    // Regression test for issue #30: render() used to push every curve and
    // overlay through the same chart id without ever drawing the spread
    // bands at all. `render_series` is the exact payload `render` now pushes
    // to the bridge, one entry per `setSeries` call, so asserting on it
    // proves curves/overlays/bands each land in their own series.
    let mut ts = TermStructure::new();
    ts.add_curve(
        YieldCurve::new("G-Sec", "#10b981")
            .with_point(30, dec!(7.0))
            .with_point(90, dec!(7.5)),
    );
    ts.add_overlay(
        YieldCurve::new("G-Sec (last week)", "#6b7280")
            .with_point(30, dec!(6.9))
            .with_point(90, dec!(7.4)),
    );
    ts.add_spread_band(SpreadBand {
        name: "30-90 spread".into(),
        short_tenor_days: 30,
        long_tenor_days: 90,
        color: "#f59e0b".into(),
    });
    // A band with no coverage at all must be omitted, not pushed as a bogus
    // empty/zero series.
    ts.add_spread_band(SpreadBand {
        name: "uncovered".into(),
        short_tenor_days: 1_000,
        long_tenor_days: 2_000,
        color: "#000000".into(),
    });

    let series = ts.render_series();
    let names: Vec<&str> = series.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["G-Sec", "G-Sec (last week)", "30-90 spread"]);

    // Every pushed series must carry a distinct identifying key (its name) —
    // none silently overwrites another.
    let distinct: HashSet<&str> = names.iter().copied().collect();
    assert_eq!(
        distinct.len(),
        names.len(),
        "every rendered series must have a distinct name/key"
    );

    // The spread band is now actually drawn: a flat line at the computed
    // spread (7.5 - 7.0 = 0.5) spanning the band's tenor range.
    let band_series = series.last().expect("spread band series present");
    assert_eq!(band_series.points.len(), 2);
    assert_eq!(band_series.points[0].time, 30);
    assert_eq!(band_series.points[0].value, 0.5);
    assert_eq!(band_series.points[1].time, 90);
    assert_eq!(band_series.points[1].value, 0.5);
}

#[test]
fn spread_band_series_omits_bands_without_coverage() {
    let mut ts = TermStructure::new();
    ts.add_curve(
        YieldCurve::new("A", "#10b981")
            .with_point(30, dec!(7.0))
            .with_point(90, dec!(7.5)),
    );
    ts.add_spread_band(SpreadBand {
        name: "uncovered".into(),
        short_tenor_days: 1_000,
        long_tenor_days: 2_000,
        color: "#000000".into(),
    });
    assert!(ts.spread_band_series().is_empty());
}
