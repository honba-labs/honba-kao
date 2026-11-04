//! Technical scanner engine (screener-honba-leptos.md §4.4/§6.3): deterministic, zero-float detector evaluation over ScreenerEquityRecord — every qualified match explains itself step by step.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use super::types::ScreenerEquityRecord;

/// Squeeze breakout: narrow Bollinger bandwidth, breakout pending at bb_upper.
pub const SETUP_BB_SQUEEZE_BREAKOUT: &str = "BB_SQUEEZE_BREAKOUT";
/// Pullback retest of the long-term trend pivot (SMA 200).
pub const SETUP_TREND_RETEST: &str = "TREND_RETEST";
/// Approaching the 52-week high/low extremes.
pub const SETUP_BREAKOUT_WATCH: &str = "BREAKOUT_WATCH";
/// No detector qualified; levels fall back to quote-derived pivots.
pub const SETUP_UNQUALIFIED: &str = "UNQUALIFIED";

/// One detector verdict: human-readable condition, exact operands and outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplainStep {
    pub condition: String,
    pub actual_value: Decimal,
    pub threshold: Decimal,
    pub passed: bool,
}

/// Scanner verdict for a single universe row: setup classification, weighted
/// score (0..100), route-specific trigger/invalidation pivots and the full
/// explanation trail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScannerResult {
    pub symbol: String,
    pub setup_type: String,
    pub score: Decimal,
    pub trigger_level: Decimal,
    pub invalidation_level: Decimal,
    pub explain_steps: Vec<ExplainStep>,
}

/// Smallest strictly-positive candidate (zeroed pivots are skipped).
fn smallest_nonzero(candidates: &[Decimal]) -> Option<Decimal> {
    candidates
        .iter()
        .copied()
        .filter(|v| *v > Decimal::ZERO)
        .min()
}

/// Evaluate every detector over `record` in the fixed contract order
/// (squeeze → trend → RVOL → RSI → 52W), score the weighted pass share and
/// select the setup route with its trigger/invalidation pivots.
pub fn scan_row(record: &ScreenerEquityRecord) -> ScannerResult {
    let price = record.price;
    let mut explain_steps: Vec<ExplainStep> = Vec::with_capacity(5);
    let mut passed_weight = Decimal::ZERO;
    let mut evaluable_weight = Decimal::ZERO;

    // 1) Bollinger Band Squeeze — evaluable iff bb_upper && bb_bandwidth are set.
    let squeeze_evaluable =
        record.bb_upper != Decimal::ZERO && record.bb_bandwidth != Decimal::ZERO;
    let squeeze_passed = squeeze_evaluable && record.bb_bandwidth < dec!(5.00);
    if squeeze_evaluable {
        explain_steps.push(ExplainStep {
            condition: format!(
                "Bollinger Bandwidth {}% < 5.00%",
                record.bb_bandwidth.round_dp(2)
            ),
            actual_value: record.bb_bandwidth,
            threshold: dec!(5.00),
            passed: squeeze_passed,
        });
        evaluable_weight += dec!(20);
        if squeeze_passed {
            passed_weight += dec!(20);
        }
    }

    // 2) Long-term trend — evaluable iff sma_200 pivot exists.
    let trend_evaluable = record.sma_200 != Decimal::ZERO;
    let trend_passed = trend_evaluable && price > record.sma_200;
    if trend_evaluable {
        explain_steps.push(ExplainStep {
            condition: format!(
                "Price ₹{} > SMA 200 ₹{}",
                price.round_dp(2),
                record.sma_200.round_dp(2)
            ),
            actual_value: price,
            threshold: record.sma_200,
            passed: trend_passed,
        });
        evaluable_weight += dec!(25);
        if trend_passed {
            passed_weight += dec!(25);
        }
    }

    // 3) Relative volume — always evaluable (rvol is derived at fixture time).
    let rvol_threshold = dec!(1.50);
    let rvol_passed = record.rvol >= rvol_threshold;
    explain_steps.push(ExplainStep {
        condition: format!("RVOL {}x ≥ {}x", record.rvol.round_dp(2), rvol_threshold),
        actual_value: record.rvol.round_dp(2),
        threshold: rvol_threshold,
        passed: rvol_passed,
    });
    evaluable_weight += dec!(20);
    if rvol_passed {
        passed_weight += dec!(20);
    }

    // 4) RSI momentum band 55.00..=70.00 — always evaluable.
    let rsi_passed = record.rsi_14 >= dec!(55.00) && record.rsi_14 <= dec!(70.00);
    explain_steps.push(ExplainStep {
        condition: format!("RSI 14 {} within 55.00–70.00", record.rsi_14.round_dp(1)),
        actual_value: record.rsi_14,
        threshold: dec!(55.00),
        passed: rsi_passed,
    });
    evaluable_weight += dec!(20);
    if rsi_passed {
        passed_weight += dec!(20);
    }

    // 5) 52-week high proximity — evaluable iff high_52w pivot exists.
    let w52_evaluable = record.high_52w != Decimal::ZERO;
    let w52_threshold = (record.high_52w * dec!(0.95)).round_dp(2);
    let w52_passed = w52_evaluable && price >= w52_threshold;
    if w52_evaluable {
        explain_steps.push(ExplainStep {
            condition: format!(
                "Price ₹{} within 5% of 52W High ₹{}",
                price.round_dp(2),
                record.high_52w.round_dp(2)
            ),
            actual_value: price,
            threshold: w52_threshold,
            passed: w52_passed,
        });
        evaluable_weight += dec!(15);
        if w52_passed {
            passed_weight += dec!(15);
        }
    }

    // Weighted score: passed share of the evaluable detectors, exact decimal
    // math (operands ≤ 100, so the division cannot overflow).
    let score = if evaluable_weight > Decimal::ZERO {
        (passed_weight * Decimal::from(100) / evaluable_weight).round_dp(2)
    } else {
        Decimal::ZERO
    };

    // Route selection precedence: squeeze breakout > trend retest > 52W watch.
    let setup: &'static str = if squeeze_evaluable && squeeze_passed {
        SETUP_BB_SQUEEZE_BREAKOUT
    } else if trend_passed && rvol_passed {
        SETUP_TREND_RETEST
    } else if w52_passed {
        SETUP_BREAKOUT_WATCH
    } else {
        SETUP_UNQUALIFIED
    };

    let (trigger_level, invalidation_level) = if setup == SETUP_BB_SQUEEZE_BREAKOUT {
        (record.bb_upper, record.bb_lower)
    } else if setup == SETUP_TREND_RETEST {
        let fallback = if record.low_52w != Decimal::ZERO {
            record.low_52w
        } else {
            price
        };
        let invalidation = smallest_nonzero(&[record.day_low, record.sma_50]).unwrap_or(fallback);
        (record.sma_200, invalidation)
    } else if setup == SETUP_BREAKOUT_WATCH {
        (record.high_52w, record.low_52w)
    } else {
        let invalidation = if record.day_low != Decimal::ZERO {
            record.day_low
        } else {
            price
        };
        (price, invalidation)
    };

    ScannerResult {
        symbol: record.symbol.clone(),
        setup_type: setup.to_string(),
        score,
        trigger_level,
        invalidation_level,
        explain_steps,
    }
}

/// Scan every row and rank the verdicts by score descending; the sort is
/// stable, so equal scores keep their input order.
pub fn scan_universe(records: &[ScreenerEquityRecord]) -> Vec<ScannerResult> {
    let mut results: Vec<ScannerResult> = records.iter().map(scan_row).collect();
    results.sort_by_key(|r| std::cmp::Reverse(r.score));
    results
}
