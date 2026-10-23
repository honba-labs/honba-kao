//! Pure Journal v1 replay, free of any wasm-bindgen types.
//!
//! Everything here runs natively, so it is conformance-tested without a wasm runtime (see
//! `tests/replay_conformance.rs`). Rust callers use the typed [`journal_header`] and
//! [`replay_journal`]; the `#[wasm_bindgen]` layer in [`crate`] forwards to the JSON-string
//! wrappers [`journal_header_json`] and [`replay_journal_json`], which render those results.
//!
//! The audit trail itself is `honba_engine::audit` (L3). The browser has no filesystem, so the
//! NDJSON crosses the boundary as an in-memory string: this module parses it with
//! [`std::io::BufReader`] over the string's bytes (no file handle) and answers with a JSON string.
//! Nothing is reimplemented — the header, the records and the reconstructed [`ReplayState`] all
//! come from the engine.
//!
//! # Wire contract
//!
//! A journal is NDJSON: the first non-empty line is the provenance header (its `type` is
//! `"header"`), and every later line is one audit record. [`journal_header_json`] returns the
//! [`JournalHeader`] as JSON; [`replay_journal_json`] returns
//! `{"header": <JournalHeader>, "replay": <summary>}` where `summary` is
//! `{"trading_state", "dispatches", "positions", "orders", "rejections"}` with `positions` and
//! `orders` sorted by their key and `rejections` keyed by order id. The shape is deterministic
//! and stable.
//!
//! Invalid input is refused with a message rather than answered: an empty stream, a missing
//! header, a malformed line, or an unsupported journal version all throw a JavaScript `Error`
//! carrying the underlying `io`/`serde` message. No field is ever `NaN` or infinite; a
//! non-finite value that somehow reached the log is rendered as `null`.

use std::collections::BTreeMap;
use std::io::BufReader;

use honba_engine::{AuditKind, AuditLog, JournalHeader, JournalV1, ReplayState};
use honba_core::{OrderEventKind, OrderSide};
use honba_api::TradingState;
use serde_json::{Value, json};

/// Why a journal request was refused: a stable `reason` in a `reason: message` pair
/// (ADR 0012 rule 4).
///
/// Malformed header JSON classifies as [`ReplayError::InvalidHeaderJson`]; a header that parses
/// but carries wrong field values classifies as [`ReplayError::InvalidHeader`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayError {
    /// The stream has no non-empty line at all.
    EmptyJournal,
    /// The first line fails header JSON parsing.
    InvalidHeaderJson,
    /// The first line is not a journal v1 header.
    MissingHeader,
    /// The header fails `JournalHeader` deserialization.
    InvalidHeader,
    /// `journal_version` is not 1.
    UnsupportedJournalVersion,
    /// A later line fails record JSON parsing or record deserialization.
    InvalidRecord,
    /// The header fails JSON serialization for the answer.
    HeaderSerialize,
}

impl ReplayError {
    /// A stable machine-readable reason string.
    #[must_use]
    pub const fn reason(self) -> &'static str {
        match self {
            Self::EmptyJournal => "empty_journal",
            Self::InvalidHeaderJson => "invalid_header_json",
            Self::MissingHeader => "missing_header",
            Self::InvalidHeader => "invalid_header",
            Self::UnsupportedJournalVersion => "unsupported_journal_version",
            Self::InvalidRecord => "invalid_record",
            Self::HeaderSerialize => "header_serialize",
        }
    }
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: journal rejected", self.reason())
    }
}

/// Parses an in-memory NDJSON journal without touching the filesystem.
///
/// Every refusal is `(reason, message)`: the stable `reason` string comes from
/// `ReplayError::reason`, the message is the reader's original diagnostic, separated by
/// `": "` so the [`crate`] boundary can split them apart again.
/// Classifies one `read_ndjson` failure. The reader carries no phase, so header failures are
/// told apart by their engine message: `"missing journal v1 header"` is a missing header,
/// `"unsupported journal version: N"` a version refusal, and any header JSON or value failure
/// lands as the malformed-header or bad-header reason respectively; record failures carry the
/// record text.
fn classify(error: &std::io::Error, header_line: Option<&str>, record_lines: usize) -> ReplayError {
    let message = error.to_string();
    match error.kind() {
        std::io::ErrorKind::UnexpectedEof => ReplayError::EmptyJournal,
        std::io::ErrorKind::InvalidData if message == "missing journal v1 header" => {
            ReplayError::MissingHeader
        }
        std::io::ErrorKind::InvalidData if message.starts_with("unsupported journal version:") => {
            ReplayError::UnsupportedJournalVersion
        }
        // A header-value failure names the field; anything else from the header line is
        // malformed header JSON.
        std::io::ErrorKind::InvalidData if record_lines == 0 && header_line.is_some() => {
            if message.contains("missing field")
                || message.contains("invalid type")
                || message.contains("invalid value")
            {
                ReplayError::InvalidHeader
            } else {
                ReplayError::InvalidHeaderJson
            }
        }
        _ => ReplayError::InvalidRecord,
    }
}

