//! Simulator replay slice — the P7 replay engine, its playback clock
//! (play/pause/step/instant, speed), the bar cursor and the visible candle
//! window. Owned by [`super::simulator_store::SimulatorStore`].
//!
//! The [`ReplayController`] stays plain state — no signals inside it, no
//! timers — and its owner here bumps a generation [`RwSignal`] after every
//! mutation, matching fe's hook-shaped `setState` notify. fe re-initialised
//! the engine from the *chart's* bars on an `initKey` change; here the
//! chart container publishes its loaded dataset through the slice's
//! `engine_bars` signal (via a callback prop), so the replay shares
//! the chart's single fetch. The frame driver is one `spawn_local` loop
//! token-guarded against re-arm (fe's `setInterval`); the driver bumps the
//! generation per frame — the dock tabs, the HUD pills and the visible-bars
//! chart swap all read it.

use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
use super::run_store::BrowserClock;
use crate::features::simulator::kernel::bars::BarsState;
use crate::features::simulator::kernel::chart_data::ChartBar;
use crate::features::simulator::kernel::controls::SimulatorControls;
use crate::features::simulator::kernel::engine::controller::{ReplayController, SimulationEngineState};
use crate::features::simulator::kernel::engine::{DEFAULT_ACCOUNT_ID, FillModel, ReplayConfig, ReplayStatus};
#[cfg(target_arch = "wasm32")]
use crate::features::simulator::kernel::run::Clock;
use crate::features::simulator::kernel::selection::StrategyGraphShape;

/// The replay engine plus the reactive surface that mirrors it.
#[derive(Clone, Copy)]
pub struct ReplayStore {
    /// The P7 replay engine — plain state held once, mutated in place
    /// (fe kept the controller in a hook; here the store owns it).
    pub controller: StoredValue<ReplayController>,
    /// Bumped by every replay mutation. The dock's engine-state memo and
    /// each panel read track this one integer (fe's `setState` notify).
    pub generation: RwSignal<u64>,
    /// Bumped whenever the frame-driver loop must retire (pause, reset,
    /// init, speed change, unmount); each loop holds its issued token and
    /// exits when the signal moves past it.
    pub driver: RwSignal<u64>,
    /// The dataset the chart container loaded (fe read the dataLayer bars
    /// the chart already held; the replay shares them without a second
    /// fetch). The chart publishes through a callback prop.
    pub engine_bars: RwSignal<BarsState>,
    /// The controller's visible-bars slice (fe's `onBarChange` emission);
    /// `None` while idle, when the chart shows the full loaded dataset.
    pub visible: RwSignal<Option<Vec<ChartBar>>>,
}

impl ReplayStore {
    /// An idle controller over no dataset.
    pub(super) fn new() -> Self {
        Self {
            controller: StoredValue::new(ReplayController::default()),
            generation: RwSignal::new(0u64),
            driver: RwSignal::new(0u64),
            engine_bars: RwSignal::new(BarsState::initial()),
            visible: RwSignal::new(None),
        }
    }

    // The controller stays signal-free (P7 contract); every method below
    // mutates it in place, then syncs the reactive surface: the visible
    // slice (fe's `onBarChange`) and the generation bump (fe's `setState`).

    /// Snapshots `ReplayController::state()` without cloning the controller
    /// (`try_update_value` mutates in place; nothing is written, so this is
    /// a read that happens to borrow through the write guard). Call inside
    /// a memo tracking `generation`.
    pub fn state(&self) -> SimulationEngineState {
        self.controller
            .try_update_value(|controller| controller.state())
            .expect("the replay controller outlives its store")
    }

