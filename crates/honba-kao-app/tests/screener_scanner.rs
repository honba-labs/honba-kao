//! T1..T8 contract tests for the deterministic technical scanner engine
//! (`features::screener::scanner`). Zero-floats invariant: `Decimal` only.

use honba_kao_app::features::screener::fixtures::equity_fixture;
use honba_kao_app::features::screener::scanner::{
    ExplainStep, SETUP_BB_SQUEEZE_BREAKOUT, SETUP_BREAKOUT_WATCH, SETUP_TREND_RETEST,
    SETUP_UNQUALIFIED, ScannerResult, scan_row, scan_universe,
};
use honba_kao_app::features::screener::types::ScreenerEquityRecord;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// RELIANCE-like baseline row (technicals zeroed except `rsi_14`,
/// `rvol` derived as volume / avg_volume_20d by the fixture).
fn base_row() -> ScreenerEquityRecord {
    equity_fixture(
        "RELIANCE",
        "Reliance Industries Ltd",
        "NSE",
        "Energy",
        "Oil & Gas",
        "Large Cap",
        true,
        true,
        dec!(2842.50),
        dec!(1.87),
        4_200_000,
        2_000_000,
        dec!(24.20),
        dec!(0.35),
        dec!(8.50),
        dec!(1_920_000),
        dec!(2790.00),
        dec!(2860.00),
        dec!(2220.07),
        dec!(3200.00),
        dec!(62.40),
    )
}

fn step<'a>(result: &'a ScannerResult, needle: &str) -> &'a ExplainStep {
    result
        .explain_steps
        .iter()
        .find(|s| s.condition.contains(needle))
        .unwrap_or_else(|| panic!("missing step containing {needle}"))
}

/// T1: trend-retest route with a full explanation trail.
/// Weights: trend 25, rvol 20, rsi 20, 52w 15, squeeze 20.
/// Evaluable = {trend, rvol, rsi, 52w} = 80; passed = {trend, rvol, rsi} = 65.
/// score = round2(100 * 65 / 80) = 81.25
/// (NOTE: the prompt's literal 85.94 does not equal its own formula
///  100*(25+20+20)/(25+20+20+15) = 81.25; the formula is authoritative).
#[test]
fn t1_trend_retest_scores_and_explains_each_step() {
    let mut r = base_row();
    r.sma_200 = dec!(2610.00);

    let result = scan_row(&r);
    assert_eq!(result.symbol, "RELIANCE");
    assert_eq!(result.setup_type, SETUP_TREND_RETEST);
    assert_eq!(
        result.explain_steps.len(),
        4,
        "squeeze not evaluable (bb zeros)"
    );

    let sma = step(&result, "SMA 200");
    assert!(sma.passed);
    assert_eq!(sma.actual_value, dec!(2842.50));
    assert_eq!(sma.threshold, dec!(2610.00));

    let rvol_step = step(&result, "RVOL");
    assert!(rvol_step.passed);
    assert_eq!(rvol_step.actual_value, dec!(2.10));
    assert_eq!(rvol_step.threshold, dec!(1.50));

    let rsi_step = step(&result, "RSI");
    assert!(rsi_step.passed);
    assert_eq!(rsi_step.threshold, dec!(55.00));

    let w52 = step(&result, "52W");
    assert!(!w52.passed, "2842.50 < 95% of 3200.00");
    assert_eq!(w52.threshold, dec!(3040.00));

    assert_eq!(result.score, dec!(81.25));
}

