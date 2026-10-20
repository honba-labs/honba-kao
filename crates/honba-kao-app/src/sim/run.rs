//! Backtest run state + the submit/poll loop (HBD-2026-DES-005 P4).
//!
//! The Leptos port of honba-fe `core/simulator/run.ts`. `RunStatus` on the
//! wire is `pending | running | completed | failed | cancelled`; the simulator
//! folds `pending`/`running` into `running` and keeps the terminal states
//! distinct. The polling loop is deliberately injected — [`RunClient`] (the
//! two routes it needs) and [`Clock`] (the sleep) — so it is exercised in
//! tests without a live server.
//!
//! fe owned the loop to an `AbortSignal`; the typed port uses the
//! cooperative [`RunAbort`] flag: the loop checks it before every sleep and
//! folds an abort into a failed state, so an `on_cleanup` close cannot leave
//! the signal in "running".
//!
//! Honesty invariants carried over: a poll cap folds into `failed` (the UI
//! never hangs on running); a failed poll carries the wire `error` detail
//! (`{code}: {message}`); http/textarea errors become the run error text
//! verbatim.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use honba_kao_client::wire::{BacktestRequest, BacktestResponse, RunStatus};
use honba_kao_client::ClientError;

/// A boxed future, so client doubles can be type-erased trait objects
/// (`dyn`-dispatch is what keeps fe's structural client mock viable here).
pub type ClientFut<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// The simulator's folded run status (fe `SimRunStatus`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SimRunStatus {
    /// Nothing has been submitted yet.
    #[default]
    Idle,
    /// Submitted and awaiting a terminal state.
    Running,
    /// Finished successfully.
    Completed,
    /// Failed to submit, to poll, aborted, or timed out.
    Failed,
    /// Stopped before completing.
    Cancelled,
}

impl SimRunStatus {
    /// fe's wire spelling, shown by the run header.
    pub const fn as_str(self) -> &'static str {
        match self {
            SimRunStatus::Idle => "idle",
            SimRunStatus::Running => "running",
            SimRunStatus::Completed => "completed",
            SimRunStatus::Failed => "failed",
            SimRunStatus::Cancelled => "cancelled",
        }
    }
}

/// The run's machine state (fe `SimRunState`).
///
/// This is a *fold target*, not a global: the store holds it in one signal
/// and replaces it wholesale on each transition, exactly like fe's zustand
/// slice did.
#[derive(Clone, Debug, PartialEq)]
pub struct SimRunState {
    /// Folded status (`status`).
    pub status: SimRunStatus,
    /// Server-assigned run id (`runId`).
    pub run_id: Option<String>,
    /// The last submit/poll answer (`response`).
    pub response: Option<BacktestResponse>,
    /// Human reason when folded to a failure (`error`).
    pub error: Option<String>,
    /// Progress lines, newest last (`logs`).
    pub logs: Vec<String>,
}

/// The state a fresh run slice starts at (`initialRunState`).
pub fn initial_run_state() -> SimRunState {
    SimRunState {
        status: SimRunStatus::Idle,
        run_id: None,
        response: None,
        error: None,
        logs: Vec::new(),
    }
}

/// `true` for the folded terminal states (`isTerminalStatus`, fe spelled the
/// union `RunStatus | SimRunStatus`; the solo/folded pair is split here).
pub fn is_terminal_status(status: SimRunStatus) -> bool {
    matches!(
        status,
        SimRunStatus::Completed | SimRunStatus::Failed | SimRunStatus::Cancelled
    )
}

/// `true` for the wire's terminal statuses (`isTerminalStatus`, wire side).
/// [`RunStatus`] is `#[non_exhaustive]`, so an unknown future status lands in
/// the catch-all as *not* terminal — reporting a brand-new status as terminal
/// would hide it from the poll loop.
pub fn is_terminal_wire(status: RunStatus) -> bool {
    match status {
        RunStatus::Completed | RunStatus::Failed | RunStatus::Cancelled => true,
        RunStatus::Pending | RunStatus::Running => false,
        _ => false,
    }
}

