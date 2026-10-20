//! Run analytics (HBD-2026-DES-005 P3).
//!
//! The Rust port of honba-fe `core/simulator/analytics.ts`. Turns a completed
//! run into the numbers the analytics surface shows. The run payload
//! (the wire `BacktestMetrics`, mirrored here as [`RunMetrics`]) carries only
//! five fields — `total_return`, `sharpe`, `max_drawdown`, `net_pnl`, `trades` —
//! so everything else is either **derived** deterministically from those plus
//! the journal round trips ([`crate::pairing`]) and the run window, or reported
//! **unavailable**
//! (provenance [`MetricProvenance::Unavailable`]). Nothing here invents a value
//! the wire did not supply: every ribbon metric carries its provenance and,
//! when derived, the rule that produced it.
//!
//! The journal is fills, so "returns" are realised round-trip returns, not a
//! mark-to-market curve. That is why Sharpe (a periodic statistic) comes from
//! the payload while win rate, profit factor and exposure are computed from
//! trades, and why Sortino — which needs a *periodic* return series — is
//! unavailable.
//!
//! Money rule (per the port's DoD, mirrored from the design doc): fill costs
//! arrive in integer minor units on the wire and are converted to major-unit
//! floats **once**, at the [`crate::pairing`] boundary; everything here
//! keeps raw major-unit floats like fe, and presentation divides by nothing.
//! No wire `Money` value may leak this far — this module only speaks f64.

use std::collections::BTreeMap;

use crate::calendar::{civil_from_days, days_from_civil, IST_OFFSET_SECONDS};
use crate::pairing::{Direction, RoundTrip, RoundTripExitReason, RoundTripStatus};

/// The run payload's five wire metrics (the API's `BacktestMetrics`), as the
/// analytics read them. The app copies them over field for field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RunMetrics {
    /// Number of completed round trips.
    pub trades: u64,
    /// Net profit across the run.
    pub net_pnl: f64,
    /// Annualized Sharpe ratio.
    pub sharpe: f64,
    /// Maximum peak-to-trough drawdown, as a positive fraction.
    pub max_drawdown: f64,
    /// Total return as a fraction of initial capital.
    pub total_return: f64,
}

/// One benchmark price point: a bar's timestamp and close, all
/// [`benchmark_equity`] reads off a bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClosePoint {
    /// Bar timestamp, whole UTC seconds.
    pub time: i64,
    /// Close.
    pub close: f64,
}

/// The kind of metric and how a value renders for it (`MetricKind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetricKind {
    /// A fraction of 1, rendered as a signed percent.
    Percent,
    /// A positive drawdown fraction, rendered as a signed-off percent.
    Drawdown,
    /// A dimensionless ratio, two decimals.
    Ratio,
    /// An integer count.
    Count,
    /// A currency amount (major units).
    Currency,
}

/// Where a ribbon metric's value came from (`MetricProvenance`) — the honesty
/// invariant: `wire` is echoed, `derived` explains its rule, `unavailable`
/// never invents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetricProvenance {
    /// Reported by the run payload, echoed exactly.
    Wire,
    /// Computed here from the payload + window + journal.
    Derived,
    /// The contract cannot support this number.
    Unavailable,
}

impl MetricProvenance {
    /// fe's wire spelling, shown next to every ribbon metric.
    pub fn as_str(self) -> &'static str {
        match self {
            MetricProvenance::Wire => "wire",
            MetricProvenance::Derived => "derived",
            MetricProvenance::Unavailable => "unavailable",
        }
    }
}

/// One metrics-ribbon entry (`RibbonMetric`).
#[derive(Clone, Debug, PartialEq)]
pub struct RibbonMetric {
    /// Stable key, in the exact order [`derive_ribbon_metrics`] emits.
    pub key: &'static str,
    /// Display label.
    pub label: &'static str,
    /// `None` when the metric cannot be computed (provenance `unavailable`).
    pub value: Option<f64>,
    /// Presentation kind.
    pub kind: MetricKind,
    /// How the value was obtained.
    pub provenance: MetricProvenance,
    /// How the value was obtained, or why it is unavailable.
    pub note: String,
}

/// The run window, `YYYY-MM-DD` (inclusive start, exclusive end), when known.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunWindow {
    /// Inclusive `YYYY-MM-DD` start.
    pub start: String,
    /// Exclusive `YYYY-MM-DD` end.
    pub end: String,
}

/// Everything [`derive_ribbon_metrics`] and friends derive from
/// (`AnalyticsInput`). Borrows keep the P4 store's memos cheap; nothing is
/// computed from preview bars here — the preview is a chart *input* only,
/// never a metrics source.
#[derive(Clone, Copy, Debug)]
pub struct AnalyticsInput<'a> {
    /// The run payload's five wire metrics, when the run reported them.
    pub metrics: Option<&'a RunMetrics>,
    /// The journal's derived round trips ([`crate::pairing`]).
    pub trips: &'a [RoundTrip],
    /// The run window, when the controls resolved one.
    pub window: Option<&'a RunWindow>,
    /// Initial capital; fe's hard-coded ₹100,000 default fills in when unset.
    pub initial_capital: Option<f64>,
}

/// Seconds per day; fe's `MS_PER_DAY` (a misnomer there — these are seconds).
const SECONDS_PER_DAY: f64 = 86_400.0;
/// fe's annualisation constant: `365.25` days per year.
const DAYS_PER_YEAR: f64 = 365.25;

/// `num / den`, or `None` when the ratio is undefined (zero/absent
/// denominator, or either operand non-finite) (`safeRatio`).
pub fn safe_ratio(num: Option<f64>, den: Option<f64>) -> Option<f64> {
    let (num, den) = (num?, den?);
    if !num.is_finite() || !den.is_finite() || den == 0.0 {
        return None;
    }
    let ratio = num / den;
    ratio.is_finite().then_some(ratio)
}

/// `YYYY-MM-DD` → whole UTC seconds of that midnight (`secondsOfDate`).
///
/// Like fe's `Date.parse(`${date}T00:00:00Z`)`, but without `Intl`: the string
/// must be exactly `date`-shaped (three integer parts), otherwise `None`.
fn seconds_of_date(date: Option<&str>) -> Option<i64> {
    let date = date?;
    if date.is_empty() {
        return None;
    }
    let mut parts = date.split('-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(days_from_civil(year, month, day) * 86_400)
}

/// Whole years between two `YYYY-MM-DD` dates (inclusive start, exclusive
/// end) (`yearsBetween`).
pub fn years_between(start: Option<&str>, end: Option<&str>) -> Option<f64> {
    let from = seconds_of_date(start)?;
    let to = seconds_of_date(end)?;
    if to <= from {
        return None;
    }
    Some((to - from) as f64 / SECONDS_PER_DAY / DAYS_PER_YEAR)
}

/// CAGR from a period total return: `(1 + r)^(1/years) − 1`
/// (`annualisedReturn`).
///
/// A total return of `−100%` or worse is fully destroyed capital, reported as
/// `−1`; a non-positive window has no annualisation and yields `None`.
pub fn annualised_return(total_return: Option<f64>, years: Option<f64>) -> Option<f64> {
    let total_return = total_return?;
    if !total_return.is_finite() {
        return None;
    }
    if total_return <= -1.0 {
        return Some(-1.0);
    }
    let years = years?;
    if !years.is_finite() || years <= 0.0 {
        return None;
    }
    let cagr = (1.0 + total_return).powf(1.0 / years) - 1.0;
    cagr.is_finite().then_some(cagr)
}

/// Trade-level statistics over the *derived* round trips (`RoundTripStats`,
/// `roundTripStats`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoundTripStats {
    /// Round trips that have realised at least part of their position.
    pub closed: u32,
    /// Round trips still holding inventory at the run's end.
    pub open: u32,
    /// Closed trips with positive realised PnL.
    pub wins: u32,
    /// Closed trips with negative realised PnL.
    pub losses: u32,
    /// Closed trips with exactly zero realised PnL.
    pub breakeven: u32,
    /// Sum of positive realised PnL.
    pub gross_profit: f64,
    /// Sum of loss magnitudes.
    pub gross_loss: f64,
    /// Sum of realised PnL over closed trips.
    pub realized_pnl: f64,
    /// Win rate, `wins / closed`; `None` with nothing closed.
    pub win_rate: Option<f64>,
    /// `gross_profit / gross_loss`; `None` when nothing lost (never ∞).
    pub profit_factor: Option<f64>,
    /// `gross_profit / wins`; `None` with no wins.
    pub avg_win: Option<f64>,
    /// `gross_loss / losses`; `None` with no losses.
    pub avg_loss: Option<f64>,
}

