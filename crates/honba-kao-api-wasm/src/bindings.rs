//! Web Worker JS bridge for `honba-kao-api-wasm`.
//!
//! Gated behind the `js` feature (`wasm-bindgen`). Exposes the pure compute
//! engine as `#[wasm_bindgen]` exports consumable from JS/TS — for example from
//! a browser web worker that offloads option pricing, Greeks, margin, and
//! risk calculations off the main UI thread.
//!
//! All structured data crosses the wasm/JS boundary as JSON strings, so no
//! complex type marshaling is required. JS callers `JSON.parse` the results and
//! `JSON.stringify` the inputs.
//
//! #63: `from_json` now returns a `Result` instead of panicking on malformed
//! input, so an invalid JSON payload can never abort WASM linear memory.

use serde::de::DeserializeOwned;
use serde::Serialize;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use wasm_bindgen::prelude::*;
use std::collections::HashMap;

use crate::analytics::{
    calculate_cointegration, calculate_drawdown, calculate_performance_ratios,
    calculate_var_and_cvar,
};
use crate::indicators::{
    calculate_atr, calculate_bollinger_bands, calculate_ema, calculate_rsi, calculate_sma,
    calculate_vwap,
};
use crate::margin::{
    calculate_collateral_value, calculate_fno_margin, calculate_kelly_criterion,
    calculate_liquidation_buffer, calculate_margin_utilization, calculate_position_sizing,
    calculate_volatility_target_size, get_standard_haircut, AssetClass,
};
use crate::pairing::{pair_fills_fifo, pair_fills_lifo, Fill};
use crate::payoff::{Leg, PayoffConfig, PayoffEngine};
use crate::pricing::{
    black_scholes_call, black_scholes_put, calculate_greeks, calculate_implied_volatility,
    discount_factor, future_value, interpolate_yield, present_value,
};
use crate::replay::{simulate_fill_price, FillModel, ReplayClock};
use crate::screener::{evaluate_predicate, sort_indices, Predicate};

fn to_json<T: Serialize + ?Sized>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
}

/// Parse JSON, returning a descriptive error string instead of panicking
/// (#63). An unhandled panic in `wasm32-unknown-unknown` aborts WASM linear
/// memory and crashes the browser worker thread.
fn from_json<T: DeserializeOwned>(json: &str) -> Result<T, String> {
    serde_json::from_str(json).map_err(|e| format!("invalid JSON input: {e}"))
}

/// Convert a rust_decimal value to an f64 for the JS boundary (JS numbers are
/// IEEE-754 doubles). Internal compute remains exact Decimal.
fn dec_to_f64(d: Decimal) -> f64 {
    d.to_f64().unwrap_or(0.0)
}

/// Convert an f64 from the JS boundary into Decimal for internal compute.
fn f64_to_dec(x: f64) -> Decimal {
    Decimal::from_f64(x).unwrap_or(Decimal::ZERO)
}

/// Run a fallible JSON computation, emitting `{"error": "..."}` on failure.
fn json_or_error<T: Serialize, F: FnOnce() -> Result<T, String>>(f: F) -> String {
    match f() {
        Ok(value) => to_json(&value),
        Err(e) => to_json(&serde_json::json!({ "error": e })),
    }
}

/// ---- Options Pricing -------------------------------------------------------

