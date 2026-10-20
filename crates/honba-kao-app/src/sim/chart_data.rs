//! Chart data plumbing (HBD-2026-DES-005 P2).
//!
//! The Leptos port of honba-fe `core/simulator/chart-data.ts` (plus the
//! deterministic preview generator that lived there): converts the wire
//! [`WireBar`](honba_kao_client::wire::Bar) (`ts_event: UnixNanos`) into the
//! shape `openalgo-charts` draws, folds bars to the run's `bar_spec`, and
//! derives trade-marker timestamps.
//!
//! Two rules are load-bearing for the acceptance criteria:
//!
//! 1. **Never synthesise a candle.** Resampling folds *bars that exist* into
//!    buckets; a bucket with no source bar is not emitted, so an NSE holiday or
//!    an overnight gap stays a gap (no phantom candles).
//! 2. **Markers land on bar timestamps.** Journal fills carry their own
//!    `ts_event`; [`snap_ts`] maps each one onto the closest bar inside one
//!    bar step so an entry/exit triangle sits on the candle it belongs to.
//!
//! Day/week/month bucketing and the preview session are **NSE / Asia-Kolkata**
//! shaped, matching the chart timezone; the run contract's `bar_spec`
//! (`<n><s|m|h|d|w|mo>`) is parsed exactly as `honba-api` does (via
//! [`crate::sim::controls::parse_bar_spec`]).
//!
//! Deviation from the JS source (documented per the port's DoD): there is no
//! `unixNanosToSeconds` string dance — on the Rust side `WireBar` decodes
//! directly into the typed [`WireUnixNanos`](honba_kao_client::wire::WireUnixNanos),
//! so whole seconds come from an exact integer division, never a `Number`
//! conversion. The finite-price checks stay because they are cheap and mirror
//! fe, even though a Rust-decoded wire bar cannot hold a NaN.

use honba_kao_client::wire::{Bar as WireBar, WireUnixNanos};

use crate::sim::controls::{parse_bar_spec, BarUnit};

/// IST is the session the Simulator charts; UTC+5:30 has no DST.
pub const IST_OFFSET_SECONDS: i64 = 5 * 3600 + 1800;

/// NSE regular session, in seconds past IST midnight (09:15 → 15:30).
pub const SESSION_OPEN_SECONDS: i64 = 9 * 3600 + 15 * 60;
pub const SESSION_CLOSE_SECONDS: i64 = 15 * 3600 + 30 * 60;

/// The same window measured past **UTC** midnight (03:45 → 10:00), for
/// UTC-anchored day maths.
pub const SESSION_OPEN_UTC: i64 = SESSION_OPEN_SECONDS - IST_OFFSET_SECONDS;
pub const SESSION_CLOSE_UTC: i64 = SESSION_CLOSE_SECONDS - IST_OFFSET_SECONDS;

/// Safety valve for the preview generator (mirrors `MAX_BAR_ROWS` in spirit).
pub const MAX_PREVIEW_BARS: usize = 4_000;

/// A bar as `openalgo-charts` draws it: integer UTC seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartBar {
    /// Bar timestamp, whole UTC seconds.
    pub time: i64,
    /// Open.
    pub open: f64,
    /// High.
    pub high: f64,
    /// Low.
    pub low: f64,
    /// Close.
    pub close: f64,
    /// Volume.
    pub volume: f64,
}

/// Fixed width of one bucket in seconds; `None` for `mo`, which is
/// calendar-sized (`barSpecSeconds`).
pub fn bar_spec_seconds(bar_spec: &str) -> Option<i64> {
    let (step, unit) = parse_bar_spec(bar_spec)?;
    let unit_seconds = match unit {
        BarUnit::Second => 1,
        BarUnit::Minute => 60,
        BarUnit::Hour => 3600,
        BarUnit::Day => 86_400,
        BarUnit::Week => 604_800,
        BarUnit::Month => return None,
    };
    Some(unit_seconds * step as i64)
}

