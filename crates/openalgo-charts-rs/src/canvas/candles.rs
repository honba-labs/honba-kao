//! Streaming candlestick chart renderer.
//!
//! Maintains a bounded ring of `CandleData` in Rust and pushes it to the
//! OpenAlgo Charts canvas engine. `push` appends a new candle; `update_last`
//! revises the most recent candle in place — the efficient path for live ticks.

use crate::canvas::engine::{CanvasEngine, SeriesKind, SeriesStyle};
use crate::types::CandleData;
use std::collections::VecDeque;
use wasm_bindgen::JsValue;

/// A live candlestick chart with a bounded candle buffer.
///
/// The buffer is a `VecDeque` (ring buffer) rather than a `Vec`: eviction of
/// the oldest candle at capacity is an O(1) `pop_front`, not an O(n) `remove(0)`.
pub struct StreamingCandleChart {
    engine: CanvasEngine,
    candles: VecDeque<CandleData>,
    max_candles: usize,
}

impl StreamingCandleChart {
    /// Create a chart bound to `canvas`, keeping at most `max_candles` candles.
    pub fn new(canvas: web_sys::HtmlCanvasElement, max_candles: usize) -> Self {
        Self {
            engine: CanvasEngine::new(
                canvas,
                SeriesKind::Candlestick,
                SeriesStyle::candlestick("#10b981", "#ef4444"),
            ),
            candles: VecDeque::new(),
            max_candles,
        }
    }

    /// Mount the chart on the canvas via the engine.
    pub fn mount(&mut self) -> Result<crate::canvas::engine::ChartId, JsValue> {
        self.engine.mount()
    }

    /// Chart id once mounted.
    pub fn id(&self) -> Option<crate::canvas::engine::ChartId> {
        self.engine.id()
    }

    /// Append a new candle, evicting the oldest when the buffer is full.
    pub fn push(&mut self, candle: CandleData) {
        if self.candles.len() >= self.max_candles {
            self.candles.pop_front();
        }
        self.candles.push_back(candle);
    }

    /// Replace the most recent candle in place (live tick revision). Appends
    /// when the buffer is empty.
    pub fn update_last(&mut self, candle: CandleData) {
        if let Some(last) = self.candles.back_mut() {
            *last = candle;
        } else {
            self.candles.push_back(candle);
        }
    }

    /// Replace the entire buffer (initial load or full resync).
    pub fn set_candles(&mut self, candles: Vec<CandleData>) {
        self.candles = VecDeque::from(candles);
        while self.candles.len() > self.max_candles {
            self.candles.pop_front();
        }
    }

    /// Number of candles currently buffered.
    pub fn len(&self) -> usize {
        self.candles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candles.is_empty()
    }

    /// The buffered candles (oldest first, most recent last), indexable like
    /// a slice via `VecDeque`'s `Index` impl.
    pub fn candles(&self) -> &VecDeque<CandleData> {
        &self.candles
    }

    /// Push the buffered candles to the engine via `setSeries`.
    pub fn commit(&self) -> Result<bool, JsValue> {
        let candles: Vec<CandleData> = self.candles.iter().cloned().collect();
        self.engine.set_series(&candles)
    }

    /// Stream only the last candle to the engine via `updateLast`.
    pub fn commit_last(&self) -> Result<bool, JsValue> {
        match self.candles.back() {
            Some(candle) => self.engine.update_last(candle),
            None => Ok(false),
        }
    }

    /// Attach a volume histogram overlay.
    pub fn add_volume_overlay(&self) -> Result<bool, JsValue> {
        self.engine.add_volume_overlay()
    }
}

impl std::fmt::Debug for StreamingCandleChart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamingCandleChart")
            .field("engine", &self.engine)
            .field("candles", &self.candles.len())
            .field("max_candles", &self.max_candles)
            .finish()
    }
}
