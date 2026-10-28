//! Integration tests for `honba-kao-api-wasm::payoff`.
//!
//! Covers leg modeling, multi-leg P&L curve, breakeven points, max profit,
//! max loss, and custom spot windows.

use honba_api_wasm::payoff::{leg::*, PayoffConfig, PayoffEngine};

fn long_call(strike: f64, premium: f64, qty: f64) -> Leg {
    Leg { symbol: "TEST-CALL".into(), option_type: OptionType::Call, strike, premium, quantity: qty, side: Side::Long }
}

fn short_put(strike: f64, premium: f64, qty: f64) -> Leg {
    Leg { symbol: "TEST-PUT".into(), option_type: OptionType::Put, strike, premium, quantity: qty, side: Side::Short }
}

#[test]
fn long_call_pnl_shape() {
    let leg = long_call(100.0, 5.0, 1.0);
    assert!((leg.pnl(80.0) + 5.0).abs() < 1e-9); // below strike: lose premium
    assert!((leg.pnl(100.0) + 5.0).abs() < 1e-9); // at strike: lose premium
    assert!((leg.pnl(110.0) - 5.0).abs() < 1e-9); // above: intrinsic - premium
}

#[test]
fn short_put_pnl_shape() {
    let leg = short_put(100.0, 5.0, 1.0);
    assert!((leg.pnl(80.0) + 15.0).abs() < 1e-9); // below strike: premium - intrinsic
    assert!((leg.pnl(120.0) - 5.0).abs() < 1e-9); // above: keep premium
}

#[test]
fn straddle_breakevens_and_extrema() {
    let legs = vec![
        long_call(100.0, 5.0, 1.0),
        Leg { symbol: "TEST-PUT".into(), option_type: OptionType::Put, strike: 100.0, premium: 5.0, quantity: 1.0, side: Side::Long },
    ];
    let engine = PayoffEngine::new(legs);
    let result = engine.evaluate();
    assert!(result.max_loss < -9.0 && result.max_loss >= -10.01); // net premium 10
    assert!(result.breakeven.len() >= 2);
    assert!((result.breakeven[0] - 90.0).abs() < 1.0);
    assert!((result.breakeven[1] - 110.0).abs() < 1.0);
    assert!(result.max_profit > 0.0);
    assert!(engine.pnl(200.0) > 0.0);
}

#[test]
fn futures_only_fallback_window() {
    let legs = vec![Leg { symbol: "TEST-FUT".into(), option_type: OptionType::Call, strike: 0.0, premium: 0.0, quantity: 1.0, side: Side::Long }];
    let engine = PayoffEngine::new(legs);
    let result = engine.evaluate();
    assert!(!result.points.is_empty());
    assert!(result.points.first().unwrap().spot >= 0.0);
}

#[test]
fn config_window_respected() {
    let leg = long_call(100.0, 5.0, 1.0);
    let config = PayoffConfig { spot_min: 90.0, spot_max: 110.0, steps: 5 };
    let engine = PayoffEngine::with_config(vec![leg], config);
    let result = engine.evaluate();
    assert_eq!(result.points.len(), 5);
    assert!((result.points.first().unwrap().spot - 90.0).abs() < 1e-9);
    assert!((result.points.last().unwrap().spot - 110.0).abs() < 1e-9);
}

#[test]
fn payoff_result_symbols() {
    let legs = vec![
        long_call(100.0, 5.0, 1.0),
        Leg { symbol: "OTHER".into(), option_type: OptionType::Put, strike: 90.0, premium: 2.0, quantity: 1.0, side: Side::Short },
    ];
    let engine = PayoffEngine::new(legs);
    let result = engine.evaluate();
    assert!(result.symbols.len() == 2);
    assert!(result.symbols.iter().any(|s| s == "TEST-CALL"));
    assert!(result.symbols.iter().any(|s| s == "OTHER"));
}
