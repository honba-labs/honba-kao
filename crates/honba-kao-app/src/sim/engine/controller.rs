//! The replay controller (P7, doc §P7) — fe `replay-controller.ts` ported as
//! plain state with no signals and no browser timer: the app drives
//! [`ReplayController::advance`] / [`ReplayController::run_frame`] and
//! attaches reactivity in the store (`RwSignal<u64>` generation bump), per
//! the documented controller contract.
//!
//! Traceability:
//! - `init`/`set_*`/`play`/`pause`/`stop`/`reset`/`step`/`jump_to_bar`/
//!   `instant_run_to_end`/`advance`/`state` — fe `ReplayController` methods.
//! - the timer body (`ticksPerFrame` loop, `completed` on exhaustion) is
//!   fe's `startTimer` interval callback, expressed as [`run_frame`];
//! - `handleTradeExecution` (fill models, fees, round-trip construction) is
//!   [`ReplayController::execute_trade`], building `sim::pairing::RoundTrip`
//!   values so engines' trips read exactly like the journal pairs them.
//!
//! Honesty + deviation notes (pinned):
//! - The engine replays real bars only; the forming candle is derived from
//!   the current bar's phases, never fabricated between bars.
//! - fe drives a sandbox portfolio adapter (`portfolio-adapter.ts`, an
//!   external books service); kao's engine books are the state itself.
//!   `updatePortfolioMarkPrice`/`executeSimulatedTrade` have no kao
//!   counterpart, so fe's `onTrade` callback is likewise absorbed: the
//!   store derives reactions from [`state()`](ReplayController::state).
//! - fe's `getState` fabricates a *synthetic* close-phase tick (empty
//!   `timeStr`, the bar's own close) for the panel; kao keeps that verbatim
//!   and labels it in [`SimulationEngineState::current_tick`] docs — real
//!   ticks come from [`advance`](ReplayController::advance).
//! - fe's `l2` spread divisor is `bar.close || 1`; kao treats close ≤ 0 or
//!   non-finite as the absent case too, since only positive closes divide.
//! - fe's `onBarChange`/`notifyState` re-emissions become return values;
//!   `reset`/`jump_to_bar` no longer have a UI to re-emit into.
//! - fe's timers restart on `setSpeed`; kao has no timer — the store
//!   re-reads `interval_ms`/`ticks_per_frame` each frame.

use crate::sim::chart_data::ChartBar;
use crate::sim::engine::evaluator::{
    ConfluenceSnapshot, EvaluatedIndicators, ExecutedTrade, SignalEvaluation, TradeAction,
    compute_graph_indicators, evaluate_bar_signals,
};
use crate::sim::engine::{
    ActivePosition, DEFAULT_FILL_FEE, DEFAULT_L2_SLIPPAGE_BPS, FillModel, L2_HALF_SPREAD_FLOOR,
    L2_RANGE_FACTOR, ReplayConfig, ReplayStatus, Side, SimulatedTick, TickPhase, clamp_speed,
    clock_str,
};
use crate::sim::pairing::{
    Direction, RoundTrip, RoundTripExitReason, RoundTripLeg, RoundTripStatus,
};
use crate::sim::selection::StrategyGraphShape;

/// What one emitted tick carries — fe's `onTick` + `onBarChange` payloads,
/// folded into a return value (forming candle and "is there more").
#[derive(Clone, Debug, PartialEq)]
pub struct TickOutcome {
    /// The intra-bar tick (fe `SimulatedTick`).
    pub tick: SimulatedTick,
    /// The dynamic forming candle for the current bar, as fe's
    /// `onBarChange(index, [previous..., forming])` last element.
    pub forming_bar: ChartBar,
    /// fe `advanceTick`'s return — `false` once the index rolled past the
    /// last bar (the timer body then marks the replay completed).
    pub more: bool,
}

/// fe `SimulationEngineState`.
#[derive(Clone, Debug, PartialEq)]
pub struct SimulationEngineState {
    /// Status.
    pub status: ReplayStatus,
    /// Clamped speed.
    pub speed: f64,
    /// Index of the bar being replayed.
    pub current_bar_index: usize,
    /// `bars.length`.
    pub total_bars: usize,
    /// Synthetic close-phase view of the current bar — fe's `getState`
    /// fabrication, `time_str` empty. Real ticks come from `advance`.
    pub current_tick: Option<SimulatedTick>,
    /// Latest close-phase signal rows.
    pub active_signals: Vec<SignalEvaluation>,
    /// Latest confluence snapshot (if any bar closed).
    pub last_confluence: Option<ConfluenceSnapshot>,
    /// Confluence snapshots for bars that generated a trade, **latest
    /// first** (fe `unshift` order).
    pub confluence_history: Vec<ConfluenceSnapshot>,
    /// Closed round trips, **latest first**.
    pub round_trips: Vec<RoundTrip>,
    /// The open simulated position, if any.
    pub active_position: Option<ActivePosition>,
    /// Realized PnL, net of fees.
    pub realized_pnl: f64,
    /// Unrealized PnL at the current bar's close (0 while flat).
    pub unrealized_pnl: f64,
    /// `initial_capital + realized + unrealized`.
    pub equity: f64,
    /// fe default `100_000`.
    pub initial_capital: f64,
    /// fe default `acc-sandbox-primary`.
    pub selected_account_id: String,
    /// Multi-timeframe toggle (fe `setMultiTimeframe`).
    pub multi_timeframe: bool,
    /// Secondary timeframe (fe default `1h`).
    pub secondary_timeframe: String,
    /// fe `fillModel` string.
    pub fill_model: &'static str,
    /// User commission (fall back to the fee floor when 0).
    pub commission: f64,
    /// User slippage bps (fall back inside `l2`).
    pub slippage_bps: f64,
}