/// [`RoundTripStats`] over the journal's round trips (`roundTripStats`).
pub fn round_trip_stats(trips: &[RoundTrip]) -> RoundTripStats {
    let mut wins = 0u32;
    let mut losses = 0u32;
    let mut breakeven = 0u32;
    let mut gross_profit = 0.0f64;
    let mut gross_loss = 0.0f64;
    let mut realized_pnl = 0.0f64;
    let mut open = 0u32;

    for trip in trips {
        let Some(pnl) = trip.realized_pnl else {
            open += 1;
            continue;
        };
        realized_pnl += pnl;
        if pnl > 0.0 {
            wins += 1;
            gross_profit += pnl;
        } else if pnl < 0.0 {
            losses += 1;
            gross_loss += -pnl;
        } else {
            breakeven += 1;
        }
    }

    let closed = wins + losses + breakeven;
    RoundTripStats {
        closed,
        open,
        wins,
        losses,
        breakeven,
        gross_profit,
        gross_loss,
        realized_pnl,
        win_rate: (closed > 0).then(|| wins as f64 / closed as f64),
        profit_factor: (gross_loss > 0.0).then(|| gross_profit / gross_loss),
        avg_win: (wins > 0).then(|| gross_profit / wins as f64),
        avg_loss: (losses > 0).then(|| gross_loss / losses as f64),
    }
}

/// A half-open holding interval `[start, end)` in UTC seconds (`Interval`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interval {
    /// Inclusive start, UTC seconds.
    pub start: i64,
    /// Exclusive end, UTC seconds.
    pub end: i64,
}

/// Merge overlapping/adjacent `[start, end)` intervals into a minimal cover
/// (`mergeIntervals`). Empty or inverted intervals are dropped.
pub fn merge_intervals(intervals: &[Interval]) -> Vec<Interval> {
    let mut sorted: Vec<Interval> = intervals
        .iter()
        .filter(|interval| interval.end > interval.start)
        .copied()
        .collect();
    sorted.sort_by_key(|interval| interval.start);
    let mut merged: Vec<Interval> = Vec::with_capacity(sorted.len());
    for interval in sorted {
        match merged.last_mut() {
            Some(last) if interval.start <= last.end => {
                last.end = last.end.max(interval.end);
            }
            _ => merged.push(interval),
        }
    }
    merged
}

/// Fraction of the run window with at least one open position (union of the
/// round trips' holding intervals, clipped to the window). A position still
/// open at the end is counted to the window's end. `None` without a usable
/// window (`exposureFraction`).
pub fn exposure_fraction(input: &AnalyticsInput<'_>) -> Option<f64> {
    let window = input.window?;
    let from = seconds_of_date(Some(&window.start))?;
    let to = seconds_of_date(Some(&window.end))?;
    if to <= from {
        return None;
    }
    let span = to - from;
    let mut intervals: Vec<Interval> = Vec::new();
    for trip in input.trips {
        let start = from.max(trip.entry_ts);
        let end = to.min(trip.exit_ts.unwrap_or(to));
        if end > start {
            intervals.push(Interval { start, end });
        }
    }
    let covered: f64 = merge_intervals(&intervals)
        .iter()
        .map(|interval| (interval.end - interval.start) as f64)
        .sum();
    Some((covered / span as f64).clamp(0.0, 1.0))
}

// ---------------------------------------------------------------------------
// Equity, monthly returns & the trade journal table
// ---------------------------------------------------------------------------

/// How the realised equity curve is sampled (`EquitySampling`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquitySampling {
    /// One point per calendar day of the run window.
    Daily,
    /// One point per closed round trip (no window known).
    Trade,
}

impl EquitySampling {
    /// fe's wire spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            EquitySampling::Daily => "daily",
            EquitySampling::Trade => "trade",
        }
    }
}

/// Realised equity series (`EquitySeries`).
#[derive(Clone, Debug, PartialEq)]
pub struct EquitySeries {
    /// Index-aligned equity values; `equity[0]` is the starting capital.
    pub equity: Vec<f64>,
    /// Sampling rule used.
    pub sampling: EquitySampling,
    /// Starting capital.
    pub initial: f64,
    /// Last value (fe's `final`; `final` is a Rust keyword).
    pub final_value: f64,
    /// Number of steps (days for daily, closed trips + 1 for trade).
    pub steps: usize,
}

struct DayGrid {
    start_sec: i64,
    /// Whole days between the window edges.
    days: i64,
}

/// The run window as a whole-day grid (`dayGrid`); `None` when unusable.
fn day_grid(window: Option<&RunWindow>) -> Option<DayGrid> {
    let start_sec = seconds_of_date(window.map(|w| w.start.as_str()))?;
    let end_sec = seconds_of_date(window.map(|w| w.end.as_str()))?;
    if end_sec <= start_sec {
        return None;
    }
    let days = ((end_sec - start_sec) as f64 / SECONDS_PER_DAY).round() as i64;
    (days > 0).then_some(DayGrid { start_sec, days })
}

/// Initial capital for a run, defaulting to fe's ₹100,000 (`capitalOf`).
fn capital_of(input: &AnalyticsInput<'_>) -> f64 {
    match input.initial_capital {
        Some(value) if value.is_finite() && value > 0.0 => value,
        _ => 100_000.0,
    }
}

/// Realised equity from the journal (`equitySeries`).
///
/// With a run window the curve is sampled **daily** (one point per calendar
/// day, carrying the last realised value forward), which keeps it
/// time-aligned; without one it is sampled per closed round trip. This is
/// *realised* equity — the journal is fills, so no mark-to-market on open
/// positions is invented.
pub fn equity_series(input: &AnalyticsInput<'_>) -> EquitySeries {
    let initial = capital_of(input);
    let mut closed: Vec<&RoundTrip> = input
        .trips
        .iter()
        .filter(|trip| trip.realized_pnl.is_some() && trip.exit_ts.is_some())
        .collect();
    closed.sort_by_key(|trip| trip.exit_ts.unwrap_or(i64::MIN));

    if let Some(grid) = day_grid(input.window) {
        let mut equity: Vec<f64> = vec![initial];
        let mut cumulative = 0.0;
        let mut cursor = 0usize;
        for day in 1..=grid.days {
            let day_end = grid.start_sec + day * SECONDS_PER_DAY as i64;
            while let Some(trip) = closed.get(cursor) {
                if trip.exit_ts.unwrap_or(i64::MIN) >= day_end {
                    break;
                }
                cumulative += trip.realized_pnl.unwrap_or(0.0);
                cursor += 1;
            }
            equity.push(initial + cumulative);
        }
        return EquitySeries {
            initial,
            final_value: *equity.last().unwrap_or(&initial),
            sampling: EquitySampling::Daily,
            steps: equity.len(),
            equity,
        };
    }

    let mut equity: Vec<f64> = vec![initial];
    let mut cumulative = 0.0;
    for trip in &closed {
        cumulative += trip.realized_pnl.unwrap_or(0.0);
        equity.push(initial + cumulative);
    }
    EquitySeries {
        initial,
        final_value: *equity.last().unwrap_or(&initial),
        sampling: EquitySampling::Trade,
        steps: equity.len(),
        equity,
    }
}