/// T2: default fixture (no technicals set) skips non-evaluable pivots;
/// only rvol / rsi / 52w steps are listed and can contribute `passed`.
#[test]
fn t2_default_row_skips_non_evaluable_detectors() {
    let r = base_row();
    let result = scan_row(&r);

    assert!(
        !result
            .explain_steps
            .iter()
            .any(|s| s.condition.contains("SMA 200"))
    );
    assert!(
        !result
            .explain_steps
            .iter()
            .any(|s| s.condition.contains("Bollinger"))
    );
    assert_eq!(result.explain_steps.len(), 3);

    for s in &result.explain_steps {
        let known = s.condition.contains("RVOL")
            || s.condition.contains("RSI")
            || s.condition.contains("52W");
        assert!(known, "unexpected step: {}", s.condition);
    }
    assert!(step(&result, "RVOL").passed);
    assert!(step(&result, "RSI").passed);
    assert!(!step(&result, "52W").passed);
    assert_eq!(result.setup_type, SETUP_UNQUALIFIED);
}

/// T3: everything fails / is non-evaluable -> score 0, unqualified.
#[test]
fn t3_score_zero_and_unqualified() {
    let mut r = base_row();
    r.rsi_14 = dec!(0);
    r.volume = 0;
    r.rvol = dec!(0);
    r.high_52w = dec!(0);

    let result = scan_row(&r);
    assert_eq!(result.score, Decimal::ZERO);
    assert_eq!(result.setup_type, SETUP_UNQUALIFIED);
}

/// T4: squeeze route wins over a passing trend detector (precedence).
/// Note: price 2842.50 sits INSIDE the band (2750..2850); the BB setup is
/// selected on the squeeze (bandwidth 3.50 < 5.00) alone, with the breakout
/// trigger pinned at bb_upper.
#[test]
fn t4_bb_squeeze_breakout_takes_precedence_over_trend() {
    let mut r = base_row();
    r.sma_200 = dec!(2610.00);
    r.bb_upper = dec!(2850.00);
    r.bb_lower = dec!(2750.00);
    r.bb_bandwidth = dec!(3.50);

    let result = scan_row(&r);
    let bb = step(&result, "Bollinger");
    assert!(bb.passed, "bandwidth 3.50 < 5.00");
    assert_eq!(bb.actual_value, dec!(3.50));
    assert_eq!(bb.threshold, dec!(5.00));

    assert!(step(&result, "SMA 200").passed, "trend also passes");
    assert_eq!(result.setup_type, SETUP_BB_SQUEEZE_BREAKOUT);
    assert_eq!(result.trigger_level, dec!(2850.00));
    assert_eq!(result.invalidation_level, dec!(2750.00));
}

/// T5: squeeze evaluable but FAILED -> trend route with pivot-derived levels.
/// invalidation = smallest nonzero of {day_low, sma_50}; when day_low is 0
/// it falls back to the sma_50 pivot.
#[test]
fn t5_trend_route_invalidations_fall_back_to_sma50() {
    let mut r = base_row();
    r.sma_200 = dec!(2610.00);
    r.sma_50 = dec!(2800.00);
    r.bb_upper = dec!(2900.00);
    r.bb_bandwidth = dec!(9.00);

    let result = scan_row(&r);
    assert!(!step(&result, "Bollinger").passed, "bandwidth 9.00 >= 5.00");
    assert_eq!(result.setup_type, SETUP_TREND_RETEST);
    assert_eq!(result.trigger_level, dec!(2610.00));
    assert_eq!(result.invalidation_level, dec!(2790.00), "min(2790, 2800)");

    let mut r2 = base_row();
    r2.sma_200 = dec!(2610.00);
    r2.sma_50 = dec!(2800.00);
    r2.bb_upper = dec!(2900.00);
    r2.bb_bandwidth = dec!(9.00);
    r2.day_low = dec!(0);

    let result2 = scan_row(&r2);
    assert_eq!(result2.setup_type, SETUP_TREND_RETEST);
    assert_eq!(result2.invalidation_level, dec!(2800.00), "sma_50 fallback");
}