/// The replay controller — plain state, no signals (P7 contract).
///
/// `Clone` exists so tests can snapshot the whole book; the store's hot
/// path never clones it (mutations borrow in place through the write
/// guard).
#[derive(Clone, Debug)]
pub struct ReplayController {
    /// Bars, copied at `init`.
    bars: Vec<ChartBar>,
    /// The strategy graph (required to play).
    graph: Option<StrategyGraphShape>,
    /// Execution config (symbol/exchange/capital/fill model/speed…).
    config: ReplayConfig,
    /// Secondary timeframe (fe `1h` default).
    secondary_timeframe: String,
    /// Multi-timeframe toggle.
    multi_timeframe: bool,
    /// Pre-computed indicator series, refreshed at `init`.
    indicators: Option<EvaluatedIndicators>,
    /// Status.
    status: ReplayStatus,
    /// Index of the bar under replay.
    current_bar_index: usize,
    /// fe `currentTickPhase` (`0..=3`).
    current_tick_phase: u8,
    /// The most recent forming candle, set by every `advance` (fe emitted
    /// it through `onBarChange` per tick). Cleared by `init`/`reset`/
    /// `jumpToBar` — with it `None`, the visible slice falls back to the
    /// settled bars inclusive of the current index.
    forming: Option<ChartBar>,
    /// Latest close-phase signal rows.
    active_signals: Vec<SignalEvaluation>,
    /// Latest confluence.
    last_confluence: Option<ConfluenceSnapshot>,
    /// Trade-generating snapshots, latest first.
    confluence_history: Vec<ConfluenceSnapshot>,
    /// Closed trips, latest first.
    round_trips: Vec<RoundTrip>,
    /// The open position.
    active_position: Option<ActivePosition>,
    /// Realized PnL, net of fees.
    realized_pnl: f64,
}

impl Default for ReplayController {
    fn default() -> Self {
        Self::new(ReplayConfig::default())
    }
}

impl ReplayController {
    /// fe constructor.
    pub fn new(config: ReplayConfig) -> Self {
        Self {
            bars: Vec::new(),
            graph: None,
            config,
            secondary_timeframe: crate::sim::engine::DEFAULT_SECONDARY_TIMEFRAME.to_string(),
            multi_timeframe: false,
            indicators: None,
            status: ReplayStatus::Idle,
            current_bar_index: 0,
            current_tick_phase: 0,
            forming: None,
            active_signals: Vec::new(),
            last_confluence: None,
            confluence_history: Vec::new(),
            round_trips: Vec::new(),
            active_position: None,
            realized_pnl: 0.0,
        }
    }

    /// fe `init` — copies the dataset in, resets the books and pre-computes
    /// every indicator node over the bars.
    pub fn init(&mut self, bars: &[ChartBar], graph: &StrategyGraphShape, config: ReplayConfig) {
        self.stop();
        self.bars = bars.to_vec();
        self.graph = Some(graph.clone());
        self.config = config;

        self.current_bar_index = 0;
        self.current_tick_phase = 0;
        self.status = ReplayStatus::Idle;
        self.forming = None;
        self.active_signals = Vec::new();
        self.last_confluence = None;
        self.confluence_history = Vec::new();
        self.round_trips = Vec::new();
        self.active_position = None;
        self.realized_pnl = 0.0;

        self.indicators = Some(compute_graph_indicators(graph, bars));
    }

    /// fe `setSpeed` (`max(1, speed)` clamp, semantics preserved).
    pub fn set_speed(&mut self, speed: f64) {
        self.config.speed = clamp_speed(speed);
    }

    /// fe `setExecutionConfig` + `setAccountId`.
    pub fn set_execution_config(
        &mut self,
        fill_model: FillModel,
        commission: f64,
        slippage_bps: f64,
    ) {
        self.config.fill_model = fill_model;
        self.config.commission = commission;
        self.config.slippage_bps = slippage_bps;
    }

    /// fe `setMultiTimeframe(enabled, tf)`.
    pub fn set_multi_timeframe(&mut self, enabled: bool, timeframe: &str) {
        self.multi_timeframe = enabled;
        self.secondary_timeframe = timeframe.to_string();
    }

    /// fe `play` — resets when parked at the end, flips to playing and
    /// emits the first tick immediately (fe's no-latency behaviour).
    pub fn play(&mut self) {
        if self.bars.is_empty() || self.graph.is_none() {
            return;
        }
        if self.current_bar_index >= self.bars.len() - 1 {
            self.reset();
        }
        self.status = ReplayStatus::Playing;
        self.advance();
    }

    /// fe `pause`.
    pub fn pause(&mut self) {
        self.status = ReplayStatus::Paused;
    }

    /// fe `stop` — status only; the timer is the app's.
    pub fn stop(&mut self) {
        self.status = ReplayStatus::Idle;
    }

    /// fe `reset` — clears the books and signal state, parks at bar 0.
    pub fn reset(&mut self) {
        self.stop();
        self.current_bar_index = 0;
        self.current_tick_phase = 0;
        self.forming = None;
        self.active_position = None;
        self.realized_pnl = 0.0;
        self.confluence_history.clear();
        self.round_trips.clear();
        self.last_confluence = None;
        self.active_signals = Vec::new();
        self.status = ReplayStatus::Idle;
    }

    /// fe `step` — one tick, pausing if it was playing.
    pub fn step(&mut self) -> Option<TickOutcome> {
        if self.bars.is_empty() {
            return None;
        }
        if self.status == ReplayStatus::Playing {
            self.pause();
        }
        self.advance()
    }

