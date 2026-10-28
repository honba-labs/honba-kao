//! Streaming candlestick chart renderer.
//!
//! Maintains a bounded ring of `CandleData` in Rust and pushes it to the
//! OpenAlgo Charts canvas engine. `push` appends a new candle; `update_last`
//! revises the most recent candle in place — the efficient path for live ticks.

use crate::canvas::engine::{CanvasEngine, SeriesKind, SeriesStyle};
use crate::types::CandleData;
use wasm_bindgen::JsValue;

/// A live candlestick chart with a bounded candle buffer.
pub struct StreamingCandleChart {
    engine: CanvasEngine,
    candles: Vec<CandleData>,
    max_candles: usize,
}

impl StreamingCandleChart {
    /// Create a chart bound to `canvas`, keeping at most `max_candles` candles.
    pub fn new(
        canvas: web_sys::HtmlCanvasElement,
        max_candles: usize,
    ) -> Self {
        Self {
            engine: CanvasEngine::new(
                canvas,
                SeriesKind::Candlestick,
                SeriesStyle::candlestick("#10b981", "#ef4444"),
            ),
            candles: Vec::new(),
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
            self.candles.remove(0);
        }
        self.candles.push(candle);
    }

    /// Replace the most recent candle in place (live tick revision). Appends
    /// when the buffer is empty.
    pub fn update_last(&mut self, candle: CandleData) {
        if let Some(last) = self.candles.last_mut() {
            *last = candle;
        } else {
            self.candles.push(candle);
        }
    }

    /// Replace the entire buffer (initial load or full resync).
    pub fn set_candles(&mut self, candles: Vec<CandleData>) {
        self.candles = candles;
        if self.candles.len() > self.max_candles {
            let drop = self.candles.len() - self.max_candles;
            self.candles.drain(..drop);
        }
    }

    /// Number of candles currently buffered.
    pub fn len(&self) -> usize {
        self.candles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candles.is_empty()
    }

    /// Slice of buffered candles (most recent last).
    pub fn candles(&self) -> &[CandleData] {
        &self.candles
    }

    /// Push the buffered candles to the engine via `setSeries`.
    pub fn commit(&self) -> Result<bool, JsValue> {
        self.engine.set_series(&self.candles)
    }

    /// Stream only the last candle to the engine via `updateLast`.
    pub fn commit_last(&self) -> Result<bool, JsValue> {
        match self.candles.last() {
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
