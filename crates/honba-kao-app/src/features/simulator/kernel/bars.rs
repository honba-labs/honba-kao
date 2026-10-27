//! Bar loading for the Simulator chart (HBD-2026-DES-005 P2 / P6 wiring).
//!
//! The Leptos port of honba-fe `core/simulator/bars.ts`. The authoritative
//! source is `GET /bars/{id}` (`SYMBOL.EXCHANGE`, `tf` equal to the run's
//! `bar_spec`, windowed by the run's dates) — the same store the run itself
//! consumes, so candles and fills describe one series. When the store answers
//! with nothing (or the API is down) the chart falls back to the deterministic
//! NSE-session preview so the workspace stays usable; that state is reported
//! as `source: preview` and labelled in the UI, never passed off as served
//! history.
//!
//! Unlike fe, the transport lives at the call site (the chart component's
//! effect in P6 via the typed client); this module holds the pure fold from
//! client outcome to [`BarsState`], which is what the vitest suite exercised.

use honba_kao_client::ClientError;
use honba_kao_client::wire::BarsResponse;
use honba_core::market::Bar;

use crate::features::simulator::kernel::chart_data::{preview_bars, to_chart_bars, ChartBar, PreviewBarsRequest};

/// Where the displayed bars come from (`BarsSource`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarsSource {
    /// No source yet (initial state).
    None,
    /// Served history from `GET /bars/{id}`.
    Server,
    /// The deterministic local preview (always labelled in the UI).
    Preview,
}

/// Load lifecycle (`BarsStatus`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarsStatus {
    /// Not requested yet.
    Idle,
    /// A fetch is in flight.
    Loading,
    /// Bars are available (`source` says which).
    Ready,
    /// The attempt fell back because the store was unreachable.
    Error,
}

/// The bar-fetch seam the chart surface needs (fe's `BenchmarkBarsClient` /
/// the chart container's store access): `GET /bars/{id}` through the typed
/// client. Narrowed like [`crate::features::simulator::kernel::run::RunClient`] so mocks stay
/// trivial; the real impl lives next to the store's other ApiClient impls.
pub trait BarsSeam {
    fn get_bars<'a>(
        &'a self,
        id: &'a str,
        tf: Option<&'a str>,
        from: Option<&'a str>,
        to: Option<&'a str>,
    ) -> crate::features::simulator::kernel::run::ClientFut<'a, Result<BarsResponse, ClientError>>;
}

/// The chart's bar state (`BarsState`). Carries `Clone` so the reactive
/// store can hold it; never fabricated — `source` labels where bars came
/// from.
#[derive(Clone, Debug)]
pub struct BarsState {
    /// Lifecycle status.
    pub status: BarsStatus,
    /// The bars to draw.
    pub bars: Vec<ChartBar>,
    /// Where they came from.
    pub source: BarsSource,
    /// Human-readable note for the empty/preview badge.
    pub note: Option<String>,
    /// Transport error message, when the server was unreachable.
    pub error: Option<String>,
}

impl BarsState {
    /// The initial state (`initialBarsState`).
    pub fn initial() -> Self {
        Self {
            status: BarsStatus::Idle,
            bars: Vec::new(),
            source: BarsSource::None,
            note: None,
            error: None,
        }
    }
}

/// A `loadRunBars` request (`LoadBarsRequest`).
pub struct LoadBarsRequest<'a> {
    /// `SYMBOL.EXCHANGE`, the id `GET /bars/{id}` expects.
    pub instrument_id: String,
    /// Inclusive start, `YYYY-MM-DD`.
    pub from: &'a str,
    /// Exclusive end, `YYYY-MM-DD`.
    pub to: &'a str,
    /// Run `bar_spec`, e.g. `5m`.
    pub bar_spec: &'a str,
    /// Last known price, for the preview fallback.
    pub anchor: Option<f64>,
}

/// UTC seconds of `YYYY-MM-DD` T00:00Z (fe `secondsOfDate`); `0` for a
/// malformed date (an "epoch" window is simply empty).
pub fn seconds_of_date(date: &str) -> i64 {
    crate::features::simulator::kernel::controls::Today::from_iso(date).map_or(0, |today| {
        honba_api_wasm::calendar::days_from_civil(today.year as i64, today.month, today.day)
            * 86_400
    })
}

/// Build the preview series for a request (`previewFor`, exported so the
/// fallback is testable).
pub fn preview_for(request: &LoadBarsRequest<'_>) -> Vec<ChartBar> {
    preview_bars(&PreviewBarsRequest {
        instrument_id: &request.instrument_id,
        from: seconds_of_date(request.from),
        to: seconds_of_date(request.to),
        bar_spec: request.bar_spec,
        anchor: request.anchor,
    })
}

/// The preview fallback state, parametrised by what went wrong (`fallback`
/// closure in fe's `loadRunBars`).
fn fallback_state(
    request: &LoadBarsRequest<'_>,
    error: Option<String>,
    note: String,
) -> BarsState {
    BarsState {
        // fe folds a fallback to `ready` even when the store was unreachable
        // (the preview is renderable); `error` carries the transport story.
        status: BarsStatus::Ready,
        bars: preview_for(request),
        source: BarsSource::Preview,
        note: Some(note),
        error,
    }
}