    /// fe `jumpToBar` — bounds and evaluates at the close phase.
    pub fn jump_to_bar(&mut self, target_index: usize) {
        if self.bars.is_empty() {
            return;
        }
        self.current_bar_index = target_index.min(self.bars.len() - 1);
        self.current_tick_phase = 3;
        self.forming = None;
        self.evaluate_current_bar();
    }

    /// fe `instantRunToEnd` — sweep every bar at its close phase.
    pub fn instant_run_to_end(&mut self) {
        if self.bars.is_empty() || self.graph.is_none() || self.indicators.is_none() {
            return;
        }
        self.stop();
        self.current_bar_index = 0;
        self.current_tick_phase = 0;
        self.active_position = None;
        self.realized_pnl = 0.0;
        self.confluence_history.clear();
        self.round_trips.clear();

        for i in 0..self.bars.len() {
            self.current_bar_index = i;
            self.current_tick_phase = 3;
            self.evaluate_current_bar();
        }

        self.current_bar_index = self.bars.len() - 1;
        self.status = ReplayStatus::Completed;
    }

    /// fe `advanceTick` — emit one intra-bar tick (with its forming
    /// candle), evaluate on `close`, then roll the phase/bar.
    pub fn advance(&mut self) -> Option<TickOutcome> {
        if self.bars.is_empty() || self.current_bar_index >= self.bars.len() {
            return None;
        }

        let index = self.current_bar_index;
        let bar = self.bars[index];
        let is_bullish = bar.close >= bar.open;
        let phase = TickPhase::from_index(self.current_tick_phase);

        let tick_price = match phase {
            // fe: 0=open, 1=swing one, 2=swing two, 3=close; bullish bars
            // swing down first (low), bearish swing up first (high).
            TickPhase::Open => bar.open,
            TickPhase::Intra1 => {
                if is_bullish {
                    bar.low
                } else {
                    bar.high
                }
            }
            TickPhase::Intra2 => {
                if is_bullish {
                    bar.high
                } else {
                    bar.low
                }
            }
            TickPhase::Close => bar.close,
        };

        let tick = SimulatedTick {
            timestamp: bar.time,
            time_str: clock_str(bar.time),
            price: tick_price,
            open: bar.open,
            high: bar.high,
            low: bar.low,
            close: bar.close,
            volume: bar.volume,
            bar_index: index,
            phase,
        };

        // Dynamic forming candle, fe verbatim: the high only knows the
        // bar's own high once past the first swing; the low likewise
        // unlocks after the open phase; volume ramps `(phase+1)/4`.
        let current_high = bar
            .open
            .max(tick_price)
            .max(if self.current_tick_phase > 1 {
                bar.high
            } else {
                tick_price
            });
        let current_low = bar
            .open
            .min(tick_price)
            .min(if self.current_tick_phase > 0 {
                bar.low
            } else {
                tick_price
            });
        let forming_bar = ChartBar {
            time: bar.time,
            open: bar.open,
            high: current_high,
            low: current_low,
            close: tick_price,
            volume: (bar.volume * (phase.index() as f64 + 1.0) / 4.0)
                .round()
                .max(1.0),
        };

        // Strategy runs on the candle close.
        if self.current_tick_phase == 3 {
            self.evaluate_current_bar();
        }

        // fe emitted this through `onBarChange(index, visibleBars)` — the
        // slice accessor reads it back instead of a callback.
        self.forming = Some(forming_bar);

        self.current_tick_phase += 1;
        let mut more = true;
        if self.current_tick_phase > 3 {
            self.current_tick_phase = 0;
            self.current_bar_index += 1;
            if self.current_bar_index >= self.bars.len() {
                more = false;
            }
        }

        Some(TickOutcome {
            tick,
            forming_bar,
            more,
        })
    }

    /// fe's timer body: up to `ticks_per_frame(speed)` ticks this frame,
    /// marking the replay `completed` when the series runs out.
    pub fn run_frame(&mut self) {
        let per_frame = crate::sim::engine::ticks_per_frame(self.config.speed);
        for _ in 0..per_frame {
            match self.advance() {
                Some(outcome) if outcome.more => {}
                _ => {
                    self.status = ReplayStatus::Completed;
                    break;
                }
            }
        }
    }

    // -- Accessors for the signal-free surface ------------------------------
    // (Signals never enter the controller; the store snapshots these under
    // its generation bump.)

    /// Current status for the driver loop.
    pub fn status(&self) -> ReplayStatus {
        self.status
    }

    /// Clamped speed (fe init keeps the speed across re-inits).
    pub fn speed(&self) -> f64 {
        self.config.speed
    }

    /// The execution config read (symbol/exchange live only here, not in
    /// the state snapshot, which keeps fe's field set verbatim).
    pub fn config(&self) -> &ReplayConfig {
        &self.config
    }

    /// Whether a dataset + graph pair has been loaded (any control relies
    /// on both before transport buttons mean anything).
    pub fn has_dataset(&self) -> bool {
        !self.bars.is_empty() && self.graph.is_some()
    }

    /// Total dataset length (`totalBars`).
    pub fn total_bars(&self) -> usize {
        self.bars.len()
    }

    /// fe's `onBarChange` emissions derived from the books — the chart's
    /// visible slice. `None` means idle: the hook then falls back to the
    /// full loaded dataset, and so does the page wiring.
    pub fn visible_bars(&self) -> Option<Vec<ChartBar>> {
        match self.status {
            ReplayStatus::Idle => None,
            ReplayStatus::Completed => Some(self.bars.clone()),
            ReplayStatus::Playing | ReplayStatus::Paused => {
                if self.bars.is_empty() {
                    return Some(Vec::new());
                }
                let index = self.current_bar_index.min(self.bars.len());
                match &self.forming {
                    // Mid-bar: fe emitted `slice(0, currentBarIndex)` plus
                    // the forming candle while ticking.
                    Some(forming) => {
                        let mut visible: Vec<ChartBar> = self.bars[..index].to_vec();
                        visible.push(*forming);
                        Some(visible)
                    }
                    // Parked after a jump (or step-less pause): fe's jump
                    // emission was `slice(0, bounded + 1)` — settled bars
                    // including the current one.
                    None => Some(self.bars[..=index.min(self.bars.len() - 1)].to_vec()),
                }
            }
        }
    }