    /// fe's `initKey` re-init: a new dataset (the chart's publish) plus a
    /// fresh config from `controls` — fe re-initialised on
    /// `symbol|exchange|timeframe|bars|capital|account|fillModel|
    /// commission|slippage`; here the instrument split, the capital and the
    /// fill settings ride `controls`, so one method covers them. The
    /// speed survives re-initialisation (fe `init` never touched it).
    pub fn init(
        &self,
        bars: &[ChartBar],
        instrument_id: &str,
        controls: &SimulatorControls,
        graph: &StrategyGraphShape,
    ) {
        if bars.is_empty() {
            return;
        }
        let (symbol, exchange) = match instrument_id.rsplit_once('.') {
            Some((symbol, exchange)) if !symbol.is_empty() && !exchange.is_empty() => {
                (symbol.to_string(), exchange.to_string())
            }
            // The universe id should always be `SYMBOL.EXCHANGE`; fall back
            // to the controls' own fields rather than inventing a symbol.
            _ => (controls.symbol.clone(), controls.exchange.clone()),
        };
        let config = ReplayConfig {
            symbol,
            exchange,
            initial_capital: controls.capital,
            account_id: DEFAULT_ACCOUNT_ID.to_string(),
            fill_model: FillModel::from_name(&controls.fill_model),
            commission: controls.commission,
            slippage_bps: controls.slippage,
            // fe's `init` left `speed` alone (it was constructed once with
            // the hook); carry the clamped value across re-inits.
            speed: self
                .controller
                .try_update_value(|controller| controller.speed())
                .unwrap_or(1.0),
        };
        self.driver.update(|token| *token += 1);
        self.controller
            .update_value(|controller| controller.init(bars, graph, config));
        self.after_mutation();
    }

    /// fe `play` — reset-when-exhausted semantics live in the controller;
    /// the frame-driver loop is re-armed here (the controller owns no
    /// timers, the app drives `run_frame`).
    pub fn play(&self) {
        self.controller.update_value(|controller| controller.play());
        self.spawn_driver();
        self.after_mutation();
    }

    /// fe `pause`.
    pub fn pause(&self) {
        // Retire the loop before the status flips, so no queued frame
        // ticks behind the pause.
        self.driver.update(|token| *token += 1);
        self.controller
            .update_value(|controller| controller.pause());
        self.after_mutation();
    }

    /// fe `step` — one tick; pausing (and retiring the loop) is the
    /// controller's own semantics, this only keeps the driver in step.
    pub fn step(&self) {
        let playing = self
            .controller
            .try_update_value(|controller| {
                controller.step();
                controller.status() == ReplayStatus::Playing
            })
            .unwrap_or(false);
        if !playing {
            self.driver.update(|token| *token += 1);
        }
        self.after_mutation();
    }

    /// fe `reset`.
    pub fn reset(&self) {
        self.driver.update(|token| *token += 1);
        self.controller
            .update_value(|controller| controller.reset());
        self.after_mutation();
    }

    /// fe `instantRunToEnd` — sweep every bar at its close phase.
    pub fn instant(&self) {
        self.driver.update(|token| *token += 1);
        self.controller
            .update_value(|controller| controller.instant_run_to_end());
        self.after_mutation();
    }

    /// fe `setSpeed` — while playing the interval changes, so the timer is
    /// re-armed at the new cadence (fe stopped and restarted its timer).
    pub fn set_speed(&self, speed: f64) {
        let playing = self
            .controller
            .try_update_value(|controller| {
                controller.set_speed(speed);
                controller.status() == ReplayStatus::Playing
            })
            .unwrap_or(false);
        if playing {
            self.spawn_driver();
        }
        self.after_mutation();
    }

    /// fe `jumpToBar` — scrub across the dataset. While playing the timer
    /// continues from the landing bar (fe's jump never stopped its timer).
    pub fn jump(&self, index: usize) {
        let playing = self
            .controller
            .try_update_value(|controller| {
                controller.jump_to_bar(index);
                controller.status() == ReplayStatus::Playing
            })
            .unwrap_or(false);
        if !playing {
            self.driver.update(|token| *token += 1);
        }
        self.after_mutation();
    }

    /// fe's timer body: one frame of ticks, and the books' visible slice
    /// re-cut (called only by the driver loop).
    pub fn frame(&self) {
        self.controller
            .update_value(|controller| controller.run_frame());
        self.after_mutation();
    }

