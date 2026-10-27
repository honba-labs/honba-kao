//! Run logs & events (HBD-2026-DES-005 P4).
//!
//! The Leptos port of honba-fe `core/simulator/events.ts`: ADR 0021 designs
//! run events as an SSE stream (`GET /backtests/{id}/events` →
//! [`wire::StreamFrame`]), with `resume_from_seq` so a reader picks up where
//! it left off. The SSE channel itself is opened through the browser-native
//! `EventSource` (see `honba-kao-client/src/events.rs`, the P0 transport
//! decision); the fold to a compact one-line log lives **here**, in a pure
//! push-down function, so the mapping is testable — frames are pushed to
//! [`fold_run_log`] by the drawer's event handler (P5), which also owns the
//! `EventSource` open/`resume_from_seq`/`on_cleanup` close.
//!
//! fe drained a generator with `collectRunLog`; the callback shape of
//! `EventSource` swaps the drain for "on every message, fold", which folds
//! the *same* state machine and keeps resume tokens identical.

use honba_kao_client::wire::{ErrorDetail, Event, RunStatus, StreamFrame};

/// Human-readable label for a wire [`Event`] union member, keyed on its
/// serde `type` tag (fe `describeEvent`). Unknown tags fold to their
/// un-articulated spelling; "nothing to describe" is `event`.
pub fn describe_event(event: &Event) -> String {
    #[allow(unreachable_patterns)]
    match event {
        Event::Bar(_) => "bar".to_string(),
        Event::Quote(_) => "quote".to_string(),
        Event::Order(_) => "order update".to_string(),
        Event::Fill(_) => "trade tick".to_string(),
        Event::Metric { .. } => "metric".to_string(),
        // The event union is `#[non_exhaustive]`: a newer producer's tag is
        // spelled out honestly rather than guessed.
        _ => event_type_tag(event).replace('_', " "),
    }
}

/// The serde `type` tag of an event, through its own serializer — used only
/// for variants this build does not enumerate (see [`describe_event`]).
fn event_type_tag(event: &Event) -> String {
    match serde_json::to_value(event) {
        Ok(serde_json::Value::Object(map)) => {
            if let Some(tag) = map.get("type").and_then(|tag| tag.as_str()) {
                tag.to_string()
            } else if let Some(key) = map.keys().next() {
                key.to_string()
            } else {
                "event".to_string()
            }
        }
        _ => "event".to_string(),
    }
}

/// The display line for a status frame's error detail. fe's wire type admits
/// only an `ErrorDetail` (its test fed a bare `string` off-contract to prove
/// the fallback held); the typed detail always carries code + message.
fn describe_error_detail(detail: &ErrorDetail) -> String {
    format!("{}: {}", detail.code.as_str(), detail.message)
}

/// The status label fe printed straight off the JSON string; the typed
/// [`RunStatus`] reads its own serde tag so the spelling stays server-given.
fn status_label(status: &RunStatus) -> String {
    let Ok(json) = serde_json::to_string(status) else {
        return "unknown".to_string();
    };
    json.trim_matches('"').to_string()
}

/// One compact line for a stream frame; `None` when it carries nothing to
/// show (fe `frameToLine`).
///
/// The line repertory: `#2 status: running`, `#3 status: failed — bad: code`,
/// `#13 quote`. A record's message is typed, so a "#n event" fallback like
/// fe's can only appear if a future wire version adds a frame kind the
/// match does not know — deliberately, that build failure is the contract
/// warning to regenerate the fold.
pub fn frame_to_line(frame: Option<&StreamFrame>) -> Option<String> {
    let frame = frame?;
    match frame {
        StreamFrame::Status { status, seq, error } => {
            let detail = error
                .as_ref()
                .map(|detail| format!(" — {}", describe_error_detail(detail)))
                .unwrap_or_default();
            Some(format!("#{seq} status: {}{}", status_label(status), detail))
        }
        StreamFrame::Record { seq, message } => {
            Some(format!("#{seq} {}", describe_event(&message.event)))
        }
    }
}