    /// fe `evaluateCurrentBar` — evaluate on the close phase; snapshot
    /// activity, and only bars that would place an order enter the
    /// confluence history (fe's guard).
    fn evaluate_current_bar(&mut self) {
        if self.current_bar_index >= self.bars.len() {
            return;
        }
        let confluence = match (&self.graph, &self.indicators) {
            (Some(graph), Some(indicators)) => {
                evaluate_bar_signals(graph, &self.bars, self.current_bar_index, indicators)
            }
            _ => return,
        };

        self.active_signals = confluence.signals.clone();
        self.last_confluence = Some(confluence.clone());

        if let Some(trade) = confluence.executed_trade {
            let bar = self.bars[self.current_bar_index];
            self.execute_trade(trade, &bar);
            self.confluence_history.insert(0, confluence); // latest first
        }
    }

    /// fe `handleTradeExecution`.
    fn execute_trade(&mut self, trade: ExecutedTrade, bar: &ChartBar) {
        // Signals-only mode: record nothing but the confluence row.
        if self.config.fill_model == FillModel::None {
            return;
        }
        let is_buy = trade.side == TradeAction::Buy;

        if self.config.fill_model == FillModel::BarFillThrough {
            // Strict limit trade-through: a touch is not a fill.
            let traded_through = if is_buy {
                bar.low < trade.price
            } else {
                bar.high > trade.price
            };
            if !traded_through {
                return;
            }
        }

        let mut execution_price = trade.price;
        if self.config.fill_model == FillModel::L2 {
            // Half-spread estimate off the bar range, plus user slippage,
            // applied adversely and rounded to the paisa.
            let close = if bar.close > 0.0 { bar.close } else { 1.0 };
            let estimated_half_spread = L2_RANGE_FACTOR * ((bar.high - bar.low) / close);
            let half_spread = estimated_half_spread.max(L2_HALF_SPREAD_FLOOR);
            let slippage_pct = if self.config.slippage_bps > 0.0 {
                self.config.slippage_bps
            } else {
                DEFAULT_L2_SLIPPAGE_BPS
            } / 10_000.0;
            let impact = half_spread + slippage_pct;
            let signed = if is_buy { 1.0 + impact } else { 1.0 - impact };
            execution_price = (trade.price * signed * 100.0).round() / 100.0;
        }

        let fill_fee = if self.config.commission > 0.0 {
            self.config.commission
        } else {
            DEFAULT_FILL_FEE
        };

        if self.active_position.is_none() {
            // Open a position (entry leg).
            self.active_position = Some(ActivePosition {
                symbol: self.config.symbol.clone(),
                exchange: self.config.exchange.clone(),
                side: if is_buy { Side::Long } else { Side::Short },
                quantity: trade.quantity,
                entry_price: execution_price,
                current_price: execution_price,
                unrealized_pnl: 0.0,
                unrealized_return_pct: 0.0,
                entry_timestamp: bar.time,
            });
        } else {
            // Same-side fills while a position is open do nothing (no
            // pyramiding, fe's implicit branch).
            let current_side = self.active_position.as_ref().expect("checked above").side;
            let is_opposite =
                (current_side == Side::Long && !is_buy) || (current_side == Side::Short && is_buy);

            if is_opposite {
                // Close the position: realized PnL net of both fills.
                let entry_price = self
                    .active_position
                    .as_ref()
                    .expect("checked above")
                    .entry_price;
                let entry_timestamp = self
                    .active_position
                    .as_ref()
                    .expect("checked above")
                    .entry_timestamp;
                let quantity = self
                    .active_position
                    .as_ref()
                    .expect("checked above")
                    .quantity;
                let mult = current_side.pnl_sign();
                let gross_pnl = (execution_price - entry_price) * quantity * mult;
                let total_fees = fill_fee * 2.0;
                let net_pnl = gross_pnl - total_fees;
                self.realized_pnl += net_pnl;

                let round_trip = RoundTrip {
                    id: format!("trip-{}", self.round_trips.len() + 1),
                    instrument: format!("{}.{}", self.config.symbol, self.config.exchange),
                    direction: match current_side {
                        Side::Long => Direction::Long,
                        Side::Short => Direction::Short,
                    },
                    status: RoundTripStatus::Closed,
                    quantity,
                    open_quantity: 0.0,
                    entry_ts: entry_timestamp,
                    exit_ts: Some(bar.time),
                    avg_entry: entry_price,
                    avg_exit: Some(execution_price),
                    entry_fills: vec![RoundTripLeg {
                        order_id: "sim-entry".to_string(),
                        ts: entry_timestamp,
                        price: entry_price,
                        quantity,
                        costs: fill_fee,
                    }],
                    exit_fills: vec![RoundTripLeg {
                        order_id: "sim-exit".to_string(),
                        ts: bar.time,
                        price: execution_price,
                        quantity: trade.quantity,
                        costs: fill_fee,
                    }],
                    fees: total_fees,
                    realized_pnl: Some(net_pnl),
                    return_pct: if entry_price > 0.0 {
                        Some((execution_price - entry_price) / entry_price * mult)
                    } else {
                        Some(0.0)
                    },
                    exit_reason: RoundTripExitReason::PositionClosed,
                };

                self.round_trips.insert(0, round_trip); // latest first
                self.active_position = None;
            }
        }
    }

