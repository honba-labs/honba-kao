use openalgo_charts::curves::*;

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
fn spread_at_skips_tenors_without_coverage() {
    // Curve A covers both tenors of the 30-90 band; curve B only covers 30 and
    // cannot interpolate 90. The band must still yield Some, computed only from
    // the curve(s) where both tenors can be interpolated (A: 7.5 - 7.0 = 0.5).
    let mut ts = TermStructure::new();
    ts.add_curve(YieldCurve::new("A", "#10b981").with_point(30, 7.0).with_point(90, 7.5));
    ts.add_curve(YieldCurve::new("B", "#ef4444").with_point(30, 6.8));
    ts.add_spread_band(SpreadBand {
        name: "30-90".into(),
        short_tenor_days: 30,
        long_tenor_days: 90,
        color: "#f59e0b".into(),
    });
    let band = ts.spread_bands[0].clone();
    let spread = ts.spread_at(&band).expect("band should not fail as a whole");
    assert!((spread - 0.5).abs() < 1e-6);
}