/// `ts_event` → whole UTC seconds (exact integer division; see the module doc
/// for the JS deviation). `None` only for the epoch placeholder.
pub fn unix_nanos_to_seconds(ts: WireUnixNanos) -> Option<i64> {
    let nanos = ts.as_u64();
    if nanos == 0 {
        return None;
    }
    Some((nanos / 1_000_000_000) as i64)
}

/// Normalise wire bars for the chart: seconds-based `time`, ascending order,
/// one bar per timestamp (the later row wins) and invalid rows dropped.
///
/// Bars a source never emitted are simply absent, which is what keeps holidays
/// and weekends visible as gaps.
pub fn to_chart_bars(bars: &[WireBar]) -> Vec<ChartBar> {
    let mut by_time: std::collections::BTreeMap<i64, ChartBar> = std::collections::BTreeMap::new();
    for bar in bars {
        let Some(time) = unix_nanos_to_seconds(bar.ts_event()) else {
            continue;
        };
        let (open, high, low, close, volume) =
            (bar.open(), bar.high(), bar.low(), bar.close(), bar.volume());
        let values_ok = [open, high, low, close].iter().all(|value| value.is_finite());
        if !values_ok {
            continue;
        }
        by_time.insert(
            time,
            ChartBar {
                time,
                open,
                high,
                low,
                close,
                volume: if volume.is_finite() { volume } else { 0.0 },
            },
        );
    }
    by_time.into_values().collect()
}

/// Start of the IST day containing `time`, in UTC seconds.
fn ist_day_start(time: i64) -> i64 {
    let day = (time + IST_OFFSET_SECONDS).div_euclid(86_400);
    day * 86_400 - IST_OFFSET_SECONDS
}

/// `0 = Sunday … 6 = Saturday` for the IST day containing `time`.
fn ist_weekday(time: i64) -> i64 {
    let day = (time + IST_OFFSET_SECONDS).div_euclid(86_400);
    (day + 4).rem_euclid(7)
}

/// Civil date (year, month 1-12, day 1-31) of the day index
/// `days since 1970-01-01` (Hinnant's `civil_from_days`).
pub(crate) fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

/// Day index `days since 1970-01-01` of a civil date (Hinnant's
/// `days_from_civil`). Month must be 1-12; the day may overflow (e.g. 32),
/// the caller normalises first.
pub(crate) fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400; // [0, 399]
    let mp = if m > 2 { m - 3 } else { m + 9 } as i64; // [0, 11]
    let doy = (153 * mp + 2) / 5 + d as i64 - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

/// Canonical bucket start for `time`, aware of IST day/week/month boundaries
/// (`bucketStart`). S/M/H buckets align on the epoch grid; D/W/M align on IST
/// civil boundaries.
pub fn bucket_start(time: i64, spec: (u32, BarUnit)) -> i64 {
    let (step, unit) = spec;
    match unit {
        BarUnit::Second | BarUnit::Minute | BarUnit::Hour => {
            let width = step as i64 * match unit {
                BarUnit::Second => 1,
                BarUnit::Minute => 60,
                BarUnit::Hour => 3600,
                _ => unreachable!("guarded by the match"),
            };
            time.div_euclid(width) * width
        }
        BarUnit::Day => ist_day_start(time),
        BarUnit::Week => {
            let days_since_monday = (ist_weekday(time) + 6) % 7;
            ist_day_start(time) - days_since_monday * 86_400
        }
        BarUnit::Month => {
            let (year, month, _) =
                civil_from_days((time + IST_OFFSET_SECONDS).div_euclid(86_400));
            days_from_civil(year, month, 1) * 86_400 - IST_OFFSET_SECONDS
        }
    }
}

