//! The replay engine's configuration + status machine (P7, doc §P7).
//!
//! Ported from fe `core/simulator/engine/{types.ts,replay-controller.ts}`.
//! Everything here is pure and signal-free: the controller (next slice,
//! `controller.rs`) owns the plain state and is driven by the app; the store
//! is where any `RwSignal` reactivity is attached.
//!
//! Honesty invariants carried over: the engine replays **real** bars only —
//! the forming candle is derived phase-by-phase from the current bar
//! (`open → intra1 → intra2 → close`), never invented, and executed trades
//! are `bar.close` fills evaluated on the close phase, like fe.
//!
//! Deviations (pinned):
//! - fe drives ticks with `window.setInterval`; kao's controller is driven
//!   by an app-side ticker that honours [`interval_ms`] and
//!   [`ticks_per_frame`] — wasm components do not own browser timers.
//! - fe's `timeStr` renders through `toLocaleString('en-IN', …
//!   'Asia/Kolkata')` (ICU). Kao formats via [`stamp_str`] / [`clock_str`]
//!   on fixed IST (UTC+5:30), approximating the en-IN pattern
//!   (`15 Jan, 03:30 pm`) without an i18n runtime.
//! - fe's sandbox portfolio adapter (`portfolio-adapter.ts`) is an external
//!   books service; in kao the engine's own books (open position + round
//!   trips) are the simulation state, matching the documented wart that
//!   replay execution state lives in the replay's own store slice.
//! - fe accepts any `Number` speed (including `NaN`, which poisons the
//!   timer); kao clamps non-finite/negatives to 1 in [`clamp_speed`].

pub mod controller;
pub mod evaluator;
pub mod indicators;

use crate::sim::chart_data::{civil_from_days, IST_OFFSET_SECONDS};

/// fe default `initialCapital`.
pub const DEFAULT_INITIAL_CAPITAL: f64 = 100_000.0;
/// fe default `accountId` (the sandbox account).
pub const DEFAULT_ACCOUNT_ID: &str = "acc-sandbox-primary";
/// fe default secondary timeframe for multi-timeframe mode.
pub const DEFAULT_SECONDARY_TIMEFRAME: &str = "1h";
/// fe order-node default quantity (`Number(params?.quantity ?? 10)`).
pub const DEFAULT_QUANTITY: f64 = 10.0;
/// fe default fill fee (`this.commission > 0 ? this.commission : 10`).
pub const DEFAULT_FILL_FEE: f64 = 10.0;
/// fe's `l2` slippage fallback: `(this.slippage > 0 ? this.slippage : 2)`.
pub const DEFAULT_L2_SLIPPAGE_BPS: f64 = 2.0;
/// fe's `l2` half-spread floor: `Math.max(0.0002, range/close * 0.05)`.
pub const L2_HALF_SPREAD_FLOOR: f64 = 2.0e-4;
/// fe's `l2` spread proportional factor on the bar range.
pub const L2_RANGE_FACTOR: f64 = 0.05;

/// `ReplayStatus` — verbatim fe union.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayStatus {
    /// `idle`.
    Idle,
    /// `playing`.
    Playing,
    /// `paused`.
    Paused,
    /// `completed`.
    Completed,
}

impl ReplayStatus {
    /// The fe union member (badge copy uses these strings verbatim).
    pub fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Playing => "playing",
            Self::Paused => "paused",
            Self::Completed => "completed",
        }
    }
}

/// Tick phase within a bar — 4 phases, `0=open`, `1=intra1` (the first
/// swing), `2=intra2` (the second swing), `3=close`, exactly fe's indices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TickPhase {
    /// `open`.
    Open,
    /// `intra1`.
    Intra1,
    /// `intra2`.
    Intra2,
    /// `close`.
    Close,
}

impl TickPhase {
    /// The fe phase index (`0..=3`).
    pub fn index(self) -> u8 {
        match self {
            Self::Open => 0,
            Self::Intra1 => 1,
            Self::Intra2 => 2,
            Self::Close => 3,
        }
    }