    /// fe `getState`.
    pub fn state(&self) -> SimulationEngineState {
        let current_price = if self.current_bar_index < self.bars.len() {
            self.bars[self.current_bar_index].close
        } else {
            0.0
        };
        let (unrealized_pnl, _) =
            ActivePosition::pnl_at(self.active_position.as_ref(), current_price);
        let synthetic = |bar: ChartBar| SimulatedTick {
            timestamp: bar.time,
            time_str: String::new(), // fe getState's empty timeStr, pinned
            price: current_price,
            open: bar.open,
            high: bar.high,
            low: bar.low,
            close: bar.close,
            volume: bar.volume,
            bar_index: self.current_bar_index,
            phase: TickPhase::Close,
        };
        SimulationEngineState {
            status: self.status,
            speed: self.config.speed,
            current_bar_index: self.current_bar_index,
            total_bars: self.bars.len(),
            current_tick: self
                .bars
                .get(self.current_bar_index)
                .copied()
                .map(synthetic),
            active_signals: self.active_signals.clone(),
            last_confluence: self.last_confluence.clone(),
            confluence_history: self.confluence_history.clone(),
            round_trips: self.round_trips.clone(),
            active_position: self.active_position.clone(),
            realized_pnl: self.realized_pnl,
            unrealized_pnl,
            equity: self.config.initial_capital + self.realized_pnl + unrealized_pnl,
            initial_capital: self.config.initial_capital,
            selected_account_id: self.config.account_id.clone(),
            multi_timeframe: self.multi_timeframe,
            secondary_timeframe: self.secondary_timeframe.clone(),
            fill_model: self.config.fill_model.name(),
            commission: self.config.commission,
            slippage_bps: self.config.slippage_bps,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::engine::{FillModel, TickPhase};
    use crate::sim::selection::builtin_template;

    /// fe `engine.test.ts` `mockBars` (same generator).
    fn mock_bars(count: usize) -> Vec<ChartBar> {
        let mut bars = Vec::with_capacity(count);
        let mut price = 100.0;
        let base_time = 1_700_000_000;
        for i in 0..count {
            let delta = (i as f64 / 3.0).sin() * 2.0 + if i.is_multiple_of(2) { 1.0 } else { -0.5 };
            let open = price;
            let close = price + delta;
            let high = open.max(close) + 1.0;
            let low = open.min(close) - 1.0;
            price = close;
            bars.push(ChartBar {
                time: base_time + (i as i64) * 300,
                open,
                high,
                low,
                close,
                volume: 1000.0 + (i as f64) * 10.0,
            });
        }
        bars
    }

    /// A graph whose single condition always fires (close > 50 → bullish)
    /// with both order nodes present.
    fn always_firing_graph() -> StrategyGraphShape {
        use crate::sim::selection::{StrategyGraphEdge, StrategyGraphNode};
        let gt = |id: &str, threshold: f64| {
            let mut n = StrategyGraphNode {
                id: id.to_string(),
                node_type: "condition".to_string(),
                ..StrategyGraphNode::default()
            };
            n.params.insert("op".into(), "gt".into());
            n.params.insert("threshold".into(), threshold.into());
            n
        };
        let mut graph = StrategyGraphShape {
            name: "always".to_string(),
            ..StrategyGraphShape::default()
        };
        graph.nodes = vec![
            {
                let mut n = StrategyGraphNode {
                    id: "f".to_string(),
                    node_type: "indicator".to_string(),
                    ..StrategyGraphNode::default()
                };
                n.indicator = Some("sma".into());
                n.params.insert("period".into(), 1.into());
                n
            },
            gt("c", 50.0),
            {
                let mut n = StrategyGraphNode {
                    id: "o_buy".to_string(),
                    node_type: "order".to_string(),
                    ..StrategyGraphNode::default()
                };
                n.params.insert("side".into(), "buy".into());
                n.params.insert("quantity".into(), 10.into());
                n
            },
            {
                let mut n = StrategyGraphNode {
                    id: "o_sell".to_string(),
                    node_type: "order".to_string(),
                    ..StrategyGraphNode::default()
                };
                n.params.insert("side".into(), "sell".into());
                n.params.insert("quantity".into(), 10.into());
                n
            },
        ];
        graph.edges = vec![StrategyGraphEdge {
            id: "e".into(),
            from_node: "f".into(),
            from_port: "value".into(),
            to_node: "c".into(),
            to_port: "left".into(),
        }];
        graph
    }

    #[test]
    fn stepping_four_ticks_lands_on_the_next_bar() {
        // fe `engine.test.ts`: step() × 4 → currentBarIndex 1.
        let mut controller = ReplayController::default();
        let bars = mock_bars(30);
        let (graph, _) = builtin_template();
        controller.init(&bars, &graph, ReplayConfig::default());

        let state = controller.state();
        assert_eq!(state.status, ReplayStatus::Idle);
        assert_eq!(state.current_bar_index, 0);
        assert!(!bars.is_empty());

        for _ in 0..4 {
            controller.step();
        }
        let state = controller.state();
        assert_eq!(state.current_bar_index, 1);

        controller.instant_run_to_end();
        let state = controller.state();
        assert_eq!(state.status, ReplayStatus::Completed);
        assert_eq!(state.current_bar_index, bars.len() - 1);
    }

    #[test]
    fn tick_phases_swing_and_form_the_candle_like_fe() {
        let mut controller = ReplayController::default();
        let bars = vec![ChartBar {
            time: 1_700_000_000,
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0, // bullish: low first, then high
            volume: 800.0,
        }];
        let (graph, _) = builtin_template();
        controller.init(&bars, &graph, ReplayConfig::default());

        // Phase 0: open tick.
        let outcome = controller.advance().expect("bars present");
        assert_eq!(outcome.tick.phase, TickPhase::Open);
        assert_eq!(outcome.tick.price, 100.0);
        assert_eq!(outcome.forming_bar.volume, 200.0); // 800 × 1/4
        assert_eq!(outcome.forming_bar.close, 100.0);
        assert!(outcome.more);

        // Phase 1: bullish bar's first swing is the low.
        let outcome = controller.advance().expect("more");
        assert_eq!(outcome.tick.phase, TickPhase::Intra1);
        assert_eq!(outcome.tick.price, 90.0);
        assert_eq!(outcome.forming_bar.volume, 400.0); // 800 × 2/4
        assert_eq!(outcome.forming_bar.low, 90.0);

        // Phase 2: bullish second swing is the high; low now knows
        // the real bar low.
        let outcome = controller.advance().expect("more");
        assert_eq!(outcome.tick.phase, TickPhase::Intra2);
        assert_eq!(outcome.tick.price, 110.0);
        assert_eq!(outcome.forming_bar.high, 110.0);
        assert_eq!(outcome.forming_bar.low, 90.0);
        assert_eq!(outcome.forming_bar.volume, 600.0);

        // Phase 3 (close): evaluation runs here.
        let outcome = controller.advance().expect("final tick");
        assert_eq!(outcome.tick.phase, TickPhase::Close);
        assert_eq!(outcome.tick.price, 105.0);
        assert_eq!(outcome.forming_bar.close, 105.0);
        assert_eq!(outcome.forming_bar.volume, 800.0);
        // The single-bar series ends with the close tick emitted; no more.
        assert!(!outcome.more);
    }

    #[test]
    fn run_frame_marks_completed_when_the_series_exhausts() {
        let mut controller = ReplayController::default();
        let bars = mock_bars(3);
        let (graph, _) = builtin_template();
        controller.init(&bars, &graph, ReplayConfig::default());

        while controller.status != ReplayStatus::Completed {
            controller.run_frame();
        }
        let state = controller.state();
        assert_eq!(state.status, ReplayStatus::Completed);
        // After exhaustion the index parks past the end like fe's roll,
        // and getState clamps the synthetic current tick to `None`.
        assert!(state.current_tick.is_none());
    }

    /// A graph with two independent cross conditions on a drifting close:
    /// `crosses_above` (buys 10) and `crosses_below` (sells 10), so a full
    /// long round trip forms inside the fixture (fe's mock graph narrowed
    /// to two conditions + both order nodes).
    fn cross_graph() -> StrategyGraphShape {
        use crate::sim::selection::{StrategyGraphEdge, StrategyGraphNode};
        let indicator = |id: &str, period: usize| {
            let mut n = StrategyGraphNode {
                id: id.to_string(),
                node_type: "indicator".to_string(),
                ..StrategyGraphNode::default()
            };
            n.indicator = Some("sma".into());
            n.params.insert("period".into(), (period as u64).into());
            n
        };
        let condition = |id: &str, op: &str| {
            let mut n = StrategyGraphNode {
                id: id.to_string(),
                node_type: "condition".to_string(),
                ..StrategyGraphNode::default()
            };
            n.params.insert("op".into(), op.into());
            n
        };
        let order = |id: &str, side: &str| {
            let mut n = StrategyGraphNode {
                id: id.to_string(),
                node_type: "order".to_string(),
                ..StrategyGraphNode::default()
            };
            n.params.insert("side".into(), side.into());
            n.params.insert("quantity".into(), 10.into());
            n
        };
        let edge = |id: &str, from: &str, to: &str, to_port: &str| StrategyGraphEdge {
            id: id.to_string(),
            from_node: from.to_string(),
            from_port: "value".to_string(),
            to_node: to.to_string(),
            to_port: to_port.to_string(),
        };
        StrategyGraphShape {
            name: "crosses".to_string(),
            universe_named: String::new(),
            universe_explicit: Vec::new(),
            driving_interval: 5,
            driving_aggregation: "minute".to_string(),
            warmup_bars: 0,
            nodes: vec![
                indicator("f", 1),
                indicator("s", 2),
                condition("ca", "crosses_above"),
                condition("cb", "crosses_below"),
                order("o_buy", "buy"),
                order("o_sell", "sell"),
            ],
            edges: vec![
                edge("e1", "f", "ca", "left"),
                edge("e2", "s", "ca", "right"),
                edge("e3", "f", "cb", "left"),
                edge("e4", "s", "cb", "right"),
            ],
        }
    }

    #[test]
    fn bar_fill_through_requires_penetration() {
        let mut controller = ReplayController::default();
        // High == low == open == close = 100 (flat bar): a buy limit at the
        // close can never be *traded through* (low < price is false).
        let bars = vec![ChartBar {
            time: 1_700_000_000,
            open: 100.0,
            high: 100.0,
            low: 100.0,
            close: 100.0,
            volume: 0.0,
        }];
        let (graph, _) = builtin_template();
        let config = ReplayConfig {
            fill_model: FillModel::BarFillThrough,
            ..ReplayConfig::default()
        };
        controller.init(&bars, &graph, config);
        controller.instant_run_to_end();

        assert!(controller.active_position.is_none());
        assert!(controller.round_trips.is_empty());
        // A single flat bar never crosses — the template fires nothing, so
        // even the order rows are absent (nothing was decided).
        assert_eq!(controller.confluence_history.len(), 0);
    }

    #[test]
    fn bar_fill_through_requires_penetration_of_the_close_price() {
        let mut controller = ReplayController::default();
        // Same cross graph but with the strict model: the buy at 12 needs
        // the bar's low strictly below 12 — a flat bar never fills.
        let source: [f64; 7] = [10.0, 9.0, 8.0, 12.0, 15.0, 6.0, 5.0];
        let bars: Vec<ChartBar> = source
            .iter()
            .enumerate()
            .map(|(i, &close)| ChartBar {
                time: 1_700_000_000 + (i as i64) * 300,
                open: close.max(10.0),
                high: close.max(10.0),
                low: close.max(10.0),
                close: close.max(10.0),
                volume: 100.0,
            })
            .collect();
        let graph = cross_graph();
        let config = ReplayConfig {
            fill_model: FillModel::BarFillThrough,
            ..ReplayConfig::default()
        };
        controller.init(&bars, &graph, config);
        controller.instant_run_to_end();

        // Every low == every close ⇒ no penetration, nothing fills or trips.
        assert!(controller.active_position.is_none());
        assert!(controller.round_trips.is_empty());
        // But both deciding bars are recorded even though nothing filled
        // (fe unshifts the snapshot whenever an order would be placed,
        // regardless of the fill model's decision).
        assert_eq!(controller.confluence_history.len(), 2);
        assert!(
            controller
                .confluence_history
                .iter()
                .all(|snapshot| snapshot.executed_trade.is_some())
        );
    }

    #[test]
    fn a_pierced_limit_fills_and_an_opposite_close_books_the_trip() {
        let mut controller = ReplayController::default();
        // closes 10, 9, 8, 12, 15, 6, 5: sma1 = close, sma2 slides; buys on
        // 12 (cross above), sells on 6 (cross below) — one full long trip.
        let source: [f64; 7] = [10.0, 9.0, 8.0, 12.0, 15.0, 6.0, 5.0];
        let bars: Vec<ChartBar> = source
            .iter()
            .enumerate()
            .map(|(i, &close)| ChartBar {
                time: 1_700_000_000 + (i as i64) * 300,
                open: close,
                high: close,
                low: close,
                close,
                volume: 100.0,
            })
            .collect();
        let graph = cross_graph();
        let config = ReplayConfig {
            fill_model: FillModel::BarFill,
            ..ReplayConfig::default()
        };
        controller.init(&bars, &graph, config);
        controller.instant_run_to_end();

        let trip = controller
            .round_trips
            .first()
            .expect("buy then sell must close");
        assert_eq!(trip.id, "trip-1");
        assert_eq!(trip.instrument, "TCS.NSE");
        assert_eq!(trip.direction, Direction::Long);
        assert_eq!(trip.status, RoundTripStatus::Closed);
        assert_eq!(trip.open_quantity, 0.0);
        assert_eq!(trip.entry_fills[0].order_id, "sim-entry");
        assert_eq!(trip.exit_fills[0].order_id, "sim-exit");
        assert_eq!(trip.entry_fills[0].price, 12.0);
        // Long 10 @ 12 → exit 6: gross −60, fees 20, net −80.
        assert_eq!(trip.fees, 20.0);
        assert_eq!(trip.realized_pnl, Some(-80.0));
        assert_eq!(trip.return_pct, Some(-0.5));
        assert_eq!(controller.realized_pnl, -80.0);
        // History is latest-first and only carries trade-bearing bars.
        assert_eq!(controller.confluence_history[0].bar_index, 5);
        assert_eq!(controller.confluence_history.last().unwrap().bar_index, 3);
    }

    #[test]
    fn l2_fills_apply_half_spread_and_default_slippage_to_the_paisa() {
        let mut controller = ReplayController::default();
        // Range 4 over close 100 → half spread 0.002 (> floor), default
        // slippage 2bps → impact 0.0022 → buy price 100.22.
        let bars = vec![ChartBar {
            time: 1_700_000_000,
            open: 98.0,
            high: 102.0,
            low: 98.0,
            close: 100.0,
            volume: 100.0,
        }];
        let graph = always_firing_graph();
        let config = ReplayConfig {
            fill_model: FillModel::L2,
            ..ReplayConfig::default()
        };
        controller.init(&bars, &graph, config);
        controller.instant_run_to_end();

        let position = controller
            .active_position
            .expect("the firing bar opens long");
        assert_eq!(position.side, Side::Long);
        assert_eq!(position.entry_price, 100.22);
        assert!(controller.round_trips.is_empty());
    }

    #[test]
    fn same_side_fills_do_nothing_on_an_open_position() {
        let mut controller = ReplayController::default();
        let bars = vec![
            ChartBar {
                time: 1_700_000_000,
                open: 100.0,
                high: 101.0,
                low: 100.0,
                close: 100.0,
                volume: 1.0,
            },
            ChartBar {
                time: 1_700_000_300,
                open: 100.0,
                high: 101.0,
                low: 100.0,
                close: 100.0,
                volume: 1.0,
            },
        ];
        let graph = always_firing_graph();
        controller.init(&bars, &graph, ReplayConfig::default());
        controller.instant_run_to_end();

        // Both bars fire buys; the second must not pyramid or trip.
        assert!(controller.round_trips.is_empty());
        let position = controller.active_position.as_ref().expect("long open");
        assert_eq!(position.quantity, 10.0);
        assert_eq!(position.entry_price, 100.0);
        // Mark-to-close is flat: unrealized 0, but the books hold the long.
        assert_eq!(controller.state().unrealized_pnl, 0.0);
    }

    #[test]
    fn the_none_fill_model_records_signals_without_orders() {
        let mut controller = ReplayController::default();
        let bars = mock_bars(10);
        let graph = always_firing_graph();
        let config = ReplayConfig {
            fill_model: FillModel::None,
            ..ReplayConfig::default()
        };
        controller.init(&bars, &graph, config);
        controller.instant_run_to_end();

        assert!(controller.active_position.is_none());
        assert!(controller.round_trips.is_empty());
        // fe keeps recording the snapshots for order-bearing bars even in
        // signals-only mode — execution is skipped, not the row.
        assert!(
            controller
                .confluence_history
                .iter()
                .all(|snapshot| snapshot.executed_trade.is_some())
        );
    }

    #[test]
    fn speed_clamps_and_the_config_survives_state() {
        let mut controller = ReplayController::default();
        controller.set_speed(0.0);
        assert_eq!(controller.config.speed, 1.0);
        controller.set_speed(100.0);
        let state = controller.state();
        assert_eq!(state.speed, 100.0);
        assert_eq!(state.fill_model, "bar_fill");
        assert_eq!(state.selected_account_id, "acc-sandbox-primary");
        controller.set_execution_config(FillModel::L2, 5.0, 3.0);
        let state = controller.state();
        assert_eq!(state.fill_model, "l2");
        assert_eq!(state.commission, 5.0);
        controller.set_multi_timeframe(true, "15m");
        assert_eq!(controller.state().secondary_timeframe, "15m");
        assert_eq!(controller.state().total_bars, 0);
    }

    // -- Accessors + the visible-bars surface (P7 store integration) -------

    #[test]
    fn the_visible_slice_falls_back_to_nothing_while_idle() {
        let mut controller = ReplayController::default();
        controller.init(
            &mock_bars(5),
            &always_firing_graph(),
            ReplayConfig::default(),
        );
        assert!(controller.has_dataset());
        assert_eq!(controller.total_bars(), 5);
        assert_eq!(controller.status(), ReplayStatus::Idle);
        // Idle: the chart shows the full loaded dataset (fe's hook fell
        // back to `bars` whenever `status === 'idle'`).
        assert!(controller.visible_bars().is_none());

        controller.reset();
        assert_eq!(controller.status(), ReplayStatus::Idle);
        assert!(controller.visible_bars().is_none());
    }

    #[test]
    fn playing_emits_the_settled_prefix_plus_the_forming_candle() {
        let mut controller = ReplayController::default();
        let bars = mock_bars(4);
        controller.init(&bars, &always_firing_graph(), ReplayConfig::default());

        // fe's play() emitted the first tick immediately.
        controller.play();
        assert_eq!(controller.status(), ReplayStatus::Playing);
        let visible = controller.visible_bars().expect("slice");
        // Settled prefix is empty at bar 0's open phase; the forming candle
        // carries the bar's time/open and the tick as its close.
        assert_eq!(visible.len(), 1);
        let forming = &visible[0];
        assert_eq!(forming.time, bars[0].time);
        assert_eq!(forming.open, bars[0].open);
        assert_eq!(forming.close, bars[0].open); // Open-phase tick price.
        // The high/low only widen with the swings: at the open phase they
        // hug the tick (current_high == max(open, tick)).
        assert_eq!(forming.high, bars[0].open.max(bars[0].open));
        assert_eq!(forming.low, bars[0].open.min(bars[0].open));

        // Three more ticks push the phase onto bar 1's open; the settled
        // prefix is exactly bar 0, then bar 1's forming candle.
        controller.advance();
        controller.advance();
        controller.advance();
        controller.advance();
        let visible = controller.visible_bars().expect("slice");
        assert_eq!(visible.len(), 2);
        assert_eq!(visible[0].time, bars[0].time);
        // The settled copy equals the real bar fe showed for bar 0.
        let settled = &visible[0];
        assert_eq!(
            (settled.open, settled.high, settled.low, settled.close),
            (bars[0].open, bars[0].high, bars[0].low, bars[0].close)
        );
        assert_eq!(visible[1].close, bars[1].open);
    }

    #[test]
    fn a_jump_reads_settled_bars_inclusive_of_the_landing_bar() {
        let mut controller = ReplayController::default();
        let bars = mock_bars(6);
        controller.init(&bars, &always_firing_graph(), ReplayConfig::default());

        // A jump from the transport rides an in-flight replay (fe's idle
        // chart ignores the emission and keeps the full dataset).
        controller.play();
        controller.jump_to_bar(3);
        assert!(controller.forming.is_none());
        let visible = controller.visible_bars().expect("slice");
        // fe's jump emission: `slice(0, bounded + 1)` — settled, inclusive.
        assert_eq!(visible.len(), 4);
        assert_eq!(visible[3].close, bars[3].close);
    }

    #[test]
    fn exhausted_and_instant_runs_read_the_full_dataset() {
        let mut controller = ReplayController::default();
        let bars = mock_bars(3);
        controller.init(&bars, &always_firing_graph(), ReplayConfig::default());

        // fe's timer body advances ticks-per-frame; completion is marked
        // when the series runs out.
        controller.play();
        // A dataset of 3 bars is 12 ticks (4 phases each), including play's
        // opening tick; the budget overshoots on purpose.
        for _ in 0..20 {
            if controller.status() == ReplayStatus::Completed {
                break;
            }
            controller.run_frame();
        }
        assert_eq!(controller.status(), ReplayStatus::Completed);
        let visible = controller.visible_bars().expect("slice");
        assert_eq!(visible.len(), 3);
        assert_eq!(visible[2].close, bars[2].close);

        // fe's instant-run emission: the whole array.
        controller.reset();
        controller.instant_run_to_end();
        assert_eq!(controller.status(), ReplayStatus::Completed);
        assert_eq!(controller.visible_bars().map(|v| v.len()), Some(3));
    }
}