/// Fold bars to a coarser `bar_spec`.
///
/// Only buckets that contain at least one source bar are emitted — the
/// returned bar keeps the **first source bar's timestamp**, so a daily bar
/// keeps its session anchor rather than a UTC-midnight label (rationale copied
/// from fe: the anchor is what lets markers and pivots agree across
/// timeframes). Empty buckets are skipped entirely, so market holidays,
/// weekends and the overnight gap stay gaps (`resampleBars`).
pub fn resample_bars(bars: &[ChartBar], bar_spec: &str) -> Vec<ChartBar> {
    let Some(spec) = parse_bar_spec(bar_spec) else {
        return bars.to_vec();
    };
    if bars.is_empty() {
        return Vec::new();
    }

    let mut buckets: std::collections::BTreeMap<i64, ChartBar> = std::collections::BTreeMap::new();
    let mut sorted: Vec<ChartBar> = bars.to_vec();
    sorted.sort_by_key(|bar| bar.time);

    for bar in sorted {
        let key = bucket_start(bar.time, spec);
        match buckets.get_mut(&key) {
            None => {
                buckets.insert(key, bar);
            }
            Some(current) => {
                current.high = current.high.max(bar.high);
                current.low = current.low.min(bar.low);
                current.close = bar.close;
                current.volume += bar.volume;
            }
        }
    }

    buckets.into_values().collect()
}

/// Stable 32-bit hash → `[0, 1)` (FNV-1a style, as fe's `hash01`), so a
/// preview bar is a pure function of its time.
fn hash01(parts: &[i64]) -> f64 {
    let mut h: u32 = 2_166_136_261;
    for &part in parts {
        h ^= part as u32;
        h = h.wrapping_mul(16_777_619);
        h ^= h >> 13;
    }
    (h % 10_000) as f64 / 10_000.0
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// Preview close for one timestamp: a deterministic, window-independent price
/// path.
fn preview_close(time: i64, anchor: f64, step: i64, seed: i64) -> f64 {
    let t = time as f64;
    let cycle30 = (t / 2_592_000.0).sin();
    let cycle7 = (t / 604_800.0 + 1.7).sin();
    let cycle1 = (t / 86_400.0 + 0.4).sin();
    let noise = hash01(&[seed, time.div_euclid(step.max(1)), 1]) - 0.5;
    anchor * (1.0 + 0.1 * cycle30 + 0.05 * cycle7 + 0.02 * cycle1 + 0.012 * noise)
}

fn preview_bar(time: i64, step: i64, anchor: f64, seed: i64) -> ChartBar {
    let close = round2(preview_close(time, anchor, step, seed));
    let open = round2(preview_close(time - step, anchor, step, seed));
    let span = (close - open).abs() + anchor * 0.004;
    let hi = hash01(&[seed, time, 2]);
    let lo = hash01(&[seed, time, 3]);
    let high = round2(open.max(close) + span * hi * 0.6);
    let low = round2(open.min(close) - span * lo * 0.6);
    let volume = (40_000.0 + 90_000.0 * hash01(&[seed, time, 4])).round();
    ChartBar {
        time,
        open,
        high,
        low,
        close,
        volume,
    }
}

/// Stable per-instrument/spec seed (`seedFor`), `10..=0` over 31-mul folding.
fn seed_for(instrument_id: &str, spec: (u32, BarUnit)) -> i64 {
    let key = format!("{}|{}{}", instrument_id, spec.0, bar_spec_unit_str(spec.1));
    let mut h: i32 = 0;
    for byte in key.chars() {
        h = (h as i32).wrapping_mul(31).wrapping_add(byte as i32);
    }
    h as i64
}

/// The wire spelling of an aggregation unit (`s|m|h|d|w|mo`).
pub fn bar_spec_unit_str(unit: BarUnit) -> &'static str {
    match unit {
        BarUnit::Second => "s",
        BarUnit::Minute => "m",
        BarUnit::Hour => "h",
        BarUnit::Day => "d",
        BarUnit::Week => "w",
        BarUnit::Month => "mo",
    }
}

/// A [`preview_bars`] request (`PreviewBarsRequest`).
pub struct PreviewBarsRequest<'a> {
    /// `SYMBOL.EXCHANGE` the preview purports to chart.
    pub instrument_id: &'a str,
    /// Inclusive window start, UTC seconds.
    pub from: i64,
    /// Exclusive window end, UTC seconds.
    pub to: i64,
    /// Run `bar_spec`, e.g. `5m`.
    pub bar_spec: &'a str,
    /// Price the path oscillates around (the instrument's last known price).
    pub anchor: Option<f64>,
}