/// T6: only the 52W detector passes -> breakout watch pinned to the 52W span.
/// price raised to 3100.00 (>= 95% of 3200.00); rvol = 800k/2M = 0.40 fails,
/// rsi 9.5 fails, sma_200/bb zeros keep trend + squeeze non-evaluable.
#[test]
fn t6_52w_breakout_watch_route() {
    let mut r = base_row();
    r.symbol = "BKT".to_string();
    r.price = dec!(3100.00);
    r.rsi_14 = dec!(9.5);
    r.volume = 800_000;
    r.rvol = dec!(0.40);
    r.sma_200 = Decimal::ZERO;
    r.bb_bandwidth = Decimal::ZERO;

    let result = scan_row(&r);
    assert!(!step(&result, "RVOL").passed);
    assert!(!step(&result, "RSI").passed);
    assert!(step(&result, "52W").passed, "3100.00 >= 3040.00");
    assert_eq!(result.setup_type, SETUP_BREAKOUT_WATCH);
    assert_eq!(result.trigger_level, dec!(3200.00));
    assert_eq!(result.invalidation_level, dec!(2220.07));
}

/// T7: scan_universe sorts by score descending (stable for ties) and is
/// total (output length == input length).
#[test]
fn t7_scan_universe_sorts_descending_and_keeps_input_len() {
    let mut t3 = base_row();
    t3.symbol = "T3SYM".to_string();
    t3.rsi_14 = Decimal::ZERO;
    t3.volume = 0;
    t3.rvol = Decimal::ZERO;
    t3.high_52w = Decimal::ZERO;

    // "T4 variant": same construction as T4 but with rvol 6.10
    // (volume 12_200_000 / avg 2_000_000).
    let mut t4v = base_row();
    t4v.symbol = "T4SYM".to_string();
    t4v.sma_200 = dec!(2610.00);
    t4v.bb_upper = dec!(2850.00);
    t4v.bb_lower = dec!(2750.00);
    t4v.bb_bandwidth = dec!(3.50);
    t4v.volume = 12_200_000;
    t4v.rvol = dec!(6.10);

    let mut t5 = base_row();
    t5.symbol = "T5SYM".to_string();
    t5.sma_200 = dec!(2610.00);
    t5.sma_50 = dec!(2800.00);
    t5.bb_upper = dec!(2900.00);
    t5.bb_bandwidth = dec!(9.00);

    let mut t1 = base_row();
    t1.symbol = "T1SYM".to_string();
    t1.sma_200 = dec!(2610.00);

    let universe = vec![t3, t4v, t5, t1];
    let results = scan_universe(&universe);
    assert_eq!(results.len(), universe.len());
    assert_eq!(results[0].symbol, "T4SYM", "85.00 is the top score");

    // Stability: equal scores preserve input order.
    let mut tie_a = base_row();
    tie_a.symbol = "TIE_A".to_string();
    let mut tie_b = base_row();
    tie_b.symbol = "TIE_B".to_string();
    let tied = scan_universe(&[tie_a, tie_b]);
    let scores: Vec<Decimal> = tied.iter().map(|r| r.score).collect();
    assert_eq!(scores[0], scores[1], "fixture-default rows tie");
    assert_eq!(tied[0].symbol, "TIE_A");
    assert_eq!(tied[1].symbol, "TIE_B");
}

/// T8: ScannerResult / ExplainStep round-trip through serde_json intact.
#[test]
fn t8_scanner_result_serde_roundtrip() {
    let result = scan_row(&base_row());
    let cloned = result.clone();
    assert_eq!(cloned, result);

    let json = serde_json::to_string(&result).expect("serialize ScannerResult");
    let back: ScannerResult = serde_json::from_str(&json).expect("deserialize ScannerResult");
    assert_eq!(back, result);
    assert_eq!(back.score, result.score);

    let step_json = serde_json::to_string(&result.explain_steps[0]).expect("serialize step");
    let step_back: ExplainStep = serde_json::from_str(&step_json).expect("deserialize step");
    assert_eq!(step_back, result.explain_steps[0]);
}
