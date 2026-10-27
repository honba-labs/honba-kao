//! Live quote feed for the chrome.
//!
//! [`QuoteBook`] is the pure state: it seeds labelled synthetic quotes, applies
//! live `quote` frames from `honba_kao_client::stream`, and reports the overall
//! provenance (`Live`, `Cached`, `Synthetic`). [`MarketFeed`] wraps it in
//! signals and opens `WS /api/ws/live` through `use_market_socket`.
//!
//! Fallback: until the first live frame arrives the UI shows synthetic quotes
//! from the mock table, and if the socket fails the feed stays on that data and
//! reports `Synthetic` (or `Cached`, if live quotes were received earlier). The
//! backend endpoint is not confirmed to exist yet, so this path is the normal
//! path in development.

use std::collections::BTreeMap;

use honba_kao_client::stream::{Quality, StreamFrame, parse_frame, subscribe_message};
use honba_kao_ui_core::trading::ProvenanceQuality;
use honba_kao_ui_core::{SocketState, use_market_socket};
use leptos::prelude::*;

use crate::shared::command::symbols::{MOCK_SYMBOLS, MockSymbol};
use crate::shared::constants::branding::API_BASE_URL;

/// One quote as the UI shows it. Prices stay decimal strings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuoteView {
    pub ltp: String,
    pub change_pct: String,
    pub quality: ProvenanceQuality,
    pub source: String,
}

fn map_quality(quality: Quality) -> ProvenanceQuality {
    match quality {
        Quality::Live => ProvenanceQuality::Live,
        Quality::Delayed => ProvenanceQuality::Delayed,
        Quality::Cached => ProvenanceQuality::Cached,
        Quality::Synthetic => ProvenanceQuality::Synthetic,
        Quality::Unavailable => ProvenanceQuality::Unavailable,
    }
}

/// Pure quote state. Mutated only through its methods.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QuoteBook {
    quotes: BTreeMap<String, QuoteView>,
    /// True once any live or delayed quote has been applied.
    received_live: bool,
}

impl QuoteBook {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fills the book with synthetic quotes for every mock symbol that has no
    /// quote yet. Existing (live) quotes are left alone.
    pub fn seed_synthetic(&mut self, symbols: &[MockSymbol]) {
        for s in symbols {
            self.quotes
                .entry(s.ticker.to_string())
                .or_insert_with(|| QuoteView {
                    ltp: s.ltp.to_string(),
                    change_pct: s.change_pct.to_string(),
                    quality: ProvenanceQuality::Synthetic,
                    source: "mock".into(),
                });
        }
    }

    /// Applies a frame. Returns `true` when a quote changed.
    pub fn apply_frame(&mut self, frame: &StreamFrame) -> bool {
        let StreamFrame::Quote {
            symbol,
            ltp,
            change_pct,
            provenance,
        } = frame
        else {
            return false;
        };
        let quality = map_quality(provenance.quality);
        if matches!(
            quality,
            ProvenanceQuality::Live | ProvenanceQuality::Delayed
        ) {
            self.received_live = true;
        }
        let next = QuoteView {
            ltp: ltp.clone(),
            change_pct: change_pct.clone(),
            quality,
            source: provenance.source.clone(),
        };
        if self.quotes.get(symbol) == Some(&next) {
            return false;
        }
        self.quotes.insert(symbol.clone(), next);
        true
    }

    /// The socket is gone: live quotes become `Cached`, synthetic ones stay
    /// `Synthetic`.
    pub fn mark_offline(&mut self) {
        for quote in self.quotes.values_mut() {
            if matches!(
                quote.quality,
                ProvenanceQuality::Live | ProvenanceQuality::Delayed
            ) {
                quote.quality = ProvenanceQuality::Cached;
            }
        }
    }

    pub fn quote(&self, symbol: &str) -> Option<&QuoteView> {
        self.quotes.get(symbol)
    }

    /// Feed-level provenance for the chip: the best quality present.
    pub fn overall_quality(&self) -> ProvenanceQuality {
        let has = |q: ProvenanceQuality| self.quotes.values().any(|v| v.quality == q);
        if has(ProvenanceQuality::Live) {
            ProvenanceQuality::Live
        } else if has(ProvenanceQuality::Delayed) {
            ProvenanceQuality::Delayed
        } else if self.received_live {
            ProvenanceQuality::Cached
        } else {
            ProvenanceQuality::Synthetic
        }
    }
}

/// `ws(s)://<host>/api/ws/live`, built from the page location.
pub fn live_socket_url() -> String {
    let location = web_sys::window().map(|w| w.location());
    let protocol = location
        .as_ref()
        .and_then(|l| l.protocol().ok())
        .unwrap_or_else(|| "http:".into());
    let host = location
        .as_ref()
        .and_then(|l| l.host().ok())
        .unwrap_or_default();
    let scheme = if protocol == "https:" { "wss:" } else { "ws:" };
    format!("{scheme}//{host}{API_BASE_URL}/ws/live")
}

/// Reactive handle to the feed. `Copy`; provided by the chrome.
#[derive(Clone, Copy)]
pub struct MarketFeed {
    book: RwSignal<QuoteBook>,
    connection: RwSignal<SocketState>,
}

impl MarketFeed {
    /// The quote for `symbol`, reactive.
    pub fn quote(&self, symbol: &str) -> Signal<Option<QuoteView>> {
        let symbol = symbol.to_string();
        let book = self.book;
        Signal::derive(move || book.with(|b| b.quote(&symbol).cloned()))
    }

    /// The current quote for `symbol`, read reactively inside a tracked scope.
    pub fn snapshot(&self, symbol: &str) -> Option<QuoteView> {
        self.book.with(|b| b.quote(symbol).cloned())
    }

    /// Feed-level provenance, reactive.
    pub fn quality(&self) -> Signal<ProvenanceQuality> {
        let book = self.book;
        Signal::derive(move || book.with(|b| b.overall_quality()))
    }

    /// Socket state, reactive.
    pub fn connection(&self) -> Signal<SocketState> {
        let connection = self.connection;
        Signal::derive(move || connection.get())
    }

    pub fn provide(self) {
        provide_context(self);
    }
}

/// Starts the feed for the mock universe and returns its handle. Call once,
/// from inside the chrome component, so the socket lives as long as the chrome.
pub fn start_market_feed() -> MarketFeed {
    let book = RwSignal::new({
        let mut b = QuoteBook::new();
        b.seed_synthetic(MOCK_SYMBOLS);
        b
    });
    let connection = RwSignal::new(SocketState::Connecting);
    let feed = MarketFeed { book, connection };

    let symbols: Vec<String> = MOCK_SYMBOLS.iter().map(|s| s.ticker.to_string()).collect();
    let subscribe = subscribe_message(&symbols);

    let on_text = Callback::new(move |text: String| {
        if let Ok(frame) = parse_frame(&text) {
            book.update(|b| {
                b.apply_frame(&frame);
            });
        }
    });
    let on_state = Callback::new(move |state: SocketState| {
        connection.set(state);
        if matches!(state, SocketState::Closed | SocketState::Failed) {
            book.update(|b| b.mark_offline());
        }
    });

    use_market_socket(live_socket_url(), Some(subscribe), on_text, on_state);
    feed
}