/// Deterministic NSE-session bars for when the server has none.
///
/// This is the Simulator's only local bar source, and it is always surfaced as
/// `source: 'preview'` in the UI. It honours the market calendar as far as the
/// client can: **weekends are skipped and only the 09:15–15:30 IST session is
/// emitted** — public holidays are not known client-side, so no bar is
/// invented for a gap the source does have, and none is invented for a
/// weekday either beyond the session.
///
/// Every bar is a pure function of its own timestamp, so two overlapping
/// windows agree bar-for-bar instead of drifting with the window edges.
pub fn preview_bars(request: &PreviewBarsRequest<'_>) -> Vec<ChartBar> {
    let Some(spec) = parse_bar_spec(request.bar_spec) else {
        return Vec::new();
    };
    let (from, to) = (request.from, request.to);
    if to <= from {
        return Vec::new();
    }
    let anchor = match request.anchor {
        Some(value) if value.is_finite() && value > 0.0 => value,
        _ => 100.0,
    };
    let seed = seed_for(request.instrument_id, spec);
    let intraday = matches!(
        spec.1,
        BarUnit::Second | BarUnit::Minute | BarUnit::Hour
    );
    let step = if intraday {
        (spec.0 as i64)
            * match spec.1 {
                BarUnit::Second => 1,
                BarUnit::Minute => 60,
                BarUnit::Hour => 3600,
                _ => unreachable!("intraday only"),
            }
    } else {
        86_400
    };

    let mut out: Vec<ChartBar> = Vec::new();
    let first_day = (from + IST_OFFSET_SECONDS).div_euclid(86_400);
    let last_day = (to - 1 + IST_OFFSET_SECONDS).div_euclid(86_400);

    let mut day = first_day;
    while day <= last_day && out.len() < MAX_PREVIEW_BARS {
        let weekday = (day + 4).rem_euclid(7);
        if weekday == 0 || weekday == 6 {
            day += 1;
            continue; // weekend
        }

        let day_start = day * 86_400 - IST_OFFSET_SECONDS;
        let session_open = day_start + SESSION_OPEN_SECONDS;
        let session_close = day_start + SESSION_CLOSE_SECONDS;

        if intraday {
            let start = from.max(session_open);
            let end = to.min(session_close);
            if end > start {
                let offset = (start - session_open).div_euclid(step.max(1));
                let offset = if (start - session_open).rem_euclid(step.max(1)) > 0 {
                    offset + 1
                } else {
                    offset
                };
                let mut k = offset;
                loop {
                    let time = session_open + k * step;
                    if time >= end || out.len() >= MAX_PREVIEW_BARS {
                        break;
                    }
                    if time >= from && time < to {
                        out.push(preview_bar(time, step, anchor, seed));
                    }
                    k += 1;
                }
            }
            day += 1;
            continue;
        }

        // Daily / weekly / monthly: one bar per bucket, anchored at the open.
        let time = session_open;
        if time >= from && time < to {
            let is_monday = weekday == 1;
            let monthly_anchor = if spec.1 == BarUnit::Month {
                let (year, month, d) = civil_from_days(day);
                Some(first_monday_of(year as i32, month) == d)
            } else {
                None
            };
            let emits = match (spec.1 == BarUnit::Week, spec.1 == BarUnit::Month) {
                (true, false) => is_monday,
                (false, true) => monthly_anchor.unwrap_or(false),
                _ => true,
            };
            if emits {
                out.push(preview_bar(time, step, anchor, seed));
            }
        }
        day += 1;
    }

    // The loop already stops at the cap, so the tail slice is a no-op — kept
    // for parity with fe's `out.slice(-MAX_PREVIEW_BARS)` belt-and-braces.
    out
}

/// Day-of-month number (1-31) of the first Monday of a month.
fn first_monday_of(year: i32, month: u32) -> u32 {
    let first = days_from_civil(year as i64, month, 1);
    let weekday = (first + 4).rem_euclid(7); // 0 = Sunday
    let offset = (1 - weekday).rem_euclid(7); // days to Monday
    (1 + offset) as u32
}