/// Benchmark equity from a benchmark bar series: buy-and-hold, normalised so
/// it starts at the same capital as the run and shares the equity curve's
/// sampling (`benchmarkEquity`).
///
/// Returns `None` when the bars cannot be aligned to the equity curve (no
/// window, a different sampling, or no bars), so the caller shows a labelled
/// absence rather than a misleading overlay.
pub fn benchmark_equity(
    bars: &[ClosePoint],
    equity: &EquitySeries,
    input: &AnalyticsInput<'_>,
) -> Option<Vec<f64>> {
    let mut usable: Vec<&ClosePoint> = bars
        .iter()
        .filter(|bar| bar.close.is_finite() && bar.close > 0.0)
        .collect();
    if usable.is_empty() {
        return None;
    }

    let grid = day_grid(input.window)?;
    // Only a daily equity curve can share an axis with a bar series honestly;
    // the trade-sampled fallback has no daily grid to align to, so no overlay
    // is offered.
    if equity.sampling != EquitySampling::Daily {
        return None;
    }

    usable.sort_by_key(|bar| bar.time);
    let first = usable[0].close;
    let mut out: Vec<f64> = vec![equity.initial];
    let mut cursor = 0usize;
    let mut last_close = first;
    for day in 1..=grid.days {
        let day_end = grid.start_sec + day * SECONDS_PER_DAY as i64;
        while let Some(bar) = usable.get(cursor) {
            if bar.time >= day_end {
                break;
            }
            last_close = bar.close;
            cursor += 1;
        }
        out.push(equity.initial * (last_close / first));
    }
    Some(out)
}

// ---------------------------------------------------------------------------
// Monthly heatmap (IST bucketing, matching the NSE reporting convention)
// ---------------------------------------------------------------------------

/// One month × year cell of the heatmap (`HeatmapCell`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeatmapCell {
    /// IST calendar year.
    pub year: i64,
    /// 0 = January (fe's `getUTCMonth` convention, kept for parity).
    pub month: u32,
    /// Realised PnL of closed round trips that exited in this month.
    pub pnl: f64,
    /// PnL ÷ (starting capital + realised PnL before the month).
    pub return_pct: Option<f64>,
    /// Closed round trips whose exits landed in this month.
    pub trades: u32,
}

/// The month × year heatmap (`MonthlyHeatmap`).
#[derive(Clone, Debug, PartialEq)]
pub struct MonthlyHeatmap {
    /// Cells sorted by (year, month).
    pub cells: Vec<HeatmapCell>,
    /// Years present, ascending.
    pub years: Vec<i64>,
    /// Sum of every cell's `pnl` (equals realised PnL over closed trips).
    pub total_pnl: f64,
    /// `total_pnl / initial capital`.
    pub total_return_pct: Option<f64>,
}

/// IST year/month of a UTC-seconds timestamp (`istYearMonth`); month is
/// 0-based (0 = January), as fe's `getUTCMonth` reported.
pub fn ist_year_month(ts: i64) -> (i64, u32) {
    let (year, month, _) = civil_from_days((ts + IST_OFFSET_SECONDS).div_euclid(86_400));
    (year, month - 1)
}

/// Realised PnL bucketed by IST calendar month, for the month × year heatmap
/// (`monthlyHeatmap`). `return_pct` compounds on running realised equity.
///
/// Copy note for P5: the heatmap footer must carry fe's reconciliation
/// sentence — the realised total may differ from the payload's mark-to-market
/// total — because the journal is fills, not a curve.
pub fn monthly_heatmap(input: &AnalyticsInput<'_>) -> MonthlyHeatmap {
    let initial = capital_of(input);
    let mut closed: Vec<&RoundTrip> = input
        .trips
        .iter()
        .filter(|trip| trip.realized_pnl.is_some() && trip.exit_ts.is_some())
        .collect();
    closed.sort_by_key(|trip| trip.exit_ts.unwrap_or(i64::MIN));

    let mut by_month: BTreeMap<(i64, u32), HeatmapCell> = BTreeMap::new();
    let mut cumulative = 0.0;
    let mut total_pnl = 0.0;
    for trip in &closed {
        let (year, month) = ist_year_month(trip.exit_ts.unwrap_or(0));
        let cell = by_month.entry((year, month)).or_insert(HeatmapCell {
            year,
            month,
            pnl: 0.0,
            return_pct: None,
            trades: 0,
        });
        let month_start_equity = initial + cumulative;
        cell.pnl += trip.realized_pnl.unwrap_or(0.0);
        cell.trades += 1;
        cell.return_pct = safe_ratio(Some(cell.pnl), Some(month_start_equity));
        cumulative += trip.realized_pnl.unwrap_or(0.0);
        total_pnl += trip.realized_pnl.unwrap_or(0.0);
    }

    let cells: Vec<HeatmapCell> = by_month.into_values().collect();
    let years: Vec<i64> = {
        let mut years: Vec<i64> = cells.iter().map(|cell| cell.year).collect();
        years.dedup(); // cells are sorted, so one dedup removes repeats
        years
    };
    MonthlyHeatmap {
        total_return_pct: safe_ratio(Some(total_pnl), Some(initial)),
        cells,
        years,
        total_pnl,
    }
}

// ---------------------------------------------------------------------------
// Trade journal table + CSV export
// ---------------------------------------------------------------------------

/// One row of the journal table (`JournalRow`) — a [`RoundTrip`] flattened to
/// its reporting fields.
#[derive(Clone, Debug, PartialEq)]
pub struct JournalRow {
    /// Stable trip id.
    pub id: String,
    /// `SYMBOL.EXCHANGE` instrument id.
    pub instrument: String,
    /// Direction the trip opened with.
    pub direction: Direction,
    /// Whether the run ended with the position closed.
    pub status: RoundTripStatus,
    /// First fill's timestamp, UTC seconds.
    pub entry_ts: i64,
    /// Last exit's timestamp; `None` for a trip that never closed.
    pub exit_ts: Option<i64>,
    /// Entry size.
    pub quantity: f64,
    /// Inventory still held at the run's end.
    pub open_quantity: f64,
    /// Weighted-average entry price.
    pub avg_entry: f64,
    /// Weighted-average exit price; `None` until something closed.
    pub avg_exit: Option<f64>,
    /// Realised PnL; `None` for an open trip.
    pub realized_pnl: Option<f64>,
    /// `realized_pnl / (avg_entry × closed qty)`.
    pub return_pct: Option<f64>,
    /// Recorded fill costs, entry + exit.
    pub fees: f64,
    /// Derived exit reason (never a strategy-level guess).
    pub exit_reason: RoundTripExitReason,
}

/// Flatten the round trips into the journal table's rows (`journalRows`) —
/// the same rows the CSV export serialises (design wart W5).
pub fn journal_rows(trips: &[RoundTrip]) -> Vec<JournalRow> {
    trips
        .iter()
        .map(|trip| JournalRow {
            id: trip.id.clone(),
            instrument: trip.instrument.clone(),
            direction: trip.direction,
            status: trip.status,
            entry_ts: trip.entry_ts,
            exit_ts: trip.exit_ts,
            quantity: trip.quantity,
            open_quantity: trip.open_quantity,
            avg_entry: trip.avg_entry,
            avg_exit: trip.avg_exit,
            realized_pnl: trip.realized_pnl,
            return_pct: trip.return_pct,
            fees: trip.fees,
            exit_reason: trip.exit_reason,
        })
        .collect()
}

/// One CSV column: its header and how to read its value off a row
/// (`CsvColumn`). Value columns are pure functions so the tabular CSV holds
/// exactly the visible journal rows.
pub struct CsvColumn<Row> {
    /// Header cell.
    pub header: &'static str,
    /// The field rendered into the CSV; an empty string is an absent value.
    pub value: fn(&Row) -> String,
}