/// Folds a wire status into the simulator's (`foldStatus`); `None` (fe's
/// `null | undefined`) and any status this build does not know fold to
/// `idle`, exactly like fe's `default:` arm.
pub fn fold_status(status: Option<&RunStatus>) -> SimRunStatus {
    match status {
        Some(RunStatus::Pending | RunStatus::Running) => SimRunStatus::Running,
        Some(RunStatus::Completed) => SimRunStatus::Completed,
        Some(RunStatus::Cancelled) => SimRunStatus::Cancelled,
        Some(RunStatus::Failed) => SimRunStatus::Failed,
        Some(_) | None => SimRunStatus::Idle,
    }
}

/// One flat progress line appended by every fold.
fn push_log(state: &mut SimRunState, line: String) {
    state.logs.push(line);
}

/// The wire error detail's display line: `{code}: {message}` — fe read
/// `response.error.code`/`.message` off the JSON; on this side [`RunStatus`]'s
/// companion detail is typed and its code carries its own wire string.
pub fn error_line(detail: &honba_kao_client::wire::ErrorDetail) -> String {
    format!("{}: {}", detail.code.as_str(), detail.message)
}

/// Fold a submit/poll answer into the run state (`applyRunResponse`).
///
/// An absent payload keeps the state and logs the complaint; an answer
/// carrying a wire error folds to `failed` regardless of the status field.
pub fn apply_run_response(state: &SimRunState, response: Option<&BacktestResponse>) -> SimRunState {
    let Some(response) = response else {
        let mut folded = state.clone();
        push_log(&mut folded, "The server returned no run payload.".to_string());
        return folded;
    };
    let run_id = if response.run_id.is_empty() {
        state.run_id.clone()
    } else {
        Some(response.run_id.clone())
    };
    let error = response.error.as_ref().map(error_line);
    let status = if error.is_some() {
        SimRunStatus::Failed
    } else {
        fold_status(Some(&response.status))
    };
    let mut folded = state.clone();
    folded.status = status;
    folded.run_id = run_id;
    folded.response = Some(response.clone());
    folded.error = error;
    let line_run_id = folded.run_id.clone().unwrap_or_default();
    let line = format!("Run {line_run_id} → {}", folded.status.as_str());
    push_log(&mut folded, line);
    folded
}

/// Fold a transport/client failure into the run state (`applyRunError`).
pub fn apply_run_error(state: &SimRunState, message: &str) -> SimRunState {
    let mut folded = state.clone();
    folded.status = SimRunStatus::Failed;
    folded.error = Some(message.to_string());
    push_log(&mut folded, format!("Run failed: {message}"));
    folded
}

/// The client slice the polling loop needs (fe `RunClient`), narrowed for
/// mocking.
pub trait RunClient {
    /// `POST /backtests` — submit a run.
    fn post_backtests<'a>(
        &'a self,
        request: &'a BacktestRequest,
    ) -> ClientFut<'a, Result<BacktestResponse, ClientError>>;
    /// `GET /backtests/{id}` — poll the run.
    fn get_backtest<'a>(&'a self, id: &'a str)
    -> ClientFut<'a, Result<BacktestResponse, ClientError>>;
}

/// The injected sleep (fe's injectable `sleep`); tests fold in an instant
/// clock, the wasm app a `setTimeout` one.
pub trait Clock {
    /// Waits `ms` milliseconds.
    fn sleep<'a>(&'a self, ms: u64) -> ClientFut<'a, ()>;
}

/// Cooperative stop flag standing in for fe's `AbortSignal`; clone it, share
/// it, set it from `on_cleanup` — the loop folds an abort into `failed`.
#[derive(Clone, Debug, Default)]
pub struct RunAbort(Arc<AtomicBool>);