/// Move each fill timestamp onto the closest bar timestamp inside one bar step
/// (`snapToBars`, value-level).
///
/// A fill's `ts_event` is the bar it executed on, but a rounded or
/// close-of-bar timestamp can sit off the candle by a fraction of a step.
/// Anything further than one step from a bar is left alone rather than dragged
/// onto a candle it did not trade on. Callers map marker structs through this
/// with their own `ts` accessor.
pub fn snap_ts(ts_values: &[i64], bars: &[ChartBar], tolerance: i64) -> Vec<i64> {
    if ts_values.is_empty() || bars.is_empty() || tolerance <= 0 {
        return ts_values.to_vec();
    }
    let times: Vec<i64> = bars.iter().map(|bar| bar.time).collect();
    ts_values
        .iter()
        .map(|ts| {
            // Last bar at or before `ts`.
            let lo = match times.partition_point(|t| *t <= *ts) {
                0 => 0,
                index => index - 1,
            };
            let candidates = [times[lo], times[(lo + 1).min(times.len() - 1)]];
            let mut best = *ts;
            let mut best_delta = i64::MAX;
            for candidate in candidates {
                let delta = candidate.abs_diff(*ts);
                if delta < best_delta as u64 || (delta as i64 == best_delta && candidate < best) {
                    best = candidate;
                    best_delta = delta as i64;
                }
            }
            // delta <= tolerance, guarded against overflow.
            if best_delta <= tolerance && best_delta != i64::MAX {
                best
            } else {
                *ts
            }
        })
        .collect()
}