/// A refused journal: the stable [`ReplayError`] classification plus the reader's original
/// diagnostic. Displays as the `reason: message` pair the wasm boundary throws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalError {
    /// The stable classification.
    pub reason: ReplayError,
    /// The underlying `io`/`serde` message.
    pub message: String,
}

impl std::fmt::Display for JournalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.reason.reason(), self.message)
    }
}

impl std::error::Error for JournalError {}

fn parse_journal(journal_ndjson: &str) -> Result<JournalV1, JournalError> {
    let mut lines = journal_ndjson
        .lines()
        .filter(|line| !line.trim().is_empty());
    let header_line = lines.next().map(str::to_string);
    let record_lines = lines.count();
    JournalV1::read_ndjson(BufReader::new(journal_ndjson.as_bytes())).map_err(|error| {
        let message = error.to_string();
        let reason = classify(&error, header_line.as_deref(), record_lines);
        JournalError { reason, message }
    })
}

fn header_serialize_error(error: &serde_json::Error) -> String {
    JournalError {
        reason: ReplayError::HeaderSerialize,
        message: error.to_string(),
    }
    .to_string()
}

/// Renders a finite `f64` as a JSON number, and anything else as `null` (never `NaN`/infinity).
fn number(x: f64) -> Value {
    serde_json::Number::from_f64(x).map_or(Value::Null, Value::Number)
}

fn side_str(side: OrderSide) -> &'static str {
    // `honba_core::OrderSide` is a closed `{Buy, Sell}` enum (no `NoOrderSide`
    // sentinel, unlike the legacy wire type this replaced), so this match is
    // already exhaustive.
    match side {
        OrderSide::Buy => "buy",
        OrderSide::Sell => "sell",
    }
}

fn trading_state_str(state: TradingState) -> &'static str {
    match state {
        TradingState::Active => "active",
        TradingState::Reducing => "reducing",
        TradingState::Halted => "halted",
    }
}

/// The final rejection reason per order id, as recorded by the replayed [`ReplayState`].
///
/// The engine exposes rejections only by lookup ([`ReplayState::rejection`]); the order ids come
/// from the log's own risk-refusal and pre-gate rejection records, so the map holds exactly the
/// rejected orders and no others. Later records overwrite earlier ones, matching the state.
fn rejection_reasons(state: &ReplayState, log: &AuditLog) -> BTreeMap<String, String> {
    let mut rejections = BTreeMap::new();
    for record in log.records() {
        let order_id = match &record.kind {
            AuditKind::RiskRefused { order_id, .. } | AuditKind::OrderRejected { order_id, .. } => {
                order_id
            }
            _ => continue,
        };
        if let Some(reason) = state.rejection(order_id) {
            rejections.insert(order_id.clone(), reason.to_string());
        }
    }
    rejections
}

/// Net position per instrument after the replay.
#[derive(Clone, Debug, PartialEq)]
pub struct PositionSummary {
    /// The instrument id, `SYMBOL.EXCHANGE`.
    pub instrument: String,
    /// The signed net quantity.
    pub quantity: f64,
}

/// One order's reconstructed state after the replay.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderSummary {
    /// The client order id.
    pub order_id: String,
    /// The instrument id, `SYMBOL.EXCHANGE`.
    pub instrument: String,
    /// The order side.
    pub side: OrderSide,
    /// The cumulative filled quantity.
    pub filled_qty: f64,
    /// The latest lifecycle event kind, if any.
    pub lifecycle: Option<OrderEventKind>,
    /// Whether a cancel was requested.
    pub cancel_requested: bool,
}

/// The reconstructed state of a replayed journal.
#[derive(Clone, Debug, PartialEq)]
pub struct ReplaySummary {
    /// The trading state at the end of the log.
    pub trading_state: TradingState,
    /// Number of dispatches recorded.
    pub dispatches: u64,
    /// Net positions, sorted by instrument.
    pub positions: Vec<PositionSummary>,
    /// Orders, sorted by order id.
    pub orders: Vec<OrderSummary>,
    /// Final rejection reason per rejected order id.
    pub rejections: BTreeMap<String, String>,
}

/// A replayed journal: its provenance header and the reconstructed state.
#[derive(Clone, Debug)]
pub struct JournalReplay {
    /// The provenance header.
    pub header: JournalHeader,
    /// The reconstructed state.
    pub replay: ReplaySummary,
}