impl RunAbort {
    /// A clear flag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests the loop to stop.
    pub fn abort(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Reads the flag.
    pub fn is_aborted(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// The run loop's options (fe `RunOptions`).
pub struct RunOptions<'a> {
    /// Millis between polls.
    pub interval_ms: u64,
    /// Maximum number of polls before giving up.
    pub max_polls: u32,
    /// Abort flag checked between polls.
    pub abort: Option<RunAbort>,
    /// Injected clock.
    pub clock: &'a dyn Clock,
    /// Called on every state transition (the store writes its signal here).
    pub on_update: Option<RunUpdate<'a>>,
}

/// The boxed release hook `RunOptions` reports through.
pub type RunUpdate<'a> = Box<dyn FnMut(&SimRunState) + 'a>;

impl<'a> RunOptions<'a> {
    /// fe's defaults: poll every second, at most 120 times.
    pub fn new(clock: &'a dyn Clock) -> Self {
        Self {
            interval_ms: 1_000,
            max_polls: 120,
            abort: None,
            clock,
            on_update: None,
        }
    }
}

/// Reports a state to the caller, mirroring fe's `emit`.
fn emit(options: &mut RunOptions<'_>, state: SimRunState) -> SimRunState {
    if let Some(on_update) = options.on_update.as_mut() {
        on_update(&state);
    }
    state
}

/// Submit a backtest and fold the run to a terminal state
/// (`runBacktest`).
///
/// The submit answer may already be terminal; otherwise the loop polls
/// `GET /backtests/{id}` until terminal, the abort flag trips or `max_polls`
/// is hit. A poll-cap exit reports as a failure so the UI never hangs on
/// "running".
pub async fn run_backtest<C: RunClient>(
    client: &C,
    request: &BacktestRequest,
    options: &mut RunOptions<'_>,
) -> SimRunState {
    let interval_ms = options.interval_ms;
    let max_polls = options.max_polls;

    let strategy = request.strategy.as_deref().unwrap_or("strategy");
    let universe = request.universe.as_deref().unwrap_or("universe");
    let mut state = SimRunState {
        status: SimRunStatus::Running,
        ..initial_run_state()
    };
    push_log(
        &mut state,
        format!("Submitting {strategy} over {universe}…"),
    );
    state = emit(options, state);

    state = match client.post_backtests(request).await {
        Ok(response) => apply_run_response(&state, Some(&response)),
        Err(error) => apply_run_error(&state, &error.to_string()),
    };
    state = emit(options, state);

    let mut polls = 0u32;
    while let Some(run_id) = state.run_id.clone() {
        if is_terminal_status(state.status) || polls >= max_polls {
            break;
        }
        if options.abort.as_ref().is_some_and(|abort| abort.is_aborted()) {
            state = emit(options, apply_run_error(&state, "run aborted"));
            break;
        }
        options.clock.sleep(interval_ms).await;
        polls += 1;
        state = match client.get_backtest(&run_id).await {
            Ok(response) => apply_run_response(&state, Some(&response)),
            Err(error) => apply_run_error(&state, &error.to_string()),
        };
        state = emit(options, state);
    }

    if state.run_id.is_some() && !is_terminal_status(state.status) {
        let message = format!(
            "run {} did not finish within {max_polls} polls",
            state.run_id.clone().unwrap_or_default()
        );
        state = emit(options, apply_run_error(&state, &message));
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::testing::block_on;
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;

    /// fe's injectable instant clock: the sleep resolves on the first poll.
    struct InstantClock;
    impl Clock for InstantClock {
        fn sleep<'a>(&'a self, _ms: u64) -> ClientFut<'a, ()> {
            Box::pin(async {})
        }
    }

    fn completed_response(run_id: &str) -> BacktestResponse {
        BacktestResponse {
            run_id: run_id.to_string(),
            status: RunStatus::Completed,
            metrics: None,
            assumptions: None,
            validation: None,
            error: None,
        }
    }

    fn running_response(run_id: &str) -> BacktestResponse {
        BacktestResponse {
            run_id: run_id.to_string(),
            status: RunStatus::Running,
            metrics: None,
            assumptions: None,
            validation: None,
            error: None,
        }
    }

    fn failed_wire_response(run_id: &str) -> BacktestResponse {
        BacktestResponse {
            run_id: run_id.to_string(),
            status: RunStatus::Failed,
            metrics: None,
            assumptions: None,
            validation: None,
            error: Some(honba_kao_client::wire::ErrorDetail::new(
                honba_kao_client::wire::ErrorCode::ValidationInvalidRequest,
                "bad request",
            )),
        }
    }

    fn request() -> BacktestRequest {
        BacktestRequest {
            strategy: Some("TCS cross".to_string()),
            universe: Some("TCS.NSE".to_string()),
            ..Default::default()
        }
    }

    fn options<'a>() -> RunOptions<'a> {
        // A leaked clock is fine here: the tests never outlive the leak, and
        // borrows keep the options type's lifetime story exercised.
        static CLOCK: InstantClock = InstantClock;
        RunOptions::new(&CLOCK)
    }

    /// `FakeClient` in fe's `run.test.ts`: a queued submit answer, then a
    /// poll queue; when the queue is dry it answers `running` forever.
    struct QueuedClient {
        submit: Result<BacktestResponse, ClientError>,
        polls: RefCell<VecDeque<BacktestResponse>>,
        submit_calls: Cell<u32>,
        poll_calls: Cell<u32>,
    }

    impl QueuedClient {
        fn completed(submit: BacktestResponse, polls: Vec<BacktestResponse>) -> Self {
            Self {
                submit: Ok(submit),
                polls: RefCell::new(polls.into()),
                submit_calls: Cell::new(0),
                poll_calls: Cell::new(0),
            }
        }
    }

    impl RunClient for QueuedClient {
        fn post_backtests<'a>(
            &'a self,
            _request: &'a BacktestRequest,
        ) -> ClientFut<'a, Result<BacktestResponse, ClientError>> {
            self.submit_calls.set(self.submit_calls.get() + 1);
            let answer = match &self.submit {
                Ok(response) => Ok(response.clone()),
                Err(error) => Err(error.clone()),
            };
            Box::pin(async move { answer })
        }

