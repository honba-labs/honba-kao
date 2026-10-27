//! Journal loading (HBD-2026-DES-005 P4).
//!
//! The Leptos port of honba-fe `core/simulator/journal.ts`: `GET
//! /backtests/{id}/journal` answers with **fills** ([`wire::Trade`]), which
//! the chart and the analytics surface pair into round trips
//! ([`honba_api_wasm::pairing`]). This module owns the fetch state only; it never
//! pairs, so the pairing rules stay in one tested place.
//!
//! The client seam is a narrow trait, exactly as the run loop's is, so the
//! store degrades against a mock without the route instead of failing.

use honba_kao_client::{ClientError, wire::TradesResponse};

/// The journal fetch's status (`JournalStatus`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum JournalStatus {
    /// Nothing fetched yet.
    #[default]
    Idle,
    /// A fetch is in flight.
    Loading,
    /// Fills are in.
    Ready,
    /// The fetch failed; the run itself may still be completed.
    Error,
}

/// The journal fetch's state (fe `JournalState`).
#[derive(Clone, Debug, PartialEq)]
pub struct JournalState {
    /// Fetch status.
    pub status: JournalStatus,
    /// Which run these fills belong to (`runId`).
    pub run_id: Option<String>,
    /// The run's fills.
    pub trades: Vec<honba_kao_client::wire::Trade>,
    /// Human reason on [`JournalStatus::Error`].
    pub error: Option<String>,
}

/// The state a journal slice starts at (`initialJournalState`).
pub fn initial_journal_state() -> JournalState {
    JournalState {
        status: JournalStatus::Idle,
        run_id: None,
        trades: Vec::new(),
        error: None,
    }
}

/// The client slice a journal load needs (fe `JournalClient`).
pub trait JournalClient {
    /// `GET /backtests/{id}/journal`.
    fn get_backtest_journal<'a>(
        &'a self,
        id: &'a str,
    ) -> crate::features::simulator::kernel::run::ClientFut<'a, Result<TradesResponse, ClientError>>;
}

/// Marks a load as in flight for `run_id` (`applyJournalLoading`).
pub fn apply_journal_loading(state: &JournalState, run_id: &str) -> JournalState {
    let mut loading = state.clone();
    loading.status = JournalStatus::Loading;
    loading.run_id = Some(run_id.to_string());
    loading.error = None;
    loading
}

/// Fold a journal answer, guarding it against a newer load
/// (`applyJournalResponse`): only a `loading` state keyed to exactly this
/// `run_id` accepts fills — a newer load owns the state.
pub fn apply_journal_response(
    state: &JournalState,
    run_id: &str,
    trades: Vec<honba_kao_client::wire::Trade>,
) -> JournalState {
    if state.run_id.as_deref() != Some(run_id) || state.status != JournalStatus::Loading {
        return state.clone();
    }
    JournalState {
        status: JournalStatus::Ready,
        run_id: Some(run_id.to_string()),
        trades,
        error: None,
    }
}

/// Fold a journal failure, same ownership guard (`applyJournalError`).
pub fn apply_journal_error(state: &JournalState, run_id: &str, message: &str) -> JournalState {
    if state.run_id.as_deref() != Some(run_id) || state.status != JournalStatus::Loading {
        return state.clone();
    }
    JournalState {
        status: JournalStatus::Error,
        run_id: Some(run_id.to_string()),
        trades: Vec::new(),
        error: Some(message.to_string()),
    }
}

/// Fetch the fills of a finished run (`loadJournal`).
///
/// A missing route is not an error the user should see: it resolves to an
/// `error` state with the usual message, and the chart simply renders no
/// markers.
pub async fn load_journal<C: JournalClient>(client: &C, run_id: &str) -> JournalState {
    let loading = apply_journal_loading(&initial_journal_state(), run_id);
    match client.get_backtest_journal(run_id).await {
        Ok(response) => apply_journal_response(&loading, run_id, response.trades),
        Err(error) => apply_journal_error(&loading, run_id, &error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::simulator::kernel::run::ClientFut;
    use honba_kao_client::wire::{Exchange, InstrumentId, Trade, WireOrderSide};

    struct OkJournal(Vec<Trade>);
    struct FailingJournal;

    impl JournalClient for OkJournal {
        fn get_backtest_journal<'a>(
            &'a self,
            _id: &'a str,
        ) -> ClientFut<'a, Result<TradesResponse, ClientError>> {
            let trades = self.0.clone();
            Box::pin(async move { Ok(TradesResponse { trades }) })
        }
    }

    impl JournalClient for FailingJournal {
        fn get_backtest_journal<'a>(
            &'a self,
            _id: &'a str,
        ) -> ClientFut<'a, Result<TradesResponse, ClientError>> {
            Box::pin(async move { Err(ClientError::Status(404)) })
        }
    }

    fn fill(minute: i64) -> Trade {
        let ts = chrono::DateTime::from_timestamp(1_704_153_600 + minute * 60, 0)
            .expect("valid timestamp");
        Trade::new(
            "f",
            "o",
            honba_core::uuid::Uuid::nil(),
            InstrumentId::new(Exchange::Nse, "TCS"),
            WireOrderSide::Buy,
            honba_core::Price::new(rust_decimal::Decimal::from(100)),
            honba_core::Quantity::from_u64(10),
            "algo",
            ts,
        )
    }

    #[test]
    fn a_completed_run_loads_its_fills_into_a_ready_state() {
        let state = crate::features::simulator::kernel::testing::block_on(async {
            load_journal(&OkJournal(vec![fill(0), fill(10)]), "run-1").await
        });
        assert_eq!(state.status, JournalStatus::Ready);
        assert_eq!(state.run_id.as_deref(), Some("run-1"));
        assert_eq!(state.trades.len(), 2);
        assert_eq!(state.error, None);
    }

    #[test]
    fn a_missing_route_degrades_to_an_error_state_and_no_markers() {
        let state = crate::features::simulator::kernel::testing::block_on(async {
            load_journal(&FailingJournal, "run-1").await
        });
        assert_eq!(state.status, JournalStatus::Error);
        assert!(state.trades.is_empty());
        assert_eq!(state.error.as_deref(), Some("status 404"));
    }

    #[test]
    fn a_newer_load_owns_the_state_and_ignores_a_stale_answer() {
        let loading = apply_journal_loading(&initial_journal_state(), "run-1");
        // The stale answer arrives after the state was re-keyed to run-2.
        let stale = apply_journal_response(
            &apply_journal_loading(&loading, "run-2"),
            "run-1",
            vec![fill(0)],
        );
        assert_eq!(stale.status, JournalStatus::Loading);
        assert_eq!(stale.run_id.as_deref(), Some("run-2"));
        assert!(stale.trades.is_empty());
    }

    #[test]
    fn only_a_loading_state_accepts_an_answer() {
        let idle = initial_journal_state();
        let rejected = apply_journal_response(&idle, "run-1", vec![fill(0)]);
        assert_eq!(rejected, idle);
        let rejected_error = apply_journal_error(&idle, "run-1", "x");
        assert_eq!(rejected_error, idle);
    }
}