/// Tolerance for [`snap_ts`]: one whole bar step of the run's `bar_spec`
/// (`markerToleranceSec`), `0` (never snap) for unusable/monthly specs.
pub fn marker_tolerance_sec(bar_spec: &str) -> i64 {
    bar_spec_seconds(bar_spec).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use honba_kao_client::wire::{
        Exchange, BarAggregation, BarSpecification, BarType, InstrumentId, PriceType,
    };
    /// 2024-01-01 is a Monday.
    const JAN1_2024: i64 = 1_704_067_200;

    /// Midnight UTC of the calendar day `i` days after 2024-01-01.
    fn day(i: i64) -> i64 {
        JAN1_2024 + i * 86_400
    }

    /// 09:15 IST of the calendar day `i` after 2024-01-01.
    fn open(i: i64) -> i64 {
        day(i) + SESSION_OPEN_UTC
    }

    fn wire_bar(seconds: u64, close: f64) -> WireBar {
        let instrument = InstrumentId::new("TCS", Exchange::new("NSE"));
        let spec = BarSpecification::new(1, BarAggregation::Day, PriceType::Last);
        WireBar::new(
            BarType::new(instrument, spec),
            close - 1.0,
            close + 1.0,
            close - 2.0,
            close,
            1_000.0,
            WireUnixNanos::from_u64(seconds * 1_000_000_000),
            WireUnixNanos::from_u64(seconds * 1_000_000_000),
        )
    }

    fn chart_bar(time: i64, close: f64, volume: f64) -> ChartBar {
        ChartBar {
            time,
            open: close,
            high: close + 1.0,
            low: close - 1.0,
            close,
            volume,
        }
    }

    /// fe `minute(day, offset, close)`: one `offset`-minutes-past-open bar.
    fn minute(day_index: i64, offset: i64, close: f64, volume: f64) -> ChartBar {
        chart_bar(open(day_index) + offset * 60, close, volume)
    }

    #[test]
    fn reads_whole_seconds_off_the_wire_timestamps() {
        // 1.7e18 ns is a u64 — the seconds are an exact integer division on
        // the Rust side (fe sliced the string to dodge Number precision).
        let ts = WireUnixNanos::from_u64(1_700_000_000_250_000_000);
        assert_eq!(unix_nanos_to_seconds(ts), Some(1_700_000_000));
        // The epoch placeholder reads as absent.
        assert_eq!(unix_nanos_to_seconds(WireUnixNanos::from_u64(0)), None);
    }

    #[test]
    fn parses_the_run_contract_spelling() {
        assert_eq!(bar_spec_seconds("5m"), Some(300));
        assert_eq!(bar_spec_seconds("1d"), Some(86_400));
        assert_eq!(bar_spec_seconds("1mo"), None);
        assert_eq!(marker_tolerance_sec("5m"), 300);
        assert_eq!(marker_tolerance_sec("1mo"), 0);
        assert_eq!(bar_spec_seconds("6"), None);
        assert_eq!(bar_spec_seconds(""), None);
    }

    #[test]
    fn converts_sorts_and_de_dublicates_keeping_the_later_bar() {
        let bars = to_chart_bars(&[
            wire_bar(200, 2.0),
            wire_bar(100, 1.0),
            wire_bar(200, 3.0),
        ]);
        let pairs: Vec<(i64, f64)> = bars.iter().map(|b| (b.time, b.close)).collect();
        assert_eq!(pairs, [(100, 1.0), (200, 3.0)]);
    }

    #[test]
    fn folds_finer_bars_into_the_run_bar_spec() {
        let folded = resample_bars(
            &[minute(0, 0, 100.0, 10.0), minute(0, 5, 104.0, 10.0), minute(0, 10, 102.0, 10.0)],
            "15m",
        );
        assert_eq!(folded.len(), 1);
        assert_eq!(
            folded[0],
            ChartBar {
                time: open(0),
                open: 100.0,
                high: 105.0,
                low: 99.0,
                close: 102.0,
                volume: 30.0,
            }
        );
    }

    #[test]
    fn never_invents_a_bar_for_a_bucket_with_no_data() {
        // Mon 2024-01-01 and Mon 2024-01-08: the week in between is never
        // emitted by the source and is not fabricated here.
        let daily = resample_bars(&[minute(0, 0, 100.0, 10.0), minute(7, 0, 110.0, 10.0)], "1d");
        let times: Vec<i64> = daily.iter().map(|b| b.time).collect();
        assert_eq!(times, [open(0), open(7)]);
        assert!(resample_bars(&[], "1d").is_empty());
    }

    #[test]
    fn buckets_days_and_months_on_ist_boundaries() {
        // 16:00 IST is 10:30 UTC — the same IST day, and still January.
        let bars = [
            chart_bar(open(0), 1.0, 1.0),
            chart_bar(open(0) + 22_500, 2.0, 1.0),
        ];
        let daily = resample_bars(&bars, "1d");
        assert_eq!(daily.iter().map(|b| b.close).collect::<Vec<_>>(), [2.0]);
        assert_eq!(resample_bars(&bars, "1mo").len(), 1);
    }

    #[test]
    fn returns_the_input_unchanged_when_the_spec_is_the_bar_width() {
        let bars = vec![minute(0, 0, 1.0, 10.0), minute(0, 5, 2.0, 10.0)];
        let times: Vec<i64> = resample_bars(&bars, "5m").iter().map(|b| b.time).collect();
        let expected: Vec<i64> = bars.iter().map(|b| b.time).collect();
        assert_eq!(times, expected);
    }

    #[test]
    fn civil_helpers_round_trip() {
        // Sanity of the civil helpers the IST bucketing leans on.
        let (y, m, d) = civil_from_days((JAN1_2024).div_euclid(86_400));
        assert_eq!((y, m, d), (2024, 1u32, 1u32));
        assert_eq!(days_from_civil(2024, 1, 1), JAN1_2024.div_euclid(86_400));
    }

    #[test]
    fn preview_emits_one_bar_per_weekday_session() {
        let request = PreviewBarsRequest {
            instrument_id: "TCS.NSE",
            from: open(0) - 3_600,
            to: open(5),
            bar_spec: "1d",
            anchor: None,
        };
        let bars = preview_bars(&request);
        assert_eq!(bars.len(), 5); // Mon..Fri; Sat 2024-01-06 is outside the window
        for bar in &bars {
            let weekday = ((bar.time + IST_OFFSET_SECONDS).div_euclid(86_400) + 4).rem_euclid(7);
            assert!((1..=5).contains(&weekday), "weekday {weekday} emitted");
            assert_eq!(bar.time.rem_euclid(86_400), SESSION_OPEN_UTC);
        }
    }

    #[test]
    fn preview_is_a_pure_function_of_time() {
        let request = PreviewBarsRequest {
            instrument_id: "TCS.NSE",
            from: open(0) - 3_600,
            to: open(5),
            bar_spec: "1d",
            anchor: None,
        };
        let wide = preview_bars(&request);
        let narrow = preview_bars(&PreviewBarsRequest {
            from: open(2) - 3_600,
            ..request
        });
        assert!(!narrow.is_empty());
        let overlap: Vec<ChartBar> = wide
            .iter()
            .filter(|bar| bar.time >= narrow[0].time)
            .copied()
            .collect();
        assert_eq!(overlap, narrow);
    }

    #[test]
    fn preview_keeps_intraday_bars_inside_the_session_on_the_grid() {
        let request = PreviewBarsRequest {
            instrument_id: "TCS.NSE",
            from: open(0) - 3_600,
            to: day(1),
            bar_spec: "5m",
            anchor: None,
        };
        let bars = preview_bars(&request);
        assert!(bars.len() > 50);
        for bar in &bars {
            let ist_day = (bar.time + IST_OFFSET_SECONDS).div_euclid(86_400);
            let session_open = ist_day * 86_400 - IST_OFFSET_SECONDS + SESSION_OPEN_SECONDS;
            let session_close = ist_day * 86_400 - IST_OFFSET_SECONDS + SESSION_CLOSE_SECONDS;
            assert!(bar.time >= session_open);
            assert!(bar.time < session_close);
            assert_eq!((bar.time - session_open).rem_euclid(300), 0);
            assert!(bar.high >= bar.close);
            assert!(bar.close >= bar.low);
        }
    }

    #[test]
    fn preview_returns_nothing_for_an_unusable_spec_or_window() {
        let request = PreviewBarsRequest {
            instrument_id: "TCS.NSE",
            from: open(0) - 3_600,
            to: open(5),
            bar_spec: "soon",
            anchor: None,
        };
        assert!(preview_bars(&request).is_empty());
        assert!(preview_bars(&PreviewBarsRequest {
            to: request.from,
            ..request
        })
        .is_empty());
    }

    #[test]
    fn snap_lands_a_fill_on_the_bar_it_traded_on() {
        let bars: Vec<ChartBar> = [0, 300, 600, 900]
            .iter()
            .map(|offset| chart_bar(open(0) + offset, 1.0, 0.0))
            .collect();
        assert_eq!(
            snap_ts(&[open(0) + 600], &bars, 300),
            vec![open(0) + 600]
        );
    }

    #[test]
    fn snap_tolerates_a_fraction_of_a_step() {
        let bars: Vec<ChartBar> = [0, 300, 600, 900]
            .iter()
            .map(|offset| chart_bar(open(0) + offset, 1.0, 0.0))
            .collect();
        assert_eq!(snap_ts(&[open(0) + 610], &bars, 300), vec![open(0) + 600]);
    }

    #[test]
    fn snap_leaves_a_marker_alone_without_a_bar_in_tolerance() {
        let bars: Vec<ChartBar> = [0, 300, 600, 900]
            .iter()
            .map(|offset| chart_bar(open(0) + offset, 1.0, 0.0))
            .collect();
        let lone = [open(0) + 20_000];
        assert_eq!(snap_ts(&lone, &bars, 300), lone);
        assert_eq!(snap_ts(&lone, &[], 300), lone);
    }

    #[test]
    fn snap_reaches_the_first_and_last_bar() {
        let bars: Vec<ChartBar> = [0, 300, 600, 900]
            .iter()
            .map(|offset| chart_bar(open(0) + offset, 1.0, 0.0))
            .collect();
        assert_eq!(snap_ts(&[open(0) - 120], &bars, 300), vec![open(0)]);
        assert_eq!(snap_ts(&[open(0) + 1_000], &bars, 300), vec![open(0) + 900]);
    }
}