    /// From the fe numeric phase (values ≥ 3 are masked to `close`).
    pub fn from_index(index: u8) -> Self {
        match index {
            0 => Self::Open,
            1 => Self::Intra1,
            2 => Self::Intra2,
            _ => Self::Close,
        }
    }

    /// fe string union member (`'open' | 'intra1' | 'intra2' | 'close'`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Intra1 => "intra1",
            Self::Intra2 => "intra2",
            Self::Close => "close",
        }
    }
}

/// Signal indication — fe `'bullish' | 'bearish' | 'neutral'`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Indication {
    /// `bullish`.
    Bullish,
    /// `bearish`.
    Bearish,
    /// `neutral`.
    Neutral,
}

impl Indication {
    /// The fe union member.
    pub fn name(self) -> &'static str {
        match self {
            Self::Bullish => "bullish",
            Self::Bearish => "bearish",
            Self::Neutral => "neutral",
        }
    }
}

/// Position side / trade direction — fe `'long' | 'short' | 'buy' | 'sell'`
/// collapse to the two axes the engine actually distinguishes: the fill
/// direction (`buy`/`sell`) and the book side it opens (`long`/`short`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// `long` (opened by a buy).
    Long,
    /// `short` (opened by a sell).
    Short,
}

impl Side {
    /// fe string (`'long' | 'short'`) for book fields.
    pub fn name(self) -> &'static str {
        match self {
            Self::Long => "long",
            Self::Short => "short",
        }
    }

    /// The fill side that opened (or unwound) this side, as fe orders name
    /// it: `buy` opens longs, `sell` opens shorts.
    pub fn opening_order(self) -> &'static str {
        match self {
            Self::Long => "buy",
            Self::Short => "sell",
        }
    }

    /// The fill side that unwinds this side.
    pub fn closing_order(self) -> &'static str {
        match self {
            Self::Long => "sell",
            Self::Short => "buy",
        }
    }

    /// PnL multiplier — fe `currentSide === 'long' ? 1 : -1`.
    pub fn pnl_sign(self) -> f64 {
        match self {
            Self::Long => 1.0,
            Self::Short => -1.0,
        }
    }
}

/// Fill model — fe `fillModel` string union
/// (`'bar_fill' | 'bar_fill_through' | 'l2' | 'none'`).
///
/// `from_name` maps unknown spellings to `bar_fill`, matching fe's implicit
/// truthiness (only the three special spellings branch).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FillModel {
    /// `bar_fill` — fills at the signal price on touch (`<=/>=`).
    #[default]
    BarFill,
    /// `bar_fill_through` — strict limit trade-through: the bar must
    /// penetrate through the limit (`low < price` for buys /
    /// `high > price` for sells), else the order stays unfilled.
    BarFillThrough,
    /// `l2` — half-spread estimate `max(2e-4, range/close × 0.05)` plus the
    /// user slippage bps, applied adversely and rounded to the paisa.
    L2,
    /// `none` — signals only; confluence is recorded but no order is
    /// placed and no position opens.
    None,
}

impl FillModel {
    /// fe `fillModel` string (`'bar_fill' | …`). Unknown spellings map to
    /// `bar_fill`, matching fe's implicit truthiness.
    pub fn from_name(name: &str) -> Self {
        match name {
            "bar_fill_through" => Self::BarFillThrough,
            "l2" => Self::L2,
            "none" => Self::None,
            _ => Self::BarFill,
        }
    }

    /// The fe string for state serialization.
    pub fn name(self) -> &'static str {
        match self {
            Self::BarFill => "bar_fill",
            Self::BarFillThrough => "bar_fill_through",
            Self::L2 => "l2",
            Self::None => "none",
        }
    }
}

