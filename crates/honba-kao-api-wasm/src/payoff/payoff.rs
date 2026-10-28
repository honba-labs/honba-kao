//! Multi-leg payoff engine.
//!
//! Given a collection of [`Leg`]s, the engine computes the combined profit & loss
//! across a range of spot prices at expiry, and derives the breakeven points,
//! maximum profit and maximum loss of the overall strategy.
//!
//! All computations are pure Rust (no UI, no network, no clock) so they can run
//! identically in the browser (wasm32-unknown-unknown) and in CLI/unit-test
//! harnesses.

use crate::payoff::leg::Leg;
use serde::{Deserialize, Serialize};

/// A single (spot, pnl) sample on the payoff curve.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PayoffPoint {
    pub spot: f64,
    pub pnl: f64,
}

/// Result of running the payoff engine over a spot range.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PayoffResult {
    /// Underlying symbols covered by the strategy.
    pub symbols: Vec<String>,
    /// Breakeven spot prices where total P&L crosses zero (ascending).
    pub breakeven: Vec<f64>,
    /// Maximum profit observed over the evaluated spot range.
    pub max_profit: f64,
    /// Maximum loss observed over the evaluated spot range (negative value).
    pub max_loss: f64,
    /// Sampled payoff curve points (ascending spot).
    pub points: Vec<PayoffPoint>,
}

impl PayoffResult {
    /// Total net premium paid (positive) or received (negative) to open the strategy.
    pub fn net_premium(&self) -> f64 {
        // Net premium is embedded in the P&L curve at spot = 0 is not meaningful;
        // callers recompute from legs if needed. Kept for API symmetry.
        0.0
    }
}

/// Configuration for the payoff evaluation window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PayoffConfig {
    /// Lowest spot price to evaluate.
    pub spot_min: f64,
    /// Highest spot price to evaluate.
    pub spot_max: f64,
    /// Number of sample points across the window.
    pub steps: usize,
}

impl Default for PayoffConfig {
    fn default() -> Self {
        Self {
            spot_min: 0.0,
            spot_max: 0.0, // derived from strikes at runtime
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
    pub fn pnl(&self, spot: f64) -> f64 {
        self.legs.iter().map(|leg| leg.pnl(spot)).sum()
    }

    /// Derive a sensible spot window from the strategy's strikes.
    ///
    /// The window spans [0.5 × min_strike, 1.5 × max_strike]. For futures-only
    /// strategies (strike = 0) it falls back to [0, 1000].
    fn spot_window(&self) -> (f64, f64) {
        let strikes: Vec<f64> = self
            .legs
            .iter()
            .map(|l| l.strike)
            .filter(|&k| k > 0.0)
            .collect();
        if strikes.is_empty() {
            return (0.0, 1000.0);
        }
        let min_strike = strikes.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_strike = strikes.iter().cloned().fold(0.0, f64::max);
        ((min_strike * 0.5).max(0.0), max_strike * 1.5)
    }

    /// Run the payoff evaluation over the configured spot window.
    pub fn evaluate(&self) -> PayoffResult {
        let (spot_min, spot_max) = match (self.config.spot_min, self.config.spot_max) {
            (a, b) if a < b => (a, b),
            _ => self.spot_window(),
        };
        let steps = self.config.steps.max(2);
        let step = (spot_max - spot_min) / (steps - 1) as f64;

        let mut points: Vec<PayoffPoint> = Vec::with_capacity(steps);
        let mut max_profit = f64::NEG_INFINITY;
        let mut max_loss = f64::INFINITY;

        for i in 0..steps {
            let spot = spot_min + step * i as f64;
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
    fn find_breakevens(points: &[PayoffPoint]) -> Vec<f64> {
        let mut be: Vec<f64> = Vec::new();
        for window in points.windows(2) {
            let (p0, p1) = (&window[0], &window[1]);
            if (p0.pnl > 0.0 && p1.pnl < 0.0) || (p0.pnl < 0.0 && p1.pnl > 0.0) {
                // Linear interpolation of the zero crossing.
                let t = p0.pnl / (p0.pnl - p1.pnl);
                be.push(p0.spot + t * (p1.spot - p0.spot));
            } else if p0.pnl == 0.0 {
                be.push(p0.spot);
            }
        }
        if let Some(last) = points.last() {
            if last.pnl == 0.0 {
                be.push(last.spot);
            }
        }
        be
    }
}