impl JournalReplay {
    /// Renders the compact summary JSON described in the module docs.
    ///
    /// # Errors
    ///
    /// A `reason: message` pair when the header fails serialization.
    pub fn to_json(&self) -> Result<String, String> {
        let positions: Vec<Value> = self
            .replay
            .positions
            .iter()
            .map(|position| {
                json!({"instrument": position.instrument, "quantity": number(position.quantity)})
            })
            .collect();

        let orders: Vec<Value> = self
            .replay
            .orders
            .iter()
            .map(|order| {
                let lifecycle = match order.lifecycle {
                    Some(kind) => Value::String(kind.to_string()),
                    None => Value::Null,
                };
                json!({
                    "order_id": order.order_id,
                    "instrument": order.instrument,
                    "side": side_str(order.side),
                    "filled_qty": number(order.filled_qty),
                    "lifecycle": lifecycle,
                    "cancel_requested": order.cancel_requested,
                })
            })
            .collect();

        let rejections: serde_json::Map<String, Value> = self
            .replay
            .rejections
            .iter()
            .map(|(order_id, reason)| (order_id.clone(), Value::String(reason.clone())))
            .collect();

        let summary = json!({
            "trading_state": trading_state_str(self.replay.trading_state),
            "dispatches": self.replay.dispatches,
            "positions": positions,
            "orders": orders,
            "rejections": Value::Object(rejections),
        });

        let header = serde_json::to_value(&self.header)
            .map_err(|error| header_serialize_error(&error))?;
        let out = json!({
            "header": header,
            "replay": summary,
        });
        Ok(out.to_string())
    }
}

/// Parses the journal and returns its provenance header.
///
/// # Errors
///
/// A [`JournalError`] for an empty stream, a missing or malformed header, a malformed record
/// line, or an unsupported journal version.
pub fn journal_header(journal_ndjson: &str) -> Result<JournalHeader, JournalError> {
    Ok(parse_journal(journal_ndjson)?.header)
}

/// Replays the journal and returns the header plus the reconstructed state, with positions
/// sorted by instrument, orders sorted by order id and rejections keyed by order id.
///
/// # Errors
///
/// As [`journal_header`].
pub fn replay_journal(journal_ndjson: &str) -> Result<JournalReplay, JournalError> {
    let journal = parse_journal(journal_ndjson)?;
    let state = ReplayState::from_log(&journal.log);

    let positions: Vec<PositionSummary> = state
        .positions()
        .iter()
        .map(|(instrument, quantity)| (instrument.to_string(), *quantity))
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(instrument, quantity)| PositionSummary {
            instrument,
            quantity,
        })
        .collect();

    let orders: Vec<OrderSummary> = state
        .orders()
        .values()
        .map(|order| {
            (
                order.order_id.clone(),
                OrderSummary {
                    order_id: order.order_id.clone(),
                    instrument: order.instrument.to_string(),
                    side: order.side,
                    filled_qty: order.filled_qty,
                    lifecycle: order.lifecycle,
                    cancel_requested: order.cancel_requested,
                },
            )
        })
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect();

    let rejections = rejection_reasons(&state, &journal.log);

    Ok(JournalReplay {
        replay: ReplaySummary {
            trading_state: state.trading_state(),
            dispatches: state.dispatches(),
            positions,
            orders,
            rejections,
        },
        header: journal.header,
    })
}

/// Parses the journal and returns its header as JSON: the wire form of [`journal_header`].
///
/// # Errors
///
/// A `reason: message` pair for an empty stream, a missing or malformed header, an unsupported
/// journal version, or a header that fails serialization.
pub fn journal_header_json(journal_ndjson: &str) -> Result<String, String> {
    let header = journal_header(journal_ndjson).map_err(|error| error.to_string())?;
    serde_json::to_string(&header).map_err(|error| header_serialize_error(&error))
}

/// Replays the journal and returns the compact summary JSON described in the module docs: the
/// wire form of [`replay_journal`].
///
/// # Errors
///
/// A `reason: message` pair for an empty stream, a missing or malformed header, a malformed
/// record line, an unsupported journal version, or a header that fails serialization.
pub fn replay_journal_json(journal_ndjson: &str) -> Result<String, String> {
    replay_journal(journal_ndjson)
        .map_err(|error| error.to_string())?
        .to_json()
}

/// Parses the journal and returns the number of audit records after the header.
///
/// # Errors
///
/// A `reason: message` pair for an empty stream, a missing or malformed header, a malformed
/// record line, or an unsupported journal version.
pub fn journal_record_count(journal_ndjson: &str) -> Result<usize, String> {
    let journal = parse_journal(journal_ndjson).map_err(|error| error.to_string())?;
    Ok(journal.log.len())
}
