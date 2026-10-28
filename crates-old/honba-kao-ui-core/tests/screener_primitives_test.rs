//! Unit tests for `honba-kao-ui-core` screener primitives (Milestone 1).
//!
//! These run on the host (no WASM/DOM required) and cover the deterministic,
//! pure-logic surface of each primitive: enum string/class/label mappings and
//! computed values. Interactive DOM rendering and event handling are covered
//! by the Playwright E2E suite in Phase 8.2.

use honba_kao_ui_core::*;

#[test]
fn score_tone_boundaries() {
    assert_eq!(score_tone(100.0), ScoreTone::High);
    assert_eq!(score_tone(75.0), ScoreTone::High);
    assert_eq!(score_tone(74.9), ScoreTone::Mid);
    assert_eq!(score_tone(50.0), ScoreTone::Mid);
    assert_eq!(score_tone(49.9), ScoreTone::Low);
    assert_eq!(score_tone(0.0), ScoreTone::Low);
    assert_eq!(score_tone(f64::NAN), ScoreTone::Low);
}

#[test]
fn score_tone_as_class() {
    assert_eq!(ScoreTone::High.as_class(), "badge-high");
    assert_eq!(ScoreTone::Mid.as_class(), "badge-mid");
    assert_eq!(ScoreTone::Low.as_class(), "badge-low");
}

#[test]
fn factor_driver_label_and_class() {
    assert_eq!(FactorDriver::Value.as_label(), "VALUE");
    assert_eq!(FactorDriver::Value.as_class(), "factor-value");

    assert_eq!(FactorDriver::Momentum.as_label(), "MOM");
    assert_eq!(FactorDriver::Momentum.as_class(), "factor-momentum");

    assert_eq!(FactorDriver::Quality.as_label(), "QUALITY");
    assert_eq!(FactorDriver::Quality.as_class(), "factor-quality");

    assert_eq!(FactorDriver::LowVol.as_label(), "LOW-VOL");
    assert_eq!(FactorDriver::LowVol.as_class(), "factor-lowvol");
}

#[test]
fn classify_range_trend_degenerate_range() {
    // high <= low is degenerate, regardless of last/change_pct.
    assert_eq!(
        classify_range_trend(100.0, 100.0, 95.0, 2.0),
        RangeTrend::RangeBound
    );
    assert_eq!(
        classify_range_trend(150.0, 100.0, 95.0, -5.0),
        RangeTrend::RangeBound
    );
}

#[test]
fn classify_range_trend_near_high_regardless_of_change_pct() {
    // low=100, high=200, last=195 -> pct = 95 -> NearHigh.
    assert_eq!(
        classify_range_trend(100.0, 200.0, 195.0, 0.0),
        RangeTrend::NearHigh
    );
    assert_eq!(
        classify_range_trend(100.0, 200.0, 195.0, -5.0),
        RangeTrend::NearHigh
    );
}

#[test]
fn classify_range_trend_near_low() {
    // low=100, high=200, last=109 -> pct = 9 -> NearLow.
    assert_eq!(
        classify_range_trend(100.0, 200.0, 109.0, 3.0),
        RangeTrend::NearLow
    );
}

#[test]
fn classify_range_trend_mid_range_uptrend() {
    // low=100, high=200, last=150 -> pct = 50 (mid-range), change_pct=1.0 -> Uptrend.
    assert_eq!(
        classify_range_trend(100.0, 200.0, 150.0, 1.0),
        RangeTrend::Uptrend
    );
}

#[test]
fn classify_range_trend_mid_range_downtrend() {
    // low=100, high=200, last=150 -> pct = 50 (mid-range), change_pct=-1.0 -> Downtrend.
    assert_eq!(
        classify_range_trend(100.0, 200.0, 150.0, -1.0),
        RangeTrend::Downtrend
    );
}

#[test]
fn classify_range_trend_mid_range_bound() {
    // low=100, high=200, last=150 -> pct = 50 (mid-range), change_pct=0.0 -> RangeBound.
    assert_eq!(
        classify_range_trend(100.0, 200.0, 150.0, 0.0),
        RangeTrend::RangeBound
    );
}

#[test]
fn range_trend_label_and_data_attr() {
    assert_eq!(RangeTrend::NearHigh.as_label(), "near high");
    assert_eq!(RangeTrend::NearHigh.as_data_attr(), "near-high");

    assert_eq!(RangeTrend::NearLow.as_label(), "near low");
    assert_eq!(RangeTrend::NearLow.as_data_attr(), "near-low");

    assert_eq!(RangeTrend::Uptrend.as_label(), "uptrend");
    assert_eq!(RangeTrend::Uptrend.as_data_attr(), "uptrend");

    assert_eq!(RangeTrend::Downtrend.as_label(), "downtrend");
    assert_eq!(RangeTrend::Downtrend.as_data_attr(), "downtrend");

    assert_eq!(RangeTrend::RangeBound.as_label(), "range bound");
    assert_eq!(RangeTrend::RangeBound.as_data_attr(), "range-bound");
}