/// The stream's fold state (fe `RunLogState`).
#[derive(Clone, Debug, PartialEq)]
pub struct RunLogState {
    /// Human lines, newest last (`lines`).
    pub lines: Vec<String>,
    /// Highest seq folded in, so `resume_from_seq` continues rather than
    /// repeats (`lastSeq`).
    pub last_seq: u64,
    /// Terminal status observed on the stream, if any (`terminal`).
    pub terminal: Option<RunStatus>,
}

/// The state an event log starts at (`initialRunLog`).
pub fn initial_run_log() -> RunLogState {
    RunLogState {
        lines: Vec::new(),
        last_seq: 0,
        terminal: None,
    }
}

/// Log bound; the drawer shows the tail (fe `MAX_LOG_LINES`).
pub const MAX_LOG_LINES: usize = 500;

/// Fold one frame into the log: appends one line, tracks seq and terminal
/// status (`foldRunLog`). `None` frames (the fold's no-op input) return the
/// state unchanged.
pub fn fold_run_log(state: &RunLogState, frame: Option<&StreamFrame>) -> RunLogState {
    let Some(frame) = frame else {
        return state.clone();
    };
    let seq = match frame {
        StreamFrame::Status { seq, .. } | StreamFrame::Record { seq, .. } => *seq,
    };
    let line = frame_to_line(Some(frame));
    let mut lines = state.lines.clone();
    if let Some(line) = line {
        lines.push(line);
        if lines.len() > MAX_LOG_LINES {
            let drop = lines.len() - MAX_LOG_LINES;
            lines.drain(..drop);
        }
    }
    // fe cleared the terminal bit on any later status frame; a live stream
    // never sends status after terminal, so this fold *sticks* the observed
    // terminal state (a mild defensive deviation, documented here).
    let terminal = match frame {
        StreamFrame::Status { status, .. } => fold_terminal(Some(*status)).or(state.terminal),
        StreamFrame::Record { .. } => state.terminal,
    };
    let last_seq = state.last_seq.max(seq);
    RunLogState {
        lines,
        last_seq,
        terminal,
    }
}

