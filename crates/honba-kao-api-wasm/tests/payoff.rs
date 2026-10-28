//! Integration tests for `honba-kao-api-wasm::payoff`.
//!
//! Covers leg modeling, multi-leg P&L curve, breakeven points, max profit,
//! max loss, and custom spot windows. All monetary math uses `Decimal` (#54).

use honba_api_wasm::payoff::{leg::*, PayoffConfig, PayoffEngine};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

fn d(x: f64) -> Decimal {
    Decimal::from_f64(x).unwrap()
}

fn long_call(strike: f64, premium: f64, qty: f64) -> Leg {
    Leg {
        symbol: "TEST-CALL".into(),
        option_type: OptionType::Call,
        strike: d(strike),
        premium: d(premium),
        quantity: d(qty),
        side: Side::Long,
    }
}

fn short_put(strike: f64, premium: f64, qty: f64) -> Leg {
    Leg {
        symbol: "TEST-PUT".into(),
        option_type: OptionType::Put,
        strike: d(strike),
        premium: d(premium),
        quantity: d(qty),
        side: Side::Short,
    }
}

#[test]
fn long_call_pnl_shape() {
    let leg = long_call(100.0, 5.0, 1.0);
    assert_eq!(leg.pnl(d(80.0)), d(-5.0)); // below strike: lose premium
    assert_eq!(leg.pnl(d(100.0)), d(-5.0)); // at strike: lose premium
    assert_eq!(leg.pnl(d(110.0)), d(5.0)); // above: intrinsic - premium
}

#[test]
fn short_put_pnl_shape() {
    let leg = short_put(100.0, 5.0, 1.0);
    assert_eq!(leg.pnl(d(80.0)), d(-15.0)); // below strike: premium - intrinsic
    assert_eq!(leg.pnl(d(120.0)), d(5.0)); // above: keep premium
}

#[test]
fn straddle_breakevens_and_extrema() {
    let legs = vec![
        long_call(100.0, 5.0, 1.0),
        Leg {
            symbol: "TEST-PUT".into(),
            option_type: OptionType::Put,
            strike: d(100.0),
            premium: d(5.0),
            quantity: d(1.0),
            side: Side::Long,
        },
    ];
    let engine = PayoffEngine::new(legs);
    let result = engine.evaluate();
    // net premium 10 => max loss -10
    assert!(result.max_loss < d(-9.0) && result.max_loss >= d(-10.01));
    assert!(result.breakeven.len() >= 2);
    assert!((result.breakeven[0] - d(90.0)).abs() < d(1.0));
    assert!((result.breakeven[1] - d(110.0)).abs() < d(1.0));
    assert!(result.max_profit > d(0.0));
    assert!(engine.pnl(d(200.0)) > d(0.0));
}

#[test]
fn futures_only_fallback_window() {
    let legs = vec![Leg {
        symbol: "TEST-FUT".into(),
        option_type: OptionType::Call,
        strike: d(0.0),
        premium: d(0.0),
        quantity: d(1.0),
        side: Side::Long,
    }];
    let engine = PayoffEngine::new(legs);
    let result = engine.evaluate();
    assert!(!result.points.is_empty());
    assert!(result.points.first().unwrap().spot >= d(0.0));
}

#[test]
fn config_window_respected() {
    let leg = long_call(100.0, 5.0, 1.0);
    let config = PayoffConfig { spot_min: d(90.0), spot_max: d(110.0), steps: 5 };
    let engine = PayoffEngine::with_config(vec![leg], config);
    let result = engine.evaluate();
    assert_eq!(result.points.len(), 5);
    assert_eq!(result.points.first().unwrap().spot, d(90.0));
    assert_eq!(result.points.last().unwrap().spot, d(110.0));
}

#[test]
fn payoff_result_symbols() {
    let legs = vec![
        long_call(100.0, 5.0, 1.0),
        Leg {
            symbol: "OTHER".into(),
            option_type: OptionType::Put,
            strike: d(90.0),
            premium: d(2.0),
            quantity: d(1.0),
            side: Side::Short,
        },
    ];
    let engine = PayoffEngine::new(legs);
    let result = engine.evaluate();
    assert_eq!(result.symbols.len(), 2);
    assert!(result.symbols.iter().any(|s| s == "TEST-CALL"));
    assert!(result.symbols.iter().any(|s| s == "OTHER"));
}
