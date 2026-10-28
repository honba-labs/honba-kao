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

use serde::Serialize;
use wasm_bindgen::prelude::*;

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

fn to_json<T: Serialize + ?Sized>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
}

fn from_json<T: serde::de::DeserializeOwned>(json: &str) -> T {
    serde_json::from_str(json).unwrap_or_else(|_| {
        panic!("invalid JSON input: {}", json.chars().take(120).collect::<String>())
    })
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
    let curve: Vec<crate::pricing::term_structure::YieldPoint> = from_json(curve_json);
    interpolate_yield(&curve, target_tenor_months)
}

/// ---- Analytics -------------------------------------------------------------

#[wasm_bindgen]
pub fn analytics_drawdown(equity_curve_json: &str) -> String {
    let curve: Vec<f64> = from_json(equity_curve_json);
    to_json(&calculate_drawdown(&curve))
}

#[wasm_bindgen]
pub fn analytics_var_cvar(returns_json: &str) -> String {
    let returns: Vec<f64> = from_json(returns_json);
    to_json(&calculate_var_and_cvar(&returns))
}

#[wasm_bindgen]
pub fn analytics_ratios(
    returns_json: &str,
    trades_pnl_json: &str,
    max_drawdown_pct: f64,
    annual_risk_free_rate: f64,
) -> String {
    let returns: Vec<f64> = from_json(returns_json);
    let pnl: Vec<f64> = from_json(trades_pnl_json);
    to_json(&calculate_performance_ratios(
        &returns,
        &pnl,
        max_drawdown_pct,
        annual_risk_free_rate,
    ))
}

#[wasm_bindgen]
pub fn analytics_cointegration(y_json: &str, x_json: &str) -> String {
    let y: Vec<f64> = from_json(y_json);
    let x: Vec<f64> = from_json(x_json);
    match calculate_cointegration(&y, &x) {
        Some(r) => to_json(&r),
        None => "null".to_string(),
    }
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
    get_standard_haircut(class)
}

#[wasm_bindgen]
pub fn margin_collateral_value(balance: f64, price: f64, haircut_pct: f64) -> f64 {
    calculate_collateral_value(balance, price, haircut_pct)
}

#[wasm_bindgen]
pub fn margin_fno(notional: f64, is_spread: bool) -> String {
    to_json(&calculate_fno_margin(notional, is_spread))
}

#[wasm_bindgen]
pub fn margin_kelly(win_rate: f64, win_loss_ratio: f64) -> f64 {
    calculate_kelly_criterion(win_rate, win_loss_ratio)
}

#[wasm_bindgen]
pub fn margin_vol_target(
    portfolio_value: f64,
    target_vol_annual: f64,
    asset_vol_annual: f64,
) -> f64 {
    calculate_volatility_target_size(portfolio_value, target_vol_annual, asset_vol_annual)
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
        portfolio_value,
        win_rate,
        win_loss_ratio,
        target_vol_annual,
        asset_vol_annual,
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
        entry_price,
        position_size,
        collateral,
        maintenance_margin_rate,
        is_long,
    ))
}

#[wasm_bindgen]
pub fn margin_utilization(total_collateral: f64, margin_used: f64) -> String {
    to_json(&calculate_margin_utilization(total_collateral, margin_used))
}

/// ---- Execution Pairing -----------------------------------------------------

#[wasm_bindgen]
pub fn pairing_fifo(fills_json: &str) -> String {
    let fills: Vec<Fill> = from_json(fills_json);
    to_json(&pair_fills_fifo(&fills))
}

#[wasm_bindgen]
pub fn pairing_lifo(fills_json: &str) -> String {
    let fills: Vec<Fill> = from_json(fills_json);
    to_json(&pair_fills_lifo(&fills))
}

/// ---- Technical Indicators --------------------------------------------------

#[wasm_bindgen]
pub fn indicator_sma(prices_json: &str, period: usize) -> String {
    let prices: Vec<f64> = from_json(prices_json);
    to_json(&calculate_sma(&prices, period))
}

#[wasm_bindgen]
pub fn indicator_ema(prices_json: &str, period: usize) -> String {
    let prices: Vec<f64> = from_json(prices_json);
    to_json(&calculate_ema(&prices, period))
}

#[wasm_bindgen]
pub fn indicator_rsi(prices_json: &str, period: usize) -> String {
    let prices: Vec<f64> = from_json(prices_json);
    to_json(&calculate_rsi(&prices, period))
}

#[wasm_bindgen]
pub fn indicator_roc(prices_json: &str, period: usize) -> String {
    let prices: Vec<f64> = from_json(prices_json);
    to_json(&crate::indicators::momentum::calculate_roc(&prices, period))
}

#[wasm_bindgen]
pub fn indicator_vwap(prices_json: &str, volumes_json: &str) -> String {
    let prices: Vec<f64> = from_json(prices_json);
    let volumes: Vec<f64> = from_json(volumes_json);
    to_json(&calculate_vwap(&prices, &volumes))
}

#[wasm_bindgen]
pub fn indicator_atr(
    highs_json: &str,
    lows_json: &str,
    closes_json: &str,
    period: usize,
) -> String {
    let highs: Vec<f64> = from_json(highs_json);
    let lows: Vec<f64> = from_json(lows_json);
    let closes: Vec<f64> = from_json(closes_json);
    to_json(&calculate_atr(&highs, &lows, &closes, period))
}

#[wasm_bindgen]
pub fn indicator_bollinger(
    prices_json: &str,
    period: usize,
    num_std: f64,
) -> String {
    let prices: Vec<f64> = from_json(prices_json);
    to_json(&calculate_bollinger_bands(&prices, period, num_std))
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
    simulate_fill_price(order_price, market_price, is_buy, fill_model)
}

/// ---- Multi-Leg Payoff ------------------------------------------------------

#[wasm_bindgen]
pub fn payoff_evaluate(legs_json: &str) -> String {
    let legs: Vec<Leg> = from_json(legs_json);
    let engine = PayoffEngine::new(legs);
    to_json(&engine.evaluate())
}

#[wasm_bindgen]
pub fn payoff_evaluate_config(legs_json: &str, config_json: &str) -> String {
    let legs: Vec<Leg> = from_json(legs_json);
    let config: PayoffConfig = from_json(config_json);
    let engine = PayoffEngine::with_config(legs, config);
    to_json(&engine.evaluate())
}