/// The engine's execution configuration — fe's `init`/`setExecutionConfig`
/// argument block. Speed is included because fe's `setSpeed` mutates the
/// same state object.
#[derive(Clone, Debug, PartialEq)]
pub struct ReplayConfig {
    /// Trading symbol (fe default `TCS`).
    pub symbol: String,
    /// Exchange (fe default `NSE`).
    pub exchange: String,
    /// fe default `100_000`.
    pub initial_capital: f64,
    /// fe default `acc-sandbox-primary`.
    pub account_id: String,
    /// fe default `bar_fill`.
    pub fill_model: FillModel,
    /// fe default `0` (bucks the fee floor to [`DEFAULT_FILL_FEE`] when 0).
    pub commission: f64,
    /// fe default `0` (falls to [`DEFAULT_L2_SLIPPAGE_BPS`] in `l2` fills).
    pub slippage_bps: f64,
    /// fe default `1` (clamped, see [`clamp_speed`]).
    pub speed: f64,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            symbol: "TCS".to_string(),
            exchange: "NSE".to_string(),
            initial_capital: DEFAULT_INITIAL_CAPITAL,
            account_id: DEFAULT_ACCOUNT_ID.to_string(),
            fill_model: FillModel::default(),
            commission: 0.0,
            slippage_bps: 0.0,
            speed: 1.0,
        }
    }
}

/// fe's `setSpeed` clamp (`Math.max(1, speed)`), hardened against
/// `NaN`/infinite input which fe would happily store.
pub fn clamp_speed(speed: f64) -> f64 {
    if speed.is_finite() && speed >= 1.0 {
        speed
    } else {
        1.0
    }
}

/// Timer interval, fe replay-controller verbatim:
/// `Math.max(12, Math.round(180 / Math.min(speed, 20)))` — speed is
/// pre-clamped by [`clamp_speed`].
pub fn interval_ms(speed: f64) -> u32 {
    let speed = clamp_speed(speed);
    let raw = (180.0 / speed.min(20.0) + 0.5) as u32;
    raw.max(12)
}

/// Ticks advanced per timer fire, fe verbatim:
/// `speed >= 50 ? 8 : speed >= 25 ? 4 : 1`.
pub fn ticks_per_frame(speed: f64) -> u32 {
    if speed >= 50.0 {
        8
    } else if speed >= 25.0 {
        4
    } else {
        1
    }
}

/// One simulated intra-bar tick — fe `SimulatedTick`.
#[derive(Clone, Debug, PartialEq)]
pub struct SimulatedTick {
    /// Bar timestamp, whole UTC seconds.
    pub timestamp: i64,
    /// IST rendering (en-IN approximated, see module doc).
    pub time_str: String,
    /// The tick price for the phase.
    pub price: f64,
    /// The bar's open.
    pub open: f64,
    /// The bar's high.
    pub high: f64,
    /// The bar's low.
    pub low: f64,
    /// The bar's close.
    pub close: f64,
    /// The bar's volume.
    pub volume: f64,
    /// The bar this tick belongs to.
    pub bar_index: usize,
    /// The phase.
    pub phase: TickPhase,
}

/// Any open simulated position — fe `ActivePosition`. Mark fields
/// (`current_price`, PnL rows) are refreshed by the controller.
#[derive(Clone, Debug, PartialEq)]
pub struct ActivePosition {
    /// Symbol, fe echo.
    pub symbol: String,
    /// Exchange, fe echo.
    pub exchange: String,
    /// Long or short.
    pub side: Side,
    /// Position quantity.
    pub quantity: f64,
    /// Fill price (post slippage model).
    pub entry_price: f64,
    /// Latest mark (tick price).
    pub current_price: f64,
    /// Unrealized PnL at the last mark.
    pub unrealized_pnl: f64,
    /// Unrealized return percent at the last mark.
    pub unrealized_return_pct: f64,
    /// Entry bar timestamp.
    pub entry_timestamp: i64,
}

