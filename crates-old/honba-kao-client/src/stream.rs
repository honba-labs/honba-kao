//! Typed frames for the live market stream (`WS /ws/live`).
//!
//! Pure wire types and parsing. The browser socket lives in
//! `honba-kao-ui-core` (`hooks::use_market_socket`); the app wires the two
//! together. Prices and quantities travel as decimal **strings**, never `f64`
//! (zero-floats invariant): the UI parses them only for display.
//!
//! Provenance (`Live / Delayed / Cached / Synthetic`) rides on every quote and
//! depth frame so the UI can label where a number came from.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Where a value came from and how fresh it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quality {
    Live,
    Delayed,
    Cached,
    Synthetic,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub quality: Quality,
    /// Feed or broker name, e.g. `"kite"` or `"mock"`.
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// RFC 3339 timestamp of the underlying observation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepthLevel {
    pub price: String,
    pub quantity: String,
}

/// One frame from the live stream, tagged by `type` on the wire.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamFrame {
    Quote {
        symbol: String,
        ltp: String,
        change_pct: String,
        provenance: Provenance,
    },
    Trade {
        symbol: String,
        price: String,
        quantity: String,
        /// `"buy"` or `"sell"` when the venue reports aggressor side.
        #[serde(default)]
        side: Option<String>,
        ts: String,
    },
    Depth {
        symbol: String,
        bids: Vec<DepthLevel>,
        asks: Vec<DepthLevel>,
        provenance: Provenance,
    },
    Heartbeat {
        ts: String,
    },
}

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("malformed stream frame: {0}")]
    Malformed(#[from] serde_json::Error),
}

/// Parses one text frame from the socket.
pub fn parse_frame(text: &str) -> Result<StreamFrame, StreamError> {
    Ok(serde_json::from_str(text)?)
}

/// The subscribe request sent once the socket opens.
pub fn subscribe_message(symbols: &[String]) -> String {
    serde_json::json!({ "action": "subscribe", "symbols": symbols }).to_string()
}