/// folds the terminal bit change only once: `completed`/`failed`/`cancelled`
/// are terminal and stick (`foldTerminal`); anything else keeps running.
pub fn fold_terminal(status: Option<RunStatus>) -> Option<RunStatus> {
    match status {
        Some(
            status @ (RunStatus::Completed | RunStatus::Failed | RunStatus::Cancelled),
        ) => Some(status),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use honba_kao_client::wire::{
        ErrorCode as WireErrorCode, Exchange, InstrumentId, WireMessage, WireOrder,
        WireOrderSide, WireOrderType, WireQuoteTick,
    };
    use rust_decimal::Decimal;

    fn status_frame(seq: u64, status: RunStatus) -> StreamFrame {
        StreamFrame::Status {
            status,
            seq,
            error: None,
        }
    }

    fn failed_frame(seq: u64) -> StreamFrame {
        StreamFrame::Status {
            status: RunStatus::Failed,
            seq,
            error: Some(ErrorDetail::new(
                WireErrorCode::ValidationInvalidRequest,
                "bad manifest",
            )),
        }
    }

    fn record_frame(seq: u64, event: Event) -> StreamFrame {
        StreamFrame::Record {
            seq,
            message: WireMessage::new(event, Utc::now()),
        }
    }

    #[test]
    fn labels_every_wire_event_mapping_fe_cares_about() {
        // fe's `describeEvent` cases, keyed on the typed union now.
        let quote = WireQuoteTick::new(
            InstrumentId::new(Exchange::Nse, "TCS".to_string()),
            Decimal::from(100),
            Decimal::from(101),
            Decimal::from(1),
            Decimal::from(2),
            Utc::now(),
            Utc::now(),
        );
        assert_eq!(describe_event(&Event::Quote(quote)), "quote");
        let fill = honba_core::FillReport::new(
            "f",
            "o",
            honba_core::uuid::Uuid::nil(),
            InstrumentId::new(Exchange::Nse, "TCS"),
            WireOrderSide::Buy,
            honba_core::Price::ZERO,
            honba_core::Quantity::ZERO,
            "algo",
            chrono::Utc::now(),
        );
        assert_eq!(describe_event(&Event::Fill(fill)), "trade tick");
        let order = WireOrder::new(
            InstrumentId::new(Exchange::Nse, "TCS".to_string()),
            WireOrderSide::Buy,
            WireOrderType::Market,
            honba_core::Quantity::from_u64(10),
            "algo".to_string(),
        );
        assert_eq!(describe_event(&Event::Order(order)), "order update");
    }

    #[test]
    fn an_unknown_event_kind_is_never_guessed_by_this_build() {
        let event = Event::Metric {
            name: "custom_stat".to_string(),
            value: serde_json::json!(42),
        };
        assert_eq!(describe_event(&event), "metric");
        assert_eq!(event_type_tag(&event), "Metric");
    }

    #[test]
    fn status_lines_carry_the_spelling_and_the_error_detail() {
        assert_eq!(
            frame_to_line(Some(&status_frame(2, RunStatus::Running))),
            Some("#2 status: running".to_string())
        );
        assert_eq!(
            frame_to_line(Some(&failed_frame(3))),
            Some("#3 status: failed — validation_invalid_request: bad manifest".to_string())
        );
        assert_eq!(frame_to_line(None), None);
    }

    #[test]
    fn record_lines_name_the_event() {
        let quote = WireQuoteTick::new(
            InstrumentId::new(Exchange::Nse, "TCS".to_string()),
            Decimal::from(100),
            Decimal::from(101),
            Decimal::from(1),
            Decimal::from(2),
            Utc::now(),
            Utc::now(),
        );
        assert_eq!(
            frame_to_line(Some(&record_frame(13, Event::Quote(quote)))),
            Some("#13 quote".to_string())
        );
    }

    #[test]
    fn the_fold_tracks_seq_terminal_and_the_tail_cap() {
        let mut state = initial_run_log();
        for seq in 1..=600u64 {
            state = fold_run_log(&state, Some(&status_frame(seq, RunStatus::Running)));
        }
        assert_eq!(state.lines.len(), MAX_LOG_LINES);
        assert_eq!(state.last_seq, 600);
        assert_eq!(state.terminal, None);
        assert_eq!(state.lines.last().unwrap(), "#600 status: running");

        let terminal = fold_run_log(
            &state,
            Some(&status_frame(601, RunStatus::Completed)),
        );
        assert_eq!(terminal.terminal, Some(RunStatus::Completed));
        assert_eq!(terminal.lines.last().unwrap(), "#601 status: completed");

        // A non-terminal frame must not *unstick* an observed terminal state.
        let keep = fold_run_log(&terminal, Some(&status_frame(602, RunStatus::Running)));
        assert_eq!(keep.terminal, Some(RunStatus::Completed));
    }

    #[test]
    fn folds_terminal_only_for_terminal_statuses() {
        assert_eq!(fold_terminal(Some(RunStatus::Completed)), Some(RunStatus::Completed));
        assert_eq!(fold_terminal(Some(RunStatus::Failed)), Some(RunStatus::Failed));
        assert_eq!(fold_terminal(Some(RunStatus::Cancelled)), Some(RunStatus::Cancelled));
        assert_eq!(fold_terminal(Some(RunStatus::Running)), None);
        assert_eq!(fold_terminal(Some(RunStatus::Pending)), None);
        assert_eq!(fold_terminal(None), None);
    }

    #[test]
    fn a_null_frame_is_a_no_op_fold() {
        let state = initial_run_log();
        assert_eq!(fold_run_log(&state, None), state);
    }
}
