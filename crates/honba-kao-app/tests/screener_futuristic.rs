//! TDD contract tests for the Futuristic Neural AST Screener telemetry,
//! view modes (`Matrix`, `Constellation`), `FunnelTelemetry`, `LaserGauge`,
//! `ConstellationPoint`, `RadarAxes`, and component exports.
//!
//! Strictly adheres to the Zero-Floats Policy (`rust_decimal::Decimal` only).

use honba_kao_app::features::screener::components::{ScreenerConstellation, ScreenerSignalMatrix};
use honba_kao_app::features::screener::fixtures::{equity_fixture, stock_universe};
use honba_kao_app::features::screener::scanner::{
    SETUP_BB_SQUEEZE_BREAKOUT, SETUP_TREND_RETEST, scan_row,
};
use honba_kao_app::features::screener::telemetry::{
    ConstellationPoint, FunnelTelemetry, LaserGauge, RadarAxes, compute_constellation,
    compute_funnel, compute_laser_gauge, compute_radar_axes,
};
use honba_kao_app::features::screener::types::{FilterCriteria, ScreenerViewMode};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[test]
fn t1_view_modes_include_matrix_and_constellation() {
    let modes = [
        ScreenerViewMode::Matrix,
        ScreenerViewMode::Constellation,
        ScreenerViewMode::Table,
        ScreenerViewMode::Heatmap,
    ];
    assert_eq!(modes.len(), 4);
    assert_ne!(ScreenerViewMode::Matrix, ScreenerViewMode::Table);
    assert_ne!(ScreenerViewMode::Constellation, ScreenerViewMode::Heatmap);
}

#[test]
fn t2_funnel_telemetry_counts_universe_index_matched_and_qualified() {
    let universe = stock_universe();
    let criteria = FilterCriteria {
        market_index: "NIFTY50".to_string(),
        min_rvol: dec!(1.50),
        ..FilterCriteria::default()
    };
    let funnel: FunnelTelemetry = compute_funnel(&universe, &criteria);
    assert_eq!(funnel.total_universe, 30);
    assert_eq!(
        funnel.index_universe, 14,
        "NIFTY50 has 14 members in fixture"
    );
    assert!(funnel.matched_count <= funnel.index_universe);
    assert!(funnel.qualified_count <= funnel.matched_count);
}

#[test]
fn t3_laser_gauge_computes_exact_position_and_rr_ratio() {
    // Price 2940 between invalidation 2880 and trigger 2970:
    // risk = 60, reward = 30 -> R:R = 0.50; position = (2940-2880)/(2970-2880)*100 = 66.67%
    let gauge: LaserGauge = compute_laser_gauge(dec!(2940), dec!(2880), dec!(2970));
    assert_eq!(gauge.position_pct, dec!(66.67));
    assert_eq!(gauge.reward_risk_ratio, dec!(0.50));
    assert_eq!(gauge.dist_to_invalidation_pct, dec!(-2.04));
    assert_eq!(gauge.dist_to_trigger_pct, dec!(1.02));

    // Price closer to invalidation (2895 in 2880..2970):
    // risk = 15, reward = 75 -> R:R = 5.00; position = 15/90*100 = 16.67%
    let high_rr = compute_laser_gauge(dec!(2895), dec!(2880), dec!(2970));
    assert_eq!(high_rr.position_pct, dec!(16.67));
    assert_eq!(high_rr.reward_risk_ratio, dec!(5.00));
}

#[test]
fn t4_constellation_points_normalize_rsi_rvol_and_score_with_zero_floats() {
    let mut row = equity_fixture(
        "RELIANCE",
        "Reliance Industries Ltd",
        "NSE",
        "Energy",
        "Oil & Gas",
        "Large Cap",
        true,
        true,
        dec!(2945.60),
        dec!(2.45),
        4_900_000,
        2_000_000,
        dec!(24.80),
        dec!(0.35),
        dec!(18.40),
        dec!(1_990_000),
        dec!(2880.00),
        dec!(2960.00),
        dec!(2220.00),
        dec!(3024.00),
        dec!(64.20),
    );
    row.sma_200 = dec!(2790.00);
    row.bb_upper = dec!(2960.00);
    row.bb_lower = dec!(2880.00);
    row.bb_bandwidth = dec!(3.40);

    let pts: Vec<ConstellationPoint> = compute_constellation(&[row]);
    assert_eq!(pts.len(), 1);
    let pt = &pts[0];
    assert_eq!(pt.symbol, "RELIANCE");
    assert_eq!(pt.setup_type, SETUP_BB_SQUEEZE_BREAKOUT);
    assert_eq!(pt.score, dec!(100.00));
    assert!(pt.x_pct >= Decimal::ZERO && pt.x_pct <= dec!(100));
    assert!(pt.y_pct >= Decimal::ZERO && pt.y_pct <= dec!(100));
    assert!(pt.radius_px >= dec!(8) && pt.radius_px <= dec!(22));
}

#[test]
fn t5_radar_axes_computes_five_bounded_decimal_scores() {
    let mut row = stock_universe().remove(0);
    row.sma_200 = dec!(2610.00);
    row.bb_bandwidth = dec!(3.50);
    row.bb_upper = dec!(2850.00);
    row.bb_lower = dec!(2750.00);
    let scan = scan_row(&row);
    assert_eq!(scan.setup_type, SETUP_BB_SQUEEZE_BREAKOUT);

    let axes: RadarAxes = compute_radar_axes(&row);
    for val in [axes.squeeze, axes.trend, axes.rvol, axes.rsi, axes.roe] {
        assert!(
            val >= Decimal::ZERO && val <= dec!(100),
            "radar axis {val} out of 0..=100"
        );
    }
    assert!(!axes.svg_polygon_points().is_empty());
}

#[test]
fn t6_futuristic_components_are_exported() {
    let _ = ScreenerSignalMatrix;
    let _ = ScreenerConstellation;
    let _ = SETUP_TREND_RETEST;
}