        fn get_backtest<'a>(
            &'a self,
            _id: &'a str,
        ) -> ClientFut<'a, Result<BacktestResponse, ClientError>> {
            self.poll_calls.set(self.poll_calls.get() + 1);
            let polled = self
                .polls
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| running_response("r1"));
            Box::pin(async move { Ok(polled) })
        }
    }

    #[test]
    fn folds_pending_and_running_into_running() {
        assert_eq!(fold_status(None), SimRunStatus::Idle);
        assert_eq!(fold_status(Some(&RunStatus::Pending)), SimRunStatus::Running);
        assert_eq!(fold_status(Some(&RunStatus::Running)), SimRunStatus::Running);
        assert_eq!(fold_status(Some(&RunStatus::Completed)), SimRunStatus::Completed);
        assert_eq!(fold_status(Some(&RunStatus::Failed)), SimRunStatus::Failed);
        assert_eq!(fold_status(Some(&RunStatus::Cancelled)), SimRunStatus::Cancelled);
    }

    #[test]
    fn terminal_states_are_completed_failed_and_cancelled() {
        assert!(!is_terminal_status(SimRunStatus::Idle));
        assert!(!is_terminal_status(SimRunStatus::Running));
        assert!(is_terminal_status(SimRunStatus::Completed));
        assert!(is_terminal_status(SimRunStatus::Failed));
        assert!(is_terminal_status(SimRunStatus::Cancelled));

        assert!(is_terminal_wire(RunStatus::Completed));
        assert!(is_terminal_wire(RunStatus::Failed));
        assert!(is_terminal_wire(RunStatus::Cancelled));
        assert!(!is_terminal_wire(RunStatus::Pending));
        assert!(!is_terminal_wire(RunStatus::Running));
    }

    #[test]
    fn a_running_then_completed_poll_reaches_completed_and_logs_the_walk() {
        let client = QueuedClient::completed(
            running_response("r1"),
            vec![running_response("r1"), completed_response("r1")],
        );
        let mut run_options = options();
        let state = block_on(async {
            run_backtest(&client, &request(), &mut run_options).await
        });
        assert_eq!(state.status, SimRunStatus::Completed);
        assert_eq!(state.run_id.as_deref(), Some("r1"));
        assert_eq!(state.error, None);
        assert_eq!(client.submit_calls.get(), 1);
        assert_eq!(client.poll_calls.get(), 2);
        assert_eq!(state.logs[0], "Submitting TCS cross over TCS.NSE…");
        // The submit answer and the first poll each log their status.
        assert_eq!(state.logs[1], "Run r1 → running");
        assert_eq!(state.logs[2], "Run r1 → running");
        assert_eq!(state.logs[3], "Run r1 → completed");
    }

    #[test]
    fn a_submit_answer_carrying_a_wire_error_folds_to_failed() {
        let client = QueuedClient::completed(failed_wire_response("r1"), vec![]);
        let mut run_options = options();
        let state = block_on(async {
            run_backtest(&client, &request(), &mut run_options).await
        });
        assert_eq!(state.status, SimRunStatus::Failed);
        assert_eq!(
            state.error.as_deref(),
            Some("validation_invalid_request: bad request")
        );
        // No poll attempts after a failed submit answer.
        assert_eq!(client.poll_calls.get(), 0);
    }

    #[test]
    fn a_transport_failure_becomes_a_failed_run_with_the_client_message() {
        let client = QueuedClient {
            submit: Err(ClientError::Network("down".to_string())),
            polls: RefCell::new(VecDeque::new()),
            submit_calls: Cell::new(0),
            poll_calls: Cell::new(0),
        };
        let mut run_options = options();
        let state = block_on(async {
            run_backtest(&client, &request(), &mut run_options).await
        });
        assert_eq!(state.status, SimRunStatus::Failed);
        assert_eq!(state.error.as_deref(), Some("network: down"));
        assert_eq!(state.logs.last().unwrap(), "Run failed: network: down");
    }

    #[test]
    fn an_empty_run_id_keeps_the_previous_one() {
        let mut state = initial_run_state();
        let empty = BacktestResponse {
            run_id: String::new(),
            status: RunStatus::Running,
            metrics: None,
            assumptions: None,
            validation: None,
            error: None,
        };
        state.run_id = Some("r1".to_string());
        state = apply_run_response(&state, Some(&empty));
        assert_eq!(state.run_id.as_deref(), Some("r1"));
    }

    #[test]
    fn a_missing_payload_keeps_the_state_and_logs_the_complaint() {
        let state = apply_run_response(&initial_run_state(), None);
        assert_eq!(state.status, SimRunStatus::Idle);
        assert_eq!(state.logs, ["The server returned no run payload."]);
    }

    #[test]
    fn a_stuck_run_folds_to_failed_at_the_poll_cap() {
        let client = QueuedClient::completed(running_response("r1"), vec![]);
        let mut run_options = options();
        run_options.max_polls = 2;
        let state = block_on(async {
            run_backtest(&client, &request(), &mut run_options).await
        });
        assert_eq!(state.status, SimRunStatus::Failed);
        assert_eq!(
            state.error.as_deref(),
            Some("run r1 did not finish within 2 polls")
        );
        assert_eq!(client.poll_calls.get(), 2);
    }

    #[test]
    fn an_abort_folds_to_failed_and_never_polls() {
        let client = QueuedClient::completed(running_response("r1"), vec![]);
        let abort = RunAbort::new();
        abort.abort();
        let mut run_options = options();
        run_options.abort = Some(abort);
        let state = block_on(async {
            run_backtest(&client, &request(), &mut run_options).await
        });
        assert_eq!(state.status, SimRunStatus::Failed);
        assert_eq!(state.error.as_deref(), Some("run aborted"));
        assert_eq!(client.poll_calls.get(), 0);
    }

    #[test]
    fn a_terminal_submit_skips_the_poll_loop() {
        let client = QueuedClient::completed(completed_response("r1"), vec![]);
        let mut run_options = options();
        let state = block_on(async {
            run_backtest(&client, &request(), &mut run_options).await
        });
        assert_eq!(state.status, SimRunStatus::Completed);
        assert_eq!(client.poll_calls.get(), 0);
    }

    #[test]
    fn on_update_sees_every_transition() {
        let client = QueuedClient::completed(
            running_response("r1"),
            vec![completed_response("r1")],
        );
        // `seen` before `run_options`: the on-update closure borrows it, and
        // reverse drop order keeps the borrow valid until the options drop.
        let seen: RefCell<Vec<SimRunStatus>> = RefCell::new(Vec::new());
        let mut run_options = options();
        {
            let seen = &seen;
            run_options.on_update =
                Some(Box::new(move |state: &SimRunState| {
                    seen.borrow_mut().push(state.status)
                }));
        }
        let state = block_on(async {
            run_backtest(&client, &request(), &mut run_options).await
        });
        drop(state);
        assert_eq!(
            seen.borrow().as_slice(),
            &[
                SimRunStatus::Running,
                SimRunStatus::Running,
                SimRunStatus::Completed,
            ][..]
        );
    }
}