/// RFC-4180 field: quoted when it holds a comma, quote, CR or LF; embedded
/// quotes doubled (`csvField`).
pub fn csv_field(value: &str) -> String {
    let needs_quoting = value
        .chars()
        .any(|c| c == ',' || c == '"' || c == '\r' || c == '\n');
    if needs_quoting {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Serialise rows to CSV (`toCsv`): CRLF line endings, so Excel opens it
/// cleanly, header first.
pub fn to_csv<Row>(rows: &[Row], columns: &[CsvColumn<Row>]) -> String {
    let header: Vec<String> = columns
        .iter()
        .map(|column| csv_field(column.header))
        .collect();
    let mut lines: Vec<String> = vec![header.join(",")];
    for row in rows {
        let fields: Vec<String> = columns
            .iter()
            .map(|column| csv_field(&(column.value)(row)))
            .collect();
        lines.push(fields.join(","));
    }
    lines.join("\r\n")
}

/// Pad an unsigned integer to a fixed width (`2024 → "2024"`, `3 → "03"`).
fn pad(value: i64, width: usize) -> String {
    let digits = value.to_string();
    if digits.len() >= width {
        digits
    } else {
        format!("{}{}", "0".repeat(width - digits.len()), digits)
    }
}

/// UTC `YYYY-MM-DD HH:MM` for a UTC-seconds timestamp, `—` when absent
/// (`formatTimestamp`). Pure civil maths, no `Intl`. (fe also guarded
/// `NaN`/non-finite inputs — a typed `i64` cannot be either, so the guard is
/// structural here and the `Some(a)`/`None` shape absorbs it.)
pub fn format_timestamp(ts: Option<i64>) -> String {
    let Some(ts) = ts else {
        return "—".to_string();
    };
    let days = ts.div_euclid(86_400);
    let seconds_of_day = ts.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{}-{}-{} {}:{}",
        year,
        pad(month as i64, 2),
        pad(day as i64, 2),
        pad(seconds_of_day / 3_600, 2),
        pad((seconds_of_day % 3_600) / 60, 2),
    )
}

// ---------------------------------------------------------------------------
// Benchmark instrument
// ---------------------------------------------------------------------------

/// Best-effort `SYMBOL.EXCHANGE` id for the panel's benchmark display name
/// (`benchmarkInstrumentId`).
///
/// The run contract has no benchmark and the bars route wants an instrument
/// id, so a display name like `NIFTY 50` is normalised to the symbol
/// convention the repo uses (`NIFTY50.NSE`). The id is only ever used to
/// *attempt* a fetch: if the store has no bars for it the overlay is simply
/// absent and labelled, never fabricated.
pub fn benchmark_instrument_id(name: Option<&str>, exchange: Option<&str>) -> Option<String> {
    let name = name?;
    let symbol: String = name
        .trim()
        .to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    if symbol.is_empty() {
        return None;
    }
    let exchange = exchange
        .map(str::trim)
        .map(str::to_uppercase)
        .filter(|code| !code.is_empty())
        .unwrap_or_else(|| "NSE".to_string());
    Some(format!("{symbol}.{exchange}"))
}

// ---------------------------------------------------------------------------
// Ribbon
// ---------------------------------------------------------------------------

fn wire_metric(
    key: &'static str,
    label: &'static str,
    kind: MetricKind,
    value: Option<f64>,
) -> RibbonMetric {
    RibbonMetric {
        key,
        label,
        kind,
        value: value.filter(|v| v.is_finite()),
        provenance: MetricProvenance::Wire,
        note: "Reported by the run payload.".to_string(),
    }
}

fn two_decimals(value: f64) -> String {
    format!("{value:.2}")
}

/// The metrics ribbon (`deriveRibbonMetrics`), in fe's exact key order:
/// `total_return` (wire), `cagr` (derived), `sharpe` (wire), `sortino`
/// (unavailable — never a fabricated number), `calmar` (derived),
/// `max_drawdown` (wire), `win_rate` (derived), `profit_factor` (derived),
/// `trades` (wire), `exposure` (derived or unavailable).
pub fn derive_ribbon_metrics(input: &AnalyticsInput<'_>) -> Vec<RibbonMetric> {
    let metrics = input.metrics;
    let years = years_between(
        input.window.map(|w| w.start.as_str()),
        input.window.map(|w| w.end.as_str()),
    );
    let cagr = annualised_return(metrics.map(|m| m.total_return), years);
    let max_drawdown = metrics
        .map(|m| m.max_drawdown)
        .filter(|value| value.is_finite());
    let stats = round_trip_stats(input.trips);
    let exposure = exposure_fraction(input);

    let window_note = match years {
        None => "No run window; annualisation is undefined.".to_string(),
        Some(years) => format!(
            "Compounded from total return over the {}-year run window.",
            two_decimals(years)
        ),
    };
    let cagr_note = match cagr {
        None => window_note.clone(),
        Some(_) => format!("{window_note} Formed as (1 + total return)^(1/years) − 1."),
    };

    let ribbon: Vec<RibbonMetric> = vec![
        wire_metric("total_return", "Total Return", MetricKind::Percent, metrics.map(|m| m.total_return)),
        RibbonMetric {
            key: "cagr",
            label: "CAGR",
            kind: MetricKind::Percent,
            value: cagr,
            provenance: if cagr.is_none() {
                MetricProvenance::Unavailable
            } else {
                MetricProvenance::Derived
            },
            note: cagr_note,
        },
        wire_metric("sharpe", "Sharpe", MetricKind::Ratio, metrics.map(|m| m.sharpe)),
        RibbonMetric {
            key: "sortino",
            label: "Sortino",
            kind: MetricKind::Ratio,
            value: None,
            provenance: MetricProvenance::Unavailable,
            note: "Needs a periodic return series; the run payload carries none and the journal gives trade-level returns only.".to_string(),
        },
        RibbonMetric {
            key: "calmar",
            label: "Calmar",
            kind: MetricKind::Ratio,
            value: cagr
                .zip(max_drawdown)
                .filter(|(_, drawdown)| *drawdown > 0.0)
                .map(|(cagr, drawdown)| cagr / drawdown),
            provenance: MetricProvenance::Derived,
            note: "CAGR ÷ |max drawdown|.".to_string(),
        },
        wire_metric("max_drawdown", "Max Drawdown", MetricKind::Drawdown, metrics.map(|m| m.max_drawdown)),
        RibbonMetric {
            key: "win_rate",
            label: "Win Rate",
            kind: MetricKind::Percent,
            value: stats.win_rate,
            provenance: MetricProvenance::Derived,
            note: format!(
                "Winning round trips ÷ closed round trips ({}/{}, journal).",
                stats.wins, stats.closed
            ),
        },
        RibbonMetric {
            key: "profit_factor",
            label: "Profit Factor",
            kind: MetricKind::Ratio,
            value: stats.profit_factor,
            provenance: MetricProvenance::Derived,
            note: if stats.gross_loss > 0.0 {
                "Gross profit ÷ gross loss over closed round trips (journal).".to_string()
            } else {
                "No losing round trips, so profit factor is undefined (not infinite).".to_string()
            },
        },
        wire_metric(
            "trades",
            "Total Trades",
            MetricKind::Count,
            metrics.map(|m| m.trades as f64),
        ),
        RibbonMetric {
            key: "exposure",
            label: "Exposure",
            kind: MetricKind::Percent,
            value: exposure,
            provenance: if exposure.is_none() {
                MetricProvenance::Unavailable
            } else {
                MetricProvenance::Derived
            },
            note: if exposure.is_none() {
                "Needs a run window; exposure is time in market ÷ window.".to_string()
            } else {
                "Time with an open position ÷ run window (journal fills).".to_string()
            },
        },
    ];
    ribbon
}

/// Presentation helper; `None` renders as an em dash (`formatMetricValue`).
pub fn format_metric_value(value: Option<f64>, kind: MetricKind) -> String {
    let Some(value) = value else {
        return "—".to_string();
    };
    if !value.is_finite() {
        return "—".to_string();
    }
    match kind {
        MetricKind::Percent => {
            let sign = if value >= 0.0 { "+" } else { "" };
            format!("{sign}{:.2}%", value * 100.0)
        }
        // The wire reports drawdown as a positive fraction; it always reads
        // negative.
        MetricKind::Drawdown => format!("-{:.2}%", value.abs() * 100.0),
        MetricKind::Ratio => format!("{value:.2}"),
        MetricKind::Count => format!("{}", value.round()),
        // fe formatted via toLocaleString; there is no Intl here, so the
        // thousands-grouped en-US shape ("1,234.50") is reproduced inline.
        MetricKind::Currency => format_currency(value),
    }
}

/// `toLocaleString({minimumFractionDigits: 2, maximumFractionDigits: 2})`,
/// locale-free: thousands separators every three integer digits, always two
/// decimals, sign preserved.
fn format_currency(value: f64) -> String {
    let negative = value < 0.0;
    let scaled = (value.abs() * 100.0).round() / 100.0;
    let integer_part = scaled.trunc() as i64;
    let decimals = ((scaled.abs() - integer_part as f64) * 100.0).round() as i64;
    let grouped = integer_part.to_string();
    let mut out = String::new();
    // 86_400 remainder arithmetic, clippy's is_multiple_of form; the loop is
    // index-parallel so `(len - index) % 3 == 0` means "insert before digit".
    for (index, digit) in grouped.chars().enumerate() {
        if index > 0 && (grouped.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    format!(
        "{}{}.{}",
        if negative { "-" } else { "" },
        out,
        pad(decimals, 2)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pairing::pair_trades;
    use honba_entities::{Currency, Money as WireMoney, Trade};
    use honba_messages::{
        Exchange, InstrumentId, OrderId, OrderSide, UnixNanos as WireUnixNanos,
    };

    const JAN_2: i64 = 1_704_153_600; // 2024-01-02 00:00 UTC
    const HOURS: i64 = 3_600;
    const DAYS: i64 = 86_400;

    fn fill(side: OrderSide, quantity: f64, price: f64, at: i64, costs: f64) -> Trade {
        Trade::new(
            OrderId::new("o"),
            InstrumentId::new("TCS", Exchange::new("NSE")),
            side,
            quantity,
            price,
            Currency::Inr,
            WireUnixNanos::from_u64(at as u64 * 1_000_000_000),
            WireUnixNanos::from_u64(at as u64 * 1_000_000_000),
        )
        .with_costs(WireMoney::from_major_f64(costs, Currency::Inr).expect("valid costs"))
    }

    /// Two winners, one loser — a small journal the metrics can be checked
    /// against (fe's `FIXTURE_TRADES`).
    fn fixture_trades() -> Vec<Trade> {
        vec![
            fill(OrderSide::Buy, 10.0, 100.0, JAN_2, 1.0),                // +100
            fill(OrderSide::Sell, 10.0, 110.0, JAN_2 + 10 * DAYS, 1.0),
            fill(OrderSide::Buy, 10.0, 100.0, JAN_2 + 31 * DAYS, 1.0),    // −50
            fill(OrderSide::Sell, 10.0, 95.0, JAN_2 + 40 * DAYS, 1.0),
            fill(OrderSide::Buy, 10.0, 100.0, JAN_2 + 60 * DAYS, 1.0),    // +50
            fill(OrderSide::Sell, 10.0, 105.0, JAN_2 + 70 * DAYS, 1.0),
        ]
    }

    fn fixture_metrics() -> RunMetrics {
        RunMetrics {
            total_return: 0.1,
            sharpe: 1.25,
            max_drawdown: 0.2,
            net_pnl: 100.0,
            trades: 6,
        }
    }

    fn window() -> RunWindow {
        RunWindow {
            start: "2024-01-02".to_string(),
            end: "2024-04-02".to_string(),
        }
    }

    fn input<'a>(
        metrics: Option<&'a RunMetrics>,
        trips: &'a [RoundTrip],
        window: Option<&'a RunWindow>,
        initial_capital: Option<f64>,
    ) -> AnalyticsInput<'a> {
        AnalyticsInput {
            metrics,
            trips,
            window,
            initial_capital,
        }
    }

    // --- ratio & window helpers ------------------------------------------------

    #[test]
    fn safe_ratio_refuses_a_zero_denominator() {
        assert_eq!(safe_ratio(Some(1.0), Some(2.0)), Some(0.5));
        assert_eq!(safe_ratio(Some(1.0), Some(0.0)), None);
        assert_eq!(safe_ratio(None, Some(2.0)), None);
        assert_eq!(safe_ratio(Some(f64::INFINITY), Some(2.0)), None);
    }

    #[test]
    fn years_between_counts_whole_window_years_and_rejects_inverted_windows() {
        assert!((years_between(Some("2024-01-01"), Some("2025-01-01")).unwrap() - 366.0 / 365.25).abs() < 1e-6);
        assert_eq!(years_between(Some("2024-01-02"), Some("2024-01-02")), None);
        assert!((years_between(Some("2024-01-02"), Some("2024-04-02")).unwrap() - 91.0 / 365.25).abs() < 1e-6);
        assert_eq!(years_between(None, Some("2024-01-02")), None);
        assert_eq!(years_between(Some("not-a-date"), Some("2024-04-02")), None);
    }

    #[test]
    fn annualised_return_compounds_clamps_a_total_wipeout_and_refuses_no_window() {
        assert!((annualised_return(Some(0.0), Some(1.0)).unwrap()) < 1e-9);
        assert!((annualised_return(Some(0.1), Some(1.0)).unwrap() - 0.1).abs() < 1e-9);
        assert!((annualised_return(Some(0.44), Some(2.0)).unwrap() - 0.2).abs() < 1e-6);
        assert_eq!(annualised_return(Some(-1.0), Some(1.0)), Some(-1.0));
        assert_eq!(annualised_return(Some(0.1), None), None);
        assert_eq!(annualised_return(Some(0.1), Some(0.0)), None);
        assert_eq!(annualised_return(None, Some(1.0)), None);
    }

    // --- roundTripStats --------------------------------------------------------

    #[test]
    fn counts_wins_losses_gross_pnl_and_profit_factor_from_the_journal() {
        let trips = pair_trades(&fixture_trades()).trips;
        let stats = round_trip_stats(&trips);
        assert_eq!(stats.closed, 3);
        assert_eq!(stats.wins, 2);
        assert_eq!(stats.losses, 1);
        assert_eq!(stats.breakeven, 0);
        // Each trip nets its move less the two ₹1 fill costs: 100−2, −50−2, 50−2.
        assert!((stats.gross_profit - 146.0).abs() < 1e-6);
        assert!((stats.gross_loss - 52.0).abs() < 1e-6);
        assert!((stats.win_rate.unwrap() - 2.0 / 3.0).abs() < 1e-6);
        assert!((stats.profit_factor.unwrap() - 146.0 / 52.0).abs() < 1e-6);
        assert!((stats.realized_pnl - 94.0).abs() < 1e-6);
    }

    #[test]
    fn reports_open_trips_separately_and_returns_null_ratios_with_nothing_closed() {
        let open = pair_trades(&[fill(OrderSide::Buy, 10.0, 100.0, JAN_2, 0.0)]).trips;
        let stats = round_trip_stats(&open);
        assert_eq!(stats.open, 1);
        assert_eq!(stats.closed, 0);
        assert_eq!(stats.win_rate, None);
        assert_eq!(stats.profit_factor, None);
    }

    #[test]
    fn reports_profit_factor_null_when_nothing_lost_never_infinity() {
        let winner = pair_trades(&[
            fill(OrderSide::Buy, 10.0, 100.0, JAN_2, 0.0),
            fill(OrderSide::Sell, 10.0, 120.0, JAN_2 + HOURS, 0.0),
        ])
        .trips;
        let stats = round_trip_stats(&winner);
        assert_eq!(stats.profit_factor, None);
        assert_eq!(stats.win_rate, Some(1.0));
    }

    // --- exposure --------------------------------------------------------------

    #[test]
    fn merges_overlapping_intervals_and_clips_them_to_the_window_shape() {
        let merged = merge_intervals(&[
            Interval { start: 0, end: 10 },
            Interval { start: 5, end: 20 },
            Interval { start: 30, end: 40 },
        ]);
        assert_eq!(
            merged,
            vec![Interval { start: 0, end: 20 }, Interval { start: 30, end: 40 }]
        );
        assert!(merge_intervals(&[]).is_empty());
    }

    #[test]
    fn exposure_is_the_fraction_of_the_window_with_an_open_position() {
        let trips = pair_trades(&fixture_trades()).trips;
        let window = window();
        let exposure = exposure_fraction(&input(
            Some(&fixture_metrics()),
            &trips,
            Some(&window),
            Some(100_000.0),
        ))
        .expect("window is usable");
        assert!(exposure > 0.0);
        assert!(exposure <= 1.0);
    }

    #[test]
    fn exposure_is_null_without_a_run_window_and_zero_with_no_trips() {
        let trips = pair_trades(&fixture_trades()).trips;
        let metrics = fixture_metrics();
        assert_eq!(
            exposure_fraction(&input(Some(&metrics), &trips, None, Some(100_000.0))),
            None
        );
        assert_eq!(
            exposure_fraction(&input(Some(&metrics), &[], Some(&window()), Some(100_000.0))),
            Some(0.0)
        );
    }

    #[test]
    fn exposure_counts_a_position_still_open_at_the_end_up_to_the_window_end() {
        let open_trip = pair_trades(&[fill(OrderSide::Buy, 10.0, 100.0, JAN_2, 0.0)]).trips;
        let metrics = fixture_metrics();
        let window = window();
        let exposure = exposure_fraction(&input(Some(&metrics), &open_trip, Some(&window), Some(100_000.0)))
            .expect("window is usable");
        // entry 2024-01-02 → window end 2024-04-02 covers the whole window.
        assert!((exposure - 1.0).abs() < 1e-6);
    }

    // --- equitySeries ----------------------------------------------------------

    #[test]
    fn samples_realised_equity_daily_and_starts_at_the_initial_capital() {
        let trips = pair_trades(&fixture_trades()).trips;
        let window = window();
        let series = equity_series(&input(
            Some(&fixture_metrics()),
            &trips,
            Some(&window),
            Some(100_000.0),
        ));
        assert_eq!(series.sampling, EquitySampling::Daily);
        assert_eq!(series.initial, 100_000.0);
        // 2024-01-02 → 2024-04-02 is 91 days; the grid adds the leading capital point.
        assert_eq!(series.equity[0], 100_000.0);
        assert_eq!(series.steps, 92);
        assert!((series.final_value - 100_094.0).abs() < 1e-6);
        assert_eq!(
            series.equity.iter().cloned().fold(f64::INFINITY, f64::min),
            100_000.0
        );
        // Peaks after the first winner, before the second trip's loss is realised.
        assert_eq!(
            series.equity.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            100_098.0
        );
    }

    #[test]
    fn equity_is_non_decreasing_while_every_trade_wins_and_flat_with_no_trades() {
        let winning = pair_trades(&[
            fill(OrderSide::Buy, 10.0, 100.0, JAN_2, 0.0),
            fill(OrderSide::Sell, 10.0, 110.0, JAN_2 + 5 * DAYS, 0.0),
        ])
        .trips;
        let window = window();
        let metrics = fixture_metrics();
        let series = equity_series(&input(Some(&metrics), &winning, Some(&window), Some(100_000.0)));
        for pair in series.equity.windows(2) {
            assert!(pair[1] >= pair[0], "equity must not decrease");
        }
        let flat = equity_series(&input(Some(&metrics), &[], Some(&window), Some(100_000.0)));
        assert!(flat.equity.iter().all(|value| *value == 100_000.0));
    }

    #[test]
    fn equity_falls_back_to_a_trade_sampled_curve_when_no_window_is_known() {
        let trips = pair_trades(&fixture_trades()).trips;
        let metrics = fixture_metrics();
        let series = equity_series(&input(Some(&metrics), &trips, None, Some(100_000.0)));
        assert_eq!(series.sampling, EquitySampling::Trade);
        assert_eq!(series.steps, 1 + 3); // capital + three closed trips
        assert!((series.final_value - 100_094.0).abs() < 1e-6);
    }

    #[test]
    fn equity_uses_the_supplied_capital_not_a_hard_coded_default() {
        let window = window();
        let metrics = fixture_metrics();
        let series = equity_series(&input(
            Some(&metrics),
            &[],
            Some(&window),
            Some(1_000_000.0),
        ));
        assert_eq!(series.initial, 1_000_000.0);
    }

    // --- benchmarkEquity ---------------------------------------------------------

    #[test]
    fn benchmark_normalises_a_bar_series_to_the_same_capital_and_length() {
        let trips = pair_trades(&fixture_trades()).trips;
        let window = window();
        let metrics = fixture_metrics();
        let run = input(Some(&metrics), &trips, Some(&window), Some(100_000.0));
        let equity = equity_series(&run);
        let bars: Vec<ClosePoint> = (0..91)
            .map(|i| ClosePoint {
                time: JAN_2 + i * DAYS + 10 * HOURS,
                close: (100 + i) as f64,
            })
            .collect();
        let benchmark = benchmark_equity(&bars, &equity, &run).expect("daily sampling");
        assert_eq!(benchmark.len(), equity.equity.len());
        assert_eq!(benchmark[0], 100_000.0);
        let last = *benchmark.last().unwrap();
        assert!((last - 100_000.0 * (190.0 / 100.0)).abs() < 1e-6);
    }

    #[test]
    fn benchmark_is_null_when_there_are_no_usable_bars_or_no_window() {
        let trips = pair_trades(&fixture_trades()).trips;
        let window = window();
        let metrics = fixture_metrics();
        let run = input(Some(&metrics), &trips, Some(&window), Some(100_000.0));
        let equity = equity_series(&run);
        assert_eq!(benchmark_equity(&[], &equity, &run), None);

        let no_window_input = input(Some(&metrics), &trips, None, Some(100_000.0));
        assert_eq!(
            benchmark_equity(
                &[ClosePoint {
                    time: JAN_2,
                    close: 100.0,
                }],
                &equity,
                &no_window_input
            ),
            None
        );
    }

    // --- monthlyHeatmap ----------------------------------------------------------

    #[test]
    fn heatmap_buckets_realised_pnl_by_ist_month_and_reconciles_to_the_total() {
        let trips = pair_trades(&fixture_trades()).trips;
        let window = window();
        let map = monthly_heatmap(&input(
            Some(&fixture_metrics()),
            &trips,
            Some(&window),
            Some(100_000.0),
        ));
        assert_eq!(map.years, vec![2024]);
        assert!((map.total_pnl - 94.0).abs() < 1e-6);
        let sum: f64 = map.cells.iter().map(|cell| cell.pnl).sum();
        assert!((sum - map.total_pnl).abs() < 1e-6);
        assert!((map.total_return_pct.unwrap() - 94.0 / 100_000.0).abs() < 1e-9);
        // 2024-01 (exit on Jan 12), 2024-02 (exit on Feb 11), 2024-03 (exit on Mar 13).
        assert_eq!(
            map.cells.iter().map(|cell| cell.month).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        assert_eq!(map.cells[0].trades, 1);
    }

    #[test]
    fn heatmap_returns_an_empty_matrix_with_nothing_closed_and_still_sums_to_zero() {
        let window = window();
        let metrics = fixture_metrics();
        let map = monthly_heatmap(&input(Some(&metrics), &[], Some(&window), Some(100_000.0)));
        assert!(map.cells.is_empty());
        assert!(map.years.is_empty());
        assert_eq!(map.total_pnl, 0.0);
        assert_eq!(map.total_return_pct, Some(0.0));
    }

    #[test]
    fn ist_year_month_rolls_into_the_next_month_for_ist_early_morning() {
        // 00:00 UTC on Jan 1 = 05:30 IST on Jan 1.
        let utc_midnight = 1_704_067_200;
        assert_eq!(ist_year_month(utc_midnight), (2024, 0));
        // 20:30 UTC on Jan 31 = 02:00 IST on Feb 1 — belongs to February.
        let late_night = 1_704_067_200 + (30 * 86_400) + (20 * 3_600 + 30 * 60);
        assert_eq!(ist_year_month(late_night), (2024, 1));
    }

    // --- journalRows ---------------------------------------------------------------

    #[test]
    fn journal_rows_flatten_every_round_trip_with_its_derived_values() {
        let trips = pair_trades(&fixture_trades()).trips;
        let rows = journal_rows(&trips);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].id, "trip-1");
        assert_eq!(rows[0].instrument, "TCS.NSE");
        assert_eq!(rows[0].direction, Direction::Long);
        assert_eq!(rows[0].status, RoundTripStatus::Closed);
        assert_eq!(rows[0].avg_entry, 100.0);
        assert_eq!(rows[0].avg_exit, Some(110.0));
        assert_eq!(rows[0].fees, 2.0);
        assert_eq!(rows[0].exit_reason, RoundTripExitReason::PositionClosed);
    }

    #[test]
    fn journal_rows_keep_an_open_trip_open_with_null_exit_fields() {
        let trips = pair_trades(&[fill(OrderSide::Buy, 5.0, 200.0, JAN_2, 0.0)]).trips;
        let rows = journal_rows(&trips);
        assert_eq!(rows[0].status, RoundTripStatus::Open);
        assert_eq!(rows[0].exit_ts, None);
        assert_eq!(rows[0].avg_exit, None);
        assert_eq!(rows[0].realized_pnl, None);
        assert_eq!(rows[0].exit_reason, RoundTripExitReason::OpenAtEnd);
    }

    // --- CSV export ------------------------------------------------------------------

    struct CsvRow {
        symbol: String,
        pnl: f64,
        note: Option<String>,
    }

    fn row_value(row: &CsvRow) -> String {
        row.symbol.clone()
    }
    fn row_pnl(row: &CsvRow) -> String {
        format!("{}", row.pnl)
    }
    fn row_note(row: &CsvRow) -> String {
        row.note.clone().unwrap_or_default()
    }

    fn csv_columns() -> Vec<CsvColumn<CsvRow>> {
        vec![
            CsvColumn { header: "Symbol", value: row_value as fn(&CsvRow) -> String },
            CsvColumn { header: "PnL", value: row_pnl as fn(&CsvRow) -> String },
            CsvColumn { header: "Note", value: row_note as fn(&CsvRow) -> String },
        ]
    }

    #[test]
    fn csv_emits_a_header_and_one_crlf_line_per_row() {
        let columns = csv_columns();
        let csv = to_csv(
            &[
                CsvRow { symbol: "TCS".into(), pnl: 10.0, note: None },
                CsvRow { symbol: "INFY".into(), pnl: -4.0, note: None },
            ],
            &columns,
        );
        assert_eq!(csv, "Symbol,PnL,Note\r\nTCS,10,\r\nINFY,-4,");
    }

    #[test]
    fn csv_quotes_fields_holding_separators_and_doubles_embedded_quotes() {
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        assert_eq!(csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(csv_field("line\nbreak"), "\"line\nbreak\"");
        assert_eq!(csv_field(""), "");
        assert_eq!(csv_field("1.5"), "1.5");
    }

    #[test]
    fn csv_serialises_the_journal_rows_so_the_export_matches_the_on_screen_table() {
        let trips = pair_trades(&fixture_trades()).trips;
        let rows = journal_rows(&trips);
        // The CSV is built from the same `journal_rows` the table renders
        // (design wart W5) — and the slippage column stays empty by rule 4:
        // the journal carries no signal price, so none is invented.
        let columns = vec![
            CsvColumn { header: "id", value: |r: &JournalRow| r.id.clone() },
            CsvColumn { header: "entry_ts", value: |r: &JournalRow| format_timestamp(Some(r.entry_ts)) },
            CsvColumn { header: "exit_ts", value: |r: &JournalRow| format_timestamp(r.exit_ts) },
            CsvColumn { header: "qty", value: |r: &JournalRow| format!("{}", r.quantity) },
            CsvColumn { header: "avg_entry", value: |r: &JournalRow| format!("{}", r.avg_entry) },
            CsvColumn { header: "avg_exit", value: |r: &JournalRow| r.avg_exit.map(|v| format!("{v}")).unwrap_or_default() },
            CsvColumn { header: "net_pnl", value: |r: &JournalRow| r.realized_pnl.map(|v| format!("{v}")).unwrap_or_default() },
            CsvColumn { header: "return_pct", value: |r: &JournalRow| r.return_pct.map(|v| format!("{:.4}", v * 100.0)).unwrap_or_default() },
            CsvColumn { header: "fees", value: |r: &JournalRow| format!("{}", r.fees) },
            CsvColumn { header: "slippage", value: |_r: &JournalRow| String::new() },
        ];
        let csv = to_csv(&rows, &columns);
        let lines: Vec<&str> = csv.split("\r\n").collect();
        assert_eq!(
            lines[0],
            "id,entry_ts,exit_ts,qty,avg_entry,avg_exit,net_pnl,return_pct,fees,slippage"
        );
        assert_eq!(lines.len(), 1 + rows.len());
        let first: Vec<&str> = lines[1].split(',').collect();
        assert_eq!(first[0], "trip-1");
        assert_eq!(first[1], "2024-01-02 00:00");
        assert_eq!(first[3], "10");
        assert_eq!(first[4], "100");
        assert_eq!(first[5], "110");
        assert!((first[6].parse::<f64>().unwrap() - 98.0).abs() < 1e-6);
        assert_eq!(first[9], ""); // slippage: the journal carries no signal price
    }

    // --- formatTimestamp ------------------------------------------------------------

    #[test]
    fn format_timestamp_renders_utc_seconds_and_an_em_dash_when_absent() {
        assert_eq!(format_timestamp(Some(JAN_2)), "2024-01-02 00:00");
        assert_eq!(format_timestamp(Some(JAN_2 + 9 * HOURS + 11 * 60)), "2024-01-02 09:11");
        assert_eq!(format_timestamp(None), "—");
    }

    // --- deriveRibbonMetrics — provenance ---------------------------------------------

    fn ribbon_by_key(input: &AnalyticsInput<'_>) -> Vec<(&'static str, RibbonMetric)> {
        derive_ribbon_metrics(input)
            .into_iter()
            .map(|metric| (metric.key, metric))
            .collect()
    }

    #[test]
    fn ribbon_echoes_every_wire_metric_exactly_as_the_payload_reported() {
        let trips = pair_trades(&fixture_trades()).trips;
        let window = window();
        let metrics = fixture_metrics();
        let ribbon = ribbon_by_key(&input(Some(&metrics), &trips, Some(&window), Some(100_000.0)));
        let by_key = |key: &str| ribbon.iter().find(|(k, _)| *k == key).unwrap().1.clone();

        assert_eq!(by_key("total_return").value, Some(metrics.total_return));
        assert_eq!(by_key("total_return").provenance, MetricProvenance::Wire);
        assert_eq!(by_key("sharpe").value, Some(metrics.sharpe));
        assert_eq!(by_key("sharpe").provenance, MetricProvenance::Wire);
        assert_eq!(by_key("max_drawdown").value, Some(metrics.max_drawdown));
        assert_eq!(by_key("max_drawdown").provenance, MetricProvenance::Wire);
        assert_eq!(by_key("trades").value, Some(metrics.trades as f64));
        assert_eq!(by_key("trades").provenance, MetricProvenance::Wire);
    }

    #[test]
    fn ribbon_derives_cagr_calmar_win_rate_profit_factor_and_exposure() {
        let trips = pair_trades(&fixture_trades()).trips;
        let window = window();
        let metrics = fixture_metrics();
        let ribbon = ribbon_by_key(&input(Some(&metrics), &trips, Some(&window), Some(100_000.0)));
        let by_key = |key: &str| ribbon.iter().find(|(k, _)| *k == key).unwrap().1.clone();

        let years = years_between(Some("2024-01-02"), Some("2024-04-02")).unwrap();
        let expected_cagr = (1.0 + metrics.total_return).powf(1.0 / years) - 1.0;
        let cagr = by_key("cagr");
        assert!((cagr.value.unwrap() - expected_cagr).abs() < 1e-9);
        assert_eq!(cagr.provenance, MetricProvenance::Derived);
        assert!(cagr.note.contains("total return"));

        let calmar = by_key("calmar");
        assert!((calmar.value.unwrap() - expected_cagr / 0.2).abs() < 1e-9);
        assert_eq!(calmar.provenance, MetricProvenance::Derived);

        let win_rate = by_key("win_rate");
        assert!((win_rate.value.unwrap() - 2.0 / 3.0).abs() < 1e-6);
        assert_eq!(win_rate.provenance, MetricProvenance::Derived);

        let profit_factor = by_key("profit_factor");
        assert!((profit_factor.value.unwrap() - 146.0 / 52.0).abs() < 1e-6);

        let exposure = by_key("exposure");
        assert!(exposure.value.unwrap() > 0.0);
        assert_eq!(exposure.provenance, MetricProvenance::Derived);
    }

    #[test]
    fn ribbon_reports_sortino_unavailable_never_a_fabricated_number() {
        let trips = pair_trades(&fixture_trades()).trips;
        let window = window();
        let metrics = fixture_metrics();
        let ribbon = ribbon_by_key(&input(Some(&metrics), &trips, Some(&window), Some(100_000.0)));
        let sortino = ribbon.iter().find(|(k, _)| *k == "sortino").unwrap().1.clone();
        assert_eq!(sortino.value, None);
        assert_eq!(sortino.provenance, MetricProvenance::Unavailable);
        assert!(sortino.note.contains("return series"));
    }

    #[test]
    fn ribbon_marks_cagr_calmar_and_exposure_unavailable_when_the_window_is_unknown() {
        let trips = pair_trades(&fixture_trades()).trips;
        let metrics = fixture_metrics();
        let ribbon =
            ribbon_by_key(&input(Some(&metrics), &trips, None, Some(100_000.0)));
        let by_key = |key: &str| ribbon.iter().find(|(k, _)| *k == key).unwrap().1.clone();

        assert_eq!(by_key("cagr").provenance, MetricProvenance::Unavailable);
        assert_eq!(by_key("cagr").value, None);
        assert_eq!(by_key("calmar").value, None);
        assert_eq!(by_key("exposure").provenance, MetricProvenance::Unavailable);
        assert_eq!(by_key("exposure").value, None);
        // Wire values survive regardless of the window.
        assert_eq!(by_key("sharpe").value, Some(metrics.sharpe));
    }

    #[test]
    fn ribbon_tolerates_an_absent_payload_without_crashing() {
        let trips = pair_trades(&fixture_trades()).trips;
        let window = window();
        let ribbon = ribbon_by_key(&input(None, &trips, Some(&window), Some(100_000.0)));
        let by_key = |key: &str| ribbon.iter().find(|(k, _)| *k == key).unwrap().1.clone();
        assert_eq!(by_key("total_return").value, None);
        assert_eq!(by_key("total_return").provenance, MetricProvenance::Wire);
        assert_eq!(by_key("calmar").value, None);
        // Exactly the ten documented ribbon slots, in order and without `_`.
        let ribbon_order = derive_ribbon_metrics(&input(None, &[], None, None));
        assert_eq!(
            ribbon_order
                .iter()
                .map(|metric| metric.key)
                .collect::<Vec<_>>(),
            vec![
                "total_return", "cagr", "sharpe", "sortino", "calmar", "max_drawdown",
                "win_rate", "profit_factor", "trades", "exposure"
            ]
        );
    }

    // --- formatMetricValue ---------------------------------------------------------------

    #[test]
    fn format_metric_value_formats_each_kind_including_drawdown_as_negative() {
        assert_eq!(format_metric_value(Some(0.1234), MetricKind::Percent), "+12.34%");
        assert_eq!(format_metric_value(Some(-0.05), MetricKind::Percent), "-5.00%");
        assert_eq!(format_metric_value(Some(0.2), MetricKind::Drawdown), "-20.00%");
        assert_eq!(format_metric_value(Some(1.5), MetricKind::Ratio), "1.50");
        assert_eq!(format_metric_value(Some(42.0), MetricKind::Count), "42");
        assert!(format_metric_value(Some(1234.5), MetricKind::Currency).contains('1'));
        assert_eq!(format_metric_value(Some(1234.5), MetricKind::Currency), "1,234.50");
        assert_eq!(format_metric_value(None, MetricKind::Percent), "—");
        assert_eq!(
            format_metric_value(Some(f64::NAN), MetricKind::Ratio),
            "—"
        );
    }

    // --- benchmark_instrument_id ----------------------------------------------------------

    #[test]
    fn benchmark_ids_are_normalised_to_the_repo_s_symbol_convention() {
        assert_eq!(
            benchmark_instrument_id(Some("NIFTY 50"), Some("NSE")),
            Some("NIFTY50.NSE".to_string())
        );
        assert_eq!(
            benchmark_instrument_id(Some("nifty 50"), None),
            Some("NIFTY50.NSE".to_string())
        );
        assert_eq!(benchmark_instrument_id(Some("  "), None), None);
        assert_eq!(benchmark_instrument_id(None, None), None);
    }

    // --- reconciliation — fixture payload --------------------------------------------------

    #[test]
    fn every_wire_metric_matches_the_payload_and_derived_totals_reconcile() {
        let trips = pair_trades(&fixture_trades()).trips;
        let window = window();
        let metrics = fixture_metrics();
        let analytics = input(Some(&metrics), &trips, Some(&window), Some(100_000.0));
        let ribbon = derive_ribbon_metrics(&analytics);
        let fixture_values: [(&str, f64); 4] = [
            ("total_return", metrics.total_return),
            ("sharpe", metrics.sharpe),
            ("max_drawdown", metrics.max_drawdown),
            ("trades", metrics.trades as f64),
        ];
        for (key, expected) in fixture_values {
            let metric = ribbon.iter().find(|m| m.key == key).unwrap();
            assert_eq!(metric.provenance, MetricProvenance::Wire);
            assert_eq!(metric.value, Some(expected));
        }
        // The heatmap reconciles with realised PnL, and the equity curve's last
        // point lands on capital + that same realised PnL.
        let heatmap = monthly_heatmap(&analytics);
        let stats = round_trip_stats(&analytics.trips);
        assert!((heatmap.total_pnl - stats.realized_pnl).abs() < 1e-6);
        assert!((equity_series(&analytics).final_value - (100_000.0 + heatmap.total_pnl)).abs() < 1e-6);
    }
}