#[wasm_bindgen]
pub fn bs_call(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> String {
    to_json(&serde_json::json!({ "price": black_scholes_call(s, k, t, r, sigma) }))
}

#[wasm_bindgen]
pub fn bs_put(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> String {
    to_json(&serde_json::json!({ "price": black_scholes_put(s, k, t, r, sigma) }))
}

#[wasm_bindgen]
pub fn greeks(s: f64, k: f64, t: f64, r: f64, sigma: f64, is_call: bool) -> String {
    to_json(&calculate_greeks(s, k, t, r, sigma, is_call))
}

#[wasm_bindgen]
pub fn implied_volatility(
    price: f64,
    s: f64,
    k: f64,
    t: f64,
    r: f64,
    is_call: bool,
) -> String {
    let iv = calculate_implied_volatility(price, s, k, t, r, is_call);
    to_json(&serde_json::json!({ "iv": iv }))
}

/// ---- Yield / Term Structure ------------------------------------------------

#[wasm_bindgen]
pub fn bs_discount_factor(rate: f64, time_years: f64) -> f64 {
    discount_factor(rate, time_years)
}

#[wasm_bindgen]
pub fn bs_present_value(cash_flow: f64, rate: f64, time_years: f64) -> f64 {
    present_value(cash_flow, rate, time_years)
}

#[wasm_bindgen]
pub fn bs_future_value(cash_flow: f64, rate: f64, time_years: f64) -> f64 {
    future_value(cash_flow, rate, time_years)
}

#[wasm_bindgen]
pub fn bs_interpolate_yield(curve_json: &str, target_tenor_months: f64) -> f64 {
    match from_json::<Vec<crate::pricing::term_structure::YieldPoint>>(curve_json) {
        Ok(curve) => interpolate_yield(&curve, target_tenor_months),
        Err(_) => 0.0,
    }
}

/// ---- Analytics -------------------------------------------------------------

#[wasm_bindgen]
pub fn analytics_drawdown(equity_curve_json: &str) -> String {
    json_or_error(|| {
        let curve = from_json::<Vec<f64>>(equity_curve_json)?;
        Ok(calculate_drawdown(&curve))
    })
}

#[wasm_bindgen]
pub fn analytics_var_cvar(returns_json: &str) -> String {
    json_or_error(|| {
        let returns = from_json::<Vec<f64>>(returns_json)?;
        Ok(calculate_var_and_cvar(&returns))
    })
}

#[wasm_bindgen]
pub fn analytics_ratios(
    returns_json: &str,
    trades_pnl_json: &str,
    max_drawdown_pct: f64,
    annual_risk_free_rate: f64,
) -> String {
    json_or_error(|| {
        let returns = from_json::<Vec<f64>>(returns_json)?;
        let pnl = from_json::<Vec<f64>>(trades_pnl_json)?;
        Ok(calculate_performance_ratios(
            &returns,
            &pnl,
            max_drawdown_pct,
            annual_risk_free_rate,
        ))
    })
}

#[wasm_bindgen]
pub fn analytics_cointegration(y_json: &str, x_json: &str) -> String {
    json_or_error(|| {
        let y = from_json::<Vec<f64>>(y_json)?;
        let x = from_json::<Vec<f64>>(x_json)?;
        match calculate_cointegration(&y, &x) {
            Some(r) => Ok(r),
            None => Err("no cointegration".into()),
        }
    })
}

/// ---- Margin & Risk ---------------------------------------------------------

#[wasm_bindgen]
pub fn margin_haircut(asset_class: &str) -> f64 {
    let class = match asset_class {
        "Cash" => AssetClass::Cash,
        "GovernmentBonds" => AssetClass::GovernmentBonds,
        "LargeCapEquity" => AssetClass::LargeCapEquity,
        "MidCapEquity" => AssetClass::MidCapEquity,
        "SmallCapEquity" => AssetClass::SmallCapEquity,
        "Commodities" => AssetClass::Commodities,
        "Crypto" => AssetClass::Crypto,
        _ => AssetClass::Cash,
    };
    dec_to_f64(get_standard_haircut(class))
}

#[wasm_bindgen]
pub fn margin_collateral_value(balance: f64, price: f64, haircut_pct: f64) -> f64 {
    dec_to_f64(calculate_collateral_value(
        f64_to_dec(balance),
        f64_to_dec(price),
        f64_to_dec(haircut_pct),
    ))
}

#[wasm_bindgen]
pub fn margin_fno(notional: f64, is_spread: bool) -> String {
    to_json(&calculate_fno_margin(f64_to_dec(notional), is_spread))
}

#[wasm_bindgen]
pub fn margin_kelly(win_rate: f64, win_loss_ratio: f64) -> f64 {
    dec_to_f64(calculate_kelly_criterion(
        f64_to_dec(win_rate),
        f64_to_dec(win_loss_ratio),
    ))
}

#[wasm_bindgen]
pub fn margin_vol_target(
    portfolio_value: f64,
    target_vol_annual: f64,
    asset_vol_annual: f64,
) -> f64 {
    dec_to_f64(calculate_volatility_target_size(
        f64_to_dec(portfolio_value),
        f64_to_dec(target_vol_annual),
        f64_to_dec(asset_vol_annual),
    ))
}

#[wasm_bindgen]
pub fn margin_position_sizing(
    portfolio_value: f64,
    win_rate: f64,
    win_loss_ratio: f64,
    target_vol_annual: f64,
    asset_vol_annual: f64,
) -> String {
    to_json(&calculate_position_sizing(
        f64_to_dec(portfolio_value),
        f64_to_dec(win_rate),
        f64_to_dec(win_loss_ratio),
        f64_to_dec(target_vol_annual),
        f64_to_dec(asset_vol_annual),
    ))
}

#[wasm_bindgen]
pub fn margin_liquidation_buffer(
    entry_price: f64,
    position_size: f64,
    collateral: f64,
    maintenance_margin_rate: f64,
    is_long: bool,
) -> String {
    to_json(&calculate_liquidation_buffer(
        f64_to_dec(entry_price),
        f64_to_dec(position_size),
        f64_to_dec(collateral),
        f64_to_dec(maintenance_margin_rate),
        is_long,
    ))
}

#[wasm_bindgen]
pub fn margin_utilization(total_collateral: f64, margin_used: f64) -> String {
    to_json(&calculate_margin_utilization(
        f64_to_dec(total_collateral),
        f64_to_dec(margin_used),
    ))
}

/// ---- Execution Pairing -----------------------------------------------------

#[wasm_bindgen]
pub fn pairing_fifo(fills_json: &str) -> String {
    json_or_error(|| {
        let fills = from_json::<Vec<Fill>>(fills_json)?;
        Ok(pair_fills_fifo(&fills))
    })
}

#[wasm_bindgen]
pub fn pairing_lifo(fills_json: &str) -> String {
    json_or_error(|| {
        let fills = from_json::<Vec<Fill>>(fills_json)?;
        Ok(pair_fills_lifo(&fills))
    })
}

/// ---- Technical Indicators --------------------------------------------------

#[wasm_bindgen]
pub fn indicator_sma(prices_json: &str, period: usize) -> String {
    json_or_error(|| {
        let prices = from_json::<Vec<f64>>(prices_json)?;
        Ok(calculate_sma(&prices, period))
    })
}

#[wasm_bindgen]
pub fn indicator_ema(prices_json: &str, period: usize) -> String {
    json_or_error(|| {
        let prices = from_json::<Vec<f64>>(prices_json)?;
        Ok(calculate_ema(&prices, period))
    })
}

#[wasm_bindgen]
pub fn indicator_rsi(prices_json: &str, period: usize) -> String {
    json_or_error(|| {
        let prices = from_json::<Vec<f64>>(prices_json)?;
        Ok(calculate_rsi(&prices, period))
    })
}

#[wasm_bindgen]
pub fn indicator_roc(prices_json: &str, period: usize) -> String {
    json_or_error(|| {
        let prices = from_json::<Vec<f64>>(prices_json)?;
        Ok(crate::indicators::momentum::calculate_roc(&prices, period))
    })
}

#[wasm_bindgen]
pub fn indicator_vwap(prices_json: &str, volumes_json: &str) -> String {
    json_or_error(|| {
        let prices = from_json::<Vec<f64>>(prices_json)?;
        let volumes = from_json::<Vec<f64>>(volumes_json)?;
        Ok(calculate_vwap(&prices, &volumes))
    })
}

#[wasm_bindgen]
pub fn indicator_atr(
    highs_json: &str,
    lows_json: &str,
    closes_json: &str,
    period: usize,
) -> String {
    json_or_error(|| {
        let highs = from_json::<Vec<f64>>(highs_json)?;
        let lows = from_json::<Vec<f64>>(lows_json)?;
        let closes = from_json::<Vec<f64>>(closes_json)?;
        Ok(calculate_atr(&highs, &lows, &closes, period))
    })
}

#[wasm_bindgen]
pub fn indicator_bollinger(
    prices_json: &str,
    period: usize,
    num_std: f64,
) -> String {
    json_or_error(|| {
        let prices = from_json::<Vec<f64>>(prices_json)?;
        Ok(calculate_bollinger_bands(&prices, period, num_std))
    })
}

/// ---- Replay / Simulation ---------------------------------------------------

#[wasm_bindgen]
pub fn replay_clock_new(start_timestamp: i64, speed_multiplier: f64) -> String {
    let clock = ReplayClock::new(start_timestamp, speed_multiplier);
    to_json(&serde_json::json!({
        "current_timestamp": clock.current_timestamp,
        "speed_multiplier": clock.speed_multiplier,
        "is_running": clock.is_running,
        "step_ms": clock.step_ms,
    }))
}

#[wasm_bindgen]
pub fn replay_fill(
    order_price: Option<f64>,
    market_price: f64,
    is_buy: bool,
    model: &str,
) -> f64 {
    let fill_model = match model {
        "Immediate" => FillModel::Immediate,
        "NextOpen" => FillModel::NextOpen,
        _ if model.starts_with("SlippageBps") => {
            let bps: u32 = model
                .strip_prefix("SlippageBps")
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
            FillModel::SlippageBps(bps)
        }
        _ => FillModel::Immediate,
    };
    dec_to_f64(simulate_fill_price(
        order_price.map(f64_to_dec),
        f64_to_dec(market_price),
        is_buy,
        fill_model,
    ))
}

/// ---- Multi-Leg Payoff ------------------------------------------------------

#[wasm_bindgen]
pub fn payoff_evaluate(legs_json: &str) -> String {
    json_or_error(|| {
        let legs = from_json::<Vec<Leg>>(legs_json)?;
        let engine = PayoffEngine::new(legs);
        Ok(engine.evaluate())
    })
}

#[wasm_bindgen]
pub fn payoff_evaluate_config(legs_json: &str, config_json: &str) -> String {
    json_or_error(|| {
        let legs = from_json::<Vec<Leg>>(legs_json)?;
        let config = from_json::<PayoffConfig>(config_json)?;
        let engine = PayoffEngine::with_config(legs, config);
        Ok(engine.evaluate())
    })
}

/// ---- Screener --------------------------------------------------------------

/// #63: expose the screener predicate engine to web workers.
#[wasm_bindgen]
pub fn screener_evaluate(
    predicate_json: &str,
    num_fields_json: &str,
    text_fields_json: &str,
) -> String {
    json_or_error(|| {
        let pred = from_json::<Predicate>(predicate_json)?;
        let num_fields: HashMap<String, Decimal> = from_json(num_fields_json)?;
        let text_fields: HashMap<String, String> = from_json(text_fields_json)?;
        Ok(serde_json::json!({
            "result": evaluate_predicate(&pred, &num_fields, &text_fields)
        }))
    })
}

/// #63: expose screener sorting to web workers. Returns the ascending sort
/// order of indices for the supplied numeric values.
#[wasm_bindgen]
pub fn screener_sort(len: usize, values_json: &str) -> String {
    json_or_error(|| {
        let values = from_json::<Vec<f64>>(values_json)?;
        let n = values.len().min(len);
        let indices = sort_indices(n, |a, b| {
            values
                .get(a)
                .and_then(|va| values.get(b).map(|vb| va.partial_cmp(vb)))
                .flatten()
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(serde_json::json!({ "indices": indices }))
    })
}