/// Folds a client outcome into the bar state (`loadRunBars` body, minus the
/// transport which the caller performs once per effect run).
///
/// The caller's cancelled-run guard decides whether to keep the result; a
/// cancelled request folds exactly like an unreachable store (preview), and
/// the state is labelled, never passed off as served history.
pub fn load_run_bars_fold(
    request: &LoadBarsRequest<'_>,
    outcome: Result<Vec<Bar>, ClientError>,
) -> BarsState {
    match outcome {
        Ok(wire_bars) => {
            let bars = to_chart_bars(&wire_bars);
            if !bars.is_empty() {
                return BarsState {
                    status: BarsStatus::Ready,
                    bars,
                    source: BarsSource::Server,
                    note: None,
                    error: None,
                };
            }
            fallback_state(
                request,
                None,
                format!(
                    "The bar store has no {} bars for {} in this window; showing a preview.",
                    request.bar_spec, request.instrument_id
                ),
            )
        }
        Err(error) => {
            let message = error.to_string();
            fallback_state(
                request,
                Some(message.clone()),
                format!("Bar store unreachable ({message}); showing a preview."),
            )
        }
    }
}

/// The note used when no client is wired up at all (dev builds without the
/// API origin).
pub fn no_client_fallback_note(instrument_id: &str) -> String {
    format!("Bar API unavailable; showing a deterministic NSE-session preview for {instrument_id}.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use honba_kao_client::wire::{Exchange, InstrumentId};

    const REQUEST_FROM: &str = "2024-01-01";
    const REQUEST_TO: &str = "2024-02-01";

    fn request<'a>(bar_spec: &'a str) -> LoadBarsRequest<'a> {
        LoadBarsRequest {
            instrument_id: "TCS.NSE".to_string(),
            from: REQUEST_FROM,
            to: REQUEST_TO,
            bar_spec,
            anchor: None,
        }
    }

    fn wire_bar(seconds: u64, close: f64) -> Bar {
        use honba_core::{Price, Quantity};
        use chrono::{DateTime, Utc};
        use rust_decimal::Decimal;
        let p = Price::new(Decimal::try_from(close).unwrap_or_default());
        let instrument = InstrumentId::new(Exchange::Nse, "TCS");
        Bar {
            instrument,
            open: p,
            high: Price::new(Decimal::try_from(close + 1.0).unwrap_or_default()),
            low: Price::new(Decimal::try_from(close - 1.0).unwrap_or_default()),
            close: p,
            volume: Quantity::from_u64(5),
            timestamp: DateTime::<Utc>::from_timestamp_nanos((seconds * 1_000_000_000) as i64),
        }
    }

    #[test]
    fn starts_idle_and_empty() {
        let state = BarsState::initial();
        assert_eq!(state.status, BarsStatus::Idle);
        assert!(state.bars.is_empty());
        assert_eq!(state.source, BarsSource::None);
        assert!(state.note.is_none());
        assert!(state.error.is_none());
    }

    #[test]
    fn uses_the_served_bars_and_marks_the_source_as_server() {
        let served = vec![wire_bar(1_704_067_200, 100.0)];
        let state = load_run_bars_fold(&request("1d"), Ok(served));
        assert_eq!(state.source, BarsSource::Server);
        assert_eq!(state.status, BarsStatus::Ready);
        assert!(state.note.is_none());
        assert_eq!(
            state.bars,
            vec![ChartBar {
                time: 1_704_067_200,
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 5.0,
            }]
        );
    }

    #[test]
    fn falls_back_to_a_labelled_preview_when_the_store_has_no_bars() {
        let state = load_run_bars_fold(&request("1d"), Ok(Vec::new()));
        assert_eq!(state.source, BarsSource::Preview);
        assert!(!state.bars.is_empty());
        let note = state.note.expect("preview note");
        assert!(note.contains("no 1d bars"));
    }

    #[test]
    fn falls_back_to_a_labelled_preview_when_the_store_is_unreachable() {
        let error = ClientError::Status(503);
        let state = load_run_bars_fold(&request("1d"), Err(error.clone()));
        assert_eq!(state.source, BarsSource::Preview);
        assert!(!state.bars.is_empty());
        let error_message = state.error.expect("transport error kept");
        assert!(!error_message.is_empty());
        let note = state.note.expect("note");
        assert!(note.contains("unreachable"));
    }

    #[test]
    fn preview_for_covers_the_requested_window_on_the_grid() {
        let bars = preview_for(&request("5m"));
        assert!(!bars.is_empty());
        let from = crate::features::simulator::kernel::bars::seconds_of_date("2024-01-01");
        let to = crate::features::simulator::kernel::bars::seconds_of_date("2024-02-01");
        assert!(bars.iter().all(|bar| bar.time >= from && bar.time < to));
    }

    #[test]
    fn parses_window_dates_to_utc_midnight() {
        assert_eq!(seconds_of_date("2024-01-01"), 1_704_067_200);
        assert_eq!(seconds_of_date("2024-02-01"), 1_704_067_200 + 31 * 86_400);
        assert_eq!(seconds_of_date("not-a-date"), 0);
    }
}