    /// Syncs the reactive surface after any controller mutation: the
    /// visible slice (fe's `onBarChange`) and the generation bump (fe's
    /// state notification).
    fn after_mutation(&self) {
        let visible = self
            .controller
            .try_update_value(|controller| controller.visible_bars())
            .flatten();
        self.visible.set(visible);
        self.generation.update(|generation| *generation += 1);
    }

    /// Arms a fresh frame-driver loop: the previous one retires via the
    /// driver token, the new loop waits each frame's `interval_ms` on the
    /// browser event loop (fe's `setInterval(intervalMs)`; the controller
    /// owns no timers per the P7 contract). Native builds never render the
    /// page, so there the loop is a no-op and callers advance by step.
    fn spawn_driver(&self) {
        self.driver.update(|token| *token += 1);
        let token = self.driver.get_untracked();
        #[cfg(target_arch = "wasm32")]
        {
            let store = *self;
            let driver = self.driver;
            leptos::task::spawn_local(async move {
                loop {
                    if driver.get_untracked() != token {
                        return;
                    }
                    // fe read `intervalMs` off the live config every frame;
                    // the speed may change between sleeps.
                    let interval = store
                        .controller
                        .try_update_value(|controller| {
                            crate::features::simulator::kernel::engine::interval_ms(controller.speed())
                        })
                        .unwrap_or(180);
                    Clock::sleep(&BrowserClock, interval as u64).await;
                    if driver.get_untracked() != token {
                        return;
                    }
                    // A pause could have raced a queued frame even without
                    // a token bump (same token, different world); never
                    // tick a controller that is not playing.
                    if store
                        .controller
                        .try_update_value(|controller| controller.status())
                        != Some(ReplayStatus::Playing)
                    {
                        return;
                    }
                    store.frame();
                    if store
                        .controller
                        .try_update_value(|controller| controller.status())
                        == Some(ReplayStatus::Completed)
                    {
                        return;
                    }
                }
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = token;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::simulator::kernel::controls::default_controls;
    use crate::features::simulator::kernel::selection::builtin_template;
    use crate::features::simulator::stores::controls_store::ControlsPatch;

    /// fe `engine.test.ts` `mockBars` — the same generator the controller
    /// tests use, so the store tests run over a real shape too.
    fn chart_bars(count: usize) -> Vec<ChartBar> {
        let mut bars = Vec::with_capacity(count);
        let mut price = 100.0;
        let base_time = 1_700_000_000;
        for i in 0..count {
            let delta = (i as f64 / 3.0).sin() * 2.0 + if i.is_multiple_of(2) { 1.0 } else { -0.5 };
            let open = price;
            let close = price + delta;
            price = close;
            bars.push(ChartBar {
                time: base_time + (i as i64) * 300,
                open,
                high: open.max(close) + 1.0,
                low: open.min(close) - 1.0,
                close,
                volume: 1000.0,
            });
        }
        bars
    }

    /// Runs the body inside a reactive owner over a fresh slice.
    fn with_replay<T>(f: impl FnOnce(&ReplayStore) -> T) -> T {
        let owner = leptos::prelude::Owner::new();
        let result = owner.with(|| f(&ReplayStore::new()));
        owner.unset();
        result
    }

    /// `init` over a fresh simulator's selection: default controls and the
    /// built-in template graph.
    fn init_default(replay: &ReplayStore, bars: &[ChartBar]) {
        let (graph, _) = builtin_template();
        replay.init(bars, "TCS.NSE", &default_controls(), &graph);
    }

    #[test]
    fn init_replay_seeds_the_engine_from_the_chart_dataset_and_the_controls() {
        with_replay(|replay| {
            let bars = chart_bars(5);
            let (graph, _) = builtin_template();
            let mut controls = default_controls();
            ControlsPatch {
                capital: Some(50_000.0),
                commission: Some(12.0),
                fill_model: Some("l2".to_string()),
                ..Default::default()
            }
            .apply(&mut controls);
            replay.init(&bars, "TCS.NSE", &controls, &graph);

            let state = replay.state();
            assert_eq!(state.total_bars, 5);
            assert_eq!(replay.controller.get_value().total_bars(), 5);
            assert_eq!(replay.controller.get_value().status(), ReplayStatus::Idle);
            // fe's init: symbol/exchange split, the panel's capital and the
            // fill settings land on the controller; the account stays the
            // sandbox default.
            assert_eq!(state.selected_account_id, "acc-sandbox-primary");
            assert_eq!(state.initial_capital, 50_000.0);
            assert_eq!(state.commission, 12.0);
            assert_eq!(state.fill_model, "l2");

            // The generation moved; the visible slice stays `None` (idle
            // shows the full loaded dataset).
            assert!(replay.generation.get_untracked() >= 1);
            assert!(replay.visible.get_untracked().is_none());

            // The config fields that make the engine *trade* this
            // instrument are not in the state snapshot (fe kept the field
            // set verbatim); read the controller's config accessor.
            let controller = replay.controller.get_value();
            assert_eq!(controller.config().symbol, "TCS");
            assert_eq!(controller.config().exchange, "NSE");

            // Empty bars never initialise (fe guarded `bars.length`).
            replay.init(&[], "TCS.NSE", &controls, &graph);
            assert_eq!(replay.state().total_bars, 5);
        });
    }

    #[test]
    fn speed_survives_a_reinit_and_restarts_play_cadence() {
        with_replay(|replay| {
            let bars = chart_bars(4);
            init_default(replay, &bars);
            replay.set_speed(25.0);
            assert_eq!(replay.state().speed, 25.0);

            // fe's init leaves the speed alone; a dataset change re-inits.
            init_default(replay, &bars);
            assert_eq!(replay.state().speed, 25.0);

            // Non-playing speed changes do not touch the driver token.
            let before = replay.driver.get_untracked();
            replay.set_speed(50.0);
            assert_eq!(replay.driver.get_untracked(), before);
            assert_eq!(replay.state().speed, 50.0);
        });
    }

    #[test]
    fn play_step_and_pause_move_the_visible_slice_and_the_books() {
        with_replay(|replay| {
            let bars = chart_bars(4);
            init_default(replay, &bars);

            // fe's play() emitted the first tick immediately.
            replay.play();
            assert_eq!(replay.state().status, ReplayStatus::Playing);
            let visible = replay.visible.get_untracked().expect("slice");
            assert_eq!(visible.len(), 1);
            assert_eq!(visible[0].time, bars[0].time);
            assert_eq!(visible[0].close, bars[0].open);

            // Step one tick then pause (the controller pauses on step).
            replay.step();
            assert_eq!(replay.state().status, ReplayStatus::Paused);
            replay.pause();
            let visible = replay.visible.get_untracked().expect("slice");
            assert_eq!(visible.len(), 1);

            // Reset returns to idle; the slice falls back to nothing.
            replay.reset();
            assert_eq!(replay.state().status, ReplayStatus::Idle);
            let state = replay.state();
            assert_eq!(state.current_bar_index, 0);
            assert_eq!(state.realized_pnl, 0.0);
            assert_eq!(state.unrealized_pnl, 0.0);
            assert!(replay.visible.get_untracked().is_none());
        });
    }

    #[test]
    fn an_instant_run_completes_and_the_books_surface_the_trades() {
        with_replay(|replay| {
            let bars = chart_bars(30);
            init_default(replay, &bars);
            replay.instant();

            let state = replay.state();
            assert_eq!(state.status, ReplayStatus::Completed);
            assert_eq!(state.current_bar_index, bars.len() - 1);
            // The engine's books are the portfolio surface: either trades
            // closed or a position still holds — the store never invents
            // account rows for the panel either way.
            assert!(
                !state.round_trips.is_empty() || state.active_position.is_some(),
                "an oscillating dataset must produce visible book activity"
            );
            // Equity reconciles: initial + realized + unrealized.
            let expected = state.initial_capital + state.realized_pnl + state.unrealized_pnl;
            assert!((state.equity - expected).abs() < 1e-9);
            // The visible slice reads the full dataset at completion.
            assert_eq!(
                replay.visible.get_untracked().map(|v| v.len()),
                Some(bars.len())
            );

            // TheEngine-state memo is stable across re-reads.
            assert_eq!(replay.state(), state);
        });
    }
}
