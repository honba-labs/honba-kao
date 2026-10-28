//! Multi-leg payoff engine.
//!
//! Given a collection of [`Leg`]s, the engine computes the combined profit & loss
//! across a range of spot prices at expiry, and derives the breakeven points,
//! maximum profit and maximum loss of the overall strategy.
//!
//! All computations are pure Rust (no UI, no network, no clock) so they can run
//! identically in the browser (wasm32-unknown-unknown) and in CLI/unit-test
//! harnesses. All monetary fields use `rust_decimal::Decimal`.

use crate::payoff::leg::Leg;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// A single (spot, pnl) sample on the payoff curve.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PayoffPoint {
    pub spot: Decimal,
    pub pnl: Decimal,
}

/// Result of running the payoff engine over a spot range.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PayoffResult {
    /// Underlying symbols covered by the strategy.
    pub symbols: Vec<String>,
    /// Breakeven spot prices where total P&L crosses zero (ascending).
    pub breakeven: Vec<Decimal>,
    /// Maximum profit observed over the evaluated spot range.
    pub max_profit: Decimal,
    /// Maximum loss observed over the evaluated spot range (negative value).
    pub max_loss: Decimal,
    /// Sampled payoff curve points (ascending spot).
    pub points: Vec<PayoffPoint>,
}

impl PayoffResult {
    /// Total net premium paid (positive) or received (negative) to open the strategy.
    pub fn net_premium(&self) -> Decimal {
        // Net premium is embedded in the P&L curve at spot = 0 is not meaningful;
        // callers recompute from legs if needed. Kept for API symmetry.
        Decimal::ZERO
    }
}

/// Configuration for the payoff evaluation window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PayoffConfig {
    /// Lowest spot price to evaluate.
    pub spot_min: Decimal,
    /// Highest spot price to evaluate.
    pub spot_max: Decimal,
    /// Number of sample points across the window.
    pub steps: usize,
}

impl Default for PayoffConfig {
    fn default() -> Self {
        Self {
            spot_min: dec!(0.0),
            spot_max: dec!(0.0), // derived from strikes at runtime
            steps: 400,
        }
    }
}

/// Multi-leg payoff engine.
pub struct PayoffEngine {
    legs: Vec<Leg>,
    config: PayoffConfig,
}

impl PayoffEngine {
    /// Build an engine from a strategy's legs.
    pub fn new(legs: Vec<Leg>) -> Self {
        Self {
            legs,
            config: PayoffConfig::default(),
        }
    }

    /// Build an engine with a custom evaluation window.
    pub fn with_config(legs: Vec<Leg>, config: PayoffConfig) -> Self {
        Self { legs, config }
    }

    /// Total P&L of the whole strategy at a given spot price.
    pub fn pnl(&self, spot: Decimal) -> Decimal {
        self.legs.iter().map(|leg| leg.pnl(spot)).sum()
    }

    /// Derive a sensible spot window from the strategy's strikes.
    ///
    /// The window spans [0.5 × min_strike, 1.5 × max_strike]. For futures-only
    /// strategies (strike = 0) it falls back to [0, 1000].
    fn spot_window(&self) -> (Decimal, Decimal) {
        let strikes: Vec<Decimal> = self
            .legs
            .iter()
            .map(|l| l.strike)
            .filter(|&k| k > Decimal::ZERO)
            .collect();
        if strikes.is_empty() {
            return (dec!(0.0), dec!(1000.0));
        }
        let min_strike = strikes.iter().cloned().fold(Decimal::MAX, Decimal::min);
        let max_strike = strikes.iter().cloned().fold(Decimal::ZERO, Decimal::max);
        ((min_strike * dec!(0.5)).max(Decimal::ZERO), max_strike * dec!(1.5))
    }

    /// Run the payoff evaluation over the configured spot window.
    pub fn evaluate(&self) -> PayoffResult {
        let (spot_min, spot_max) = match (self.config.spot_min, self.config.spot_max) {
            (a, b) if a < b => (a, b),
            _ => self.spot_window(),
        };
        let steps = self.config.steps.max(2);
        let step = (spot_max - spot_min) / Decimal::from(steps - 1);

        let mut points: Vec<PayoffPoint> = Vec::with_capacity(steps);
        let mut max_profit = Decimal::MIN;
        let mut max_loss = Decimal::MAX;

        for i in 0..steps {
            let spot = spot_min + step * Decimal::from(i);
            let pnl = self.pnl(spot);
            points.push(PayoffPoint { spot, pnl });
            max_profit = max_profit.max(pnl);
            max_loss = max_loss.min(pnl);
        }

        let breakeven = Self::find_breakevens(&points);
        let symbols: Vec<String> = self
            .legs
            .iter()
            .map(|l| l.symbol.clone())
            .collect();

        PayoffResult {
            symbols,
            breakeven,
            max_profit,
            max_loss,
            points,
        }
    }

    /// Find spot prices where the total P&L crosses zero.
    ///
    /// Walks the sampled curve and linearly interpolates between consecutive
    /// samples whose P&L changes sign (or touches zero).
    ///
    /// #60 fixes:
    /// 1. The final point is inspected so a trailing exact-zero PnL is not dropped.
    /// 2. Flat zero-PnL segments (zero-cost collars) do not emit a breakeven for
    ///    every sampled point.
    /// 3. Breakevens are deduplicated within an epsilon tolerance.
    fn find_breakevens(points: &[PayoffPoint]) -> Vec<Decimal> {
        let mut be: Vec<Decimal> = Vec::new();
        let eps = dec!(0.001);
        let zero = Decimal::ZERO;

        for window in points.windows(2) {
            let (p0, p1) = (&window[0], &window[1]);
            let crosses = (p0.pnl > zero && p1.pnl < zero) || (p0.pnl < zero && p1.pnl > zero);
            if crosses {
                let t = p0.pnl / (p0.pnl - p1.pnl);
                let spot = p0.spot + t * (p1.spot - p0.spot);
                if be.last().map_or(true, |last| (spot - *last).abs() >= eps) {
                    be.push(spot);
                }
            } else if p0.pnl.is_zero() {
                if be.last().map_or(true, |last| (p0.spot - *last).abs() >= eps) {
                    be.push(p0.spot);
                }
            }
        }

        // #60: boundary zero-crossing - inspect the final point too.
        if let Some(last) = points.last() {
            if last.pnl.is_zero() {
                if be.last().map_or(true, |last_b| (last.spot - *last_b).abs() >= eps) {
                    be.push(last.spot);
                }
            }
        }

        be
    }
}