impl ActivePosition {
    /// fe `calculatePositionPnL` (position + current price → unrealized).
    /// `None` means flat.
    pub fn pnl_at(position: Option<&Self>, current_price: f64) -> (f64, f64) {
        let Some(position) = position else {
            return (0.0, 0.0);
        };
        let dir = position.side.pnl_sign();
        let unrealized_pnl = (current_price - position.entry_price) * position.quantity * dir;
        let unrealized_return_pct = if position.entry_price > 0.0 {
            ((current_price - position.entry_price) / position.entry_price) * 100.0 * dir
        } else {
            0.0
        };
        (unrealized_pnl, unrealized_return_pct)
    }
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// IST wall-clock pieces for a UTC-second timestamp: (month index 1–12,
/// day, hour, minute, second), computed on UTC+5:30.
fn ist_parts(ts: i64) -> (u32, u32, u32, u32, u32) {
    let shifted = ts + IST_OFFSET_SECONDS;
    let (_, month, day) = civil_from_days(shifted.div_euclid(86_400));
    let secs = shifted.rem_euclid(86_400) as u32;
    (month, day, secs / 3_600, (secs % 3_600) / 60, secs % 60)
}

/// 12-hour clock rendering ("03:30 pm", "12:05 am").
fn clock(hh: u32, mm: u32, ss: Option<u32>) -> String {
    let suffix = if hh < 12 { "am" } else { "pm" };
    let h12 = if hh.is_multiple_of(12) { 12 } else { hh % 12 };
    match ss {
        Some(ss) => format!("{h12:02}:{mm:02}:{ss:02} {suffix}"),
        None => format!("{h12:02}:{mm:02} {suffix}"),
    }
}

/// fe tick `timeStr` (`toLocaleTimeString('en-IN', { timeZone:
/// 'Asia/Kolkata' })` approximated): hours, minutes, seconds.
pub fn clock_str(ts: i64) -> String {
    let (_, _, hh, mm, ss) = ist_parts(ts);
    clock(hh, mm, Some(ss))
}

/// fe confluence `timeStr` (`toLocaleString('en-IN', { month: 'short',
/// day: 'numeric', hour: '2-digit', minute: '2-digit', … })` approximated):
/// `15 Jan, 03:30 pm`, day-first like en-IN.
pub fn stamp_str(ts: i64) -> String {
    let (month, day, hh, mm, _) = ist_parts(ts);
    format!("{} {}, {}", day, MONTHS[(month - 1) as usize], clock(hh, mm, None))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::sim::chart_data::days_from_civil;

    /// UTC 2024-01-15 10:00:00 → IST 15:30.
    fn ts_utc_2024_01_15_10h() -> i64 {
        days_from_civil(2024, 1, 15) * 86_400 + 36_000
    }

    #[test]
    fn the_interval_map_matches_fe_exactly() {
        assert_eq!(interval_ms(1.0), 180);
        assert_eq!(interval_ms(5.0), 36);
        assert_eq!(interval_ms(15.0), 12); // round(180/15) == 12
        assert_eq!(interval_ms(20.0), 12); // min(20,20) → 9 → floored
        assert_eq!(interval_ms(50.0), 12); // min(50,20) → 9 → floored
        assert_eq!(interval_ms(1000.0), 12);
        // Non-finite / zero speed clamps to 1 first (fe would poison).
        assert_eq!(interval_ms(f64::NAN), 180);
        assert_eq!(interval_ms(0.0), 180);
        assert_eq!(ticks_per_frame(49.9), 4); // ≥ 25 but < 50
        assert_eq!(ticks_per_frame(24.9), 1);
        assert_eq!(ticks_per_frame(50.0), 8);
        assert_eq!(ticks_per_frame(25.0), 4);
        assert_eq!(ticks_per_frame(1.0), 1);
        assert_eq!(clamp_speed(f64::INFINITY), 1.0);
        assert_eq!(clamp_speed(3.0), 3.0);
    }

    #[test]
    fn tick_phase_round_trips_the_fe_string_union() {
        assert_eq!(TickPhase::Open.index(), 0);
        assert_eq!(TickPhase::Close.index(), 3);
        assert_eq!(TickPhase::from_index(3), TickPhase::Close);
        // Index ≥ 3 masks to close, like fe's `else` branch.
        assert_eq!(TickPhase::from_index(7), TickPhase::Close);
        assert_eq!(TickPhase::Intra2.name(), "intra2");
        assert_eq!(ReplayStatus::Completed.name(), "completed");
    }

    #[test]
    fn the_side_helpers_carry_fes_pnl_and_order_mapping() {
        assert_eq!(Side::Long.pnl_sign(), 1.0);
        assert_eq!(Side::Short.pnl_sign(), -1.0);
        assert_eq!(Side::Short.opening_order(), "sell");
        assert_eq!(Side::Short.closing_order(), "buy");
        assert_eq!(Side::Long.name(), "long");
    }

    #[test]
    fn pnl_at_mirrors_fes_calculate_position_pnl() {
        // Flat books are zero, not invented numbers.
        assert_eq!(ActivePosition::pnl_at(None, 123.0), (0.0, 0.0));
        let long = ActivePosition {
            symbol: "TCS".into(),
            exchange: "NSE".into(),
            side: Side::Long,
            quantity: 10.0,
            entry_price: 100.0,
            current_price: 100.0,
            unrealized_pnl: 0.0,
            unrealized_return_pct: 0.0,
            entry_timestamp: 0,
        };
        assert_eq!(ActivePosition::pnl_at(Some(&long), 110.0), (100.0, 10.0));
        let mut short = long.clone();
        short.side = Side::Short;
        assert_eq!(ActivePosition::pnl_at(Some(&short), 90.0), (100.0, 10.0));
    }

    #[test]
    fn the_ist_formatters_match_the_en_in_anchor_times() {
        let ts = ts_utc_2024_01_15_10h();
        // UTC 10:00 → IST 15:30 → 12-hour clock "03:30 pm".
        assert_eq!(clock_str(ts), "03:30:00 pm");
        assert_eq!(stamp_str(ts), "15 Jan, 03:30 pm");
        // Day rollover: UTC 2024-01-15 20:30 → IST 2024-01-16 02:00 am.
        assert_eq!(stamp_str(ts + 37_800), "16 Jan, 02:00 am");
        // Midnight IST renders hour 12, not 0: IST 00:00 = UTC 18:30 the
        // previous day.
        assert_eq!(
            clock_str(days_from_civil(2024, 1, 14) * 86_400 + 66_600),
            "12:00:00 am"
        );
        assert_eq!(stamp_str(days_from_civil(2024, 1, 14) * 86_400 + 66_600), "15 Jan, 12:00 am");
    }

    #[test]
    fn the_fill_model_keeps_fes_string_spellings() {
        assert_eq!(FillModel::from_name("bar_fill"), FillModel::BarFill);
        assert_eq!(FillModel::from_name("bar_fill_through"), FillModel::BarFillThrough);
        assert_eq!(FillModel::from_name("l2"), FillModel::L2);
        assert_eq!(FillModel::from_name("none"), FillModel::None);
        // Unknown spellings fall back to bar_fill (fe truthiness).
        assert_eq!(FillModel::from_name(""), FillModel::BarFill);
        assert_eq!(FillModel::L2.name(), "l2");
    }

    #[test]
    fn the_config_defaults_match_fes_constructor() {
        let config = ReplayConfig::default();
        assert_eq!(config.symbol, "TCS");
        assert_eq!(config.exchange, "NSE");
        assert_eq!(config.initial_capital, 100_000.0);
        assert_eq!(config.account_id, "acc-sandbox-primary");
        assert_eq!(config.fill_model, FillModel::BarFill);
        assert_eq!(config.speed, 1.0);
        assert_eq!(DEFAULT_SECONDARY_TIMEFRAME, "1h");
        assert_eq!(DEFAULT_QUANTITY, 10.0);
        assert_eq!(DEFAULT_FILL_FEE, 10.0);
        assert_eq!(DEFAULT_L2_SLIPPAGE_BPS, 2.0);
        assert_eq!(L2_HALF_SPREAD_FLOOR, 2.0e-4);
        assert_eq!(L2_RANGE_FACTOR, 0.05);
    }
}
