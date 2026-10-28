//! Streaming bar (OHLC) chart renderer.
//!
//! Mirrors [`StreamingCandleChart`](crate::canvas::candles::StreamingCandleChart)
//! but renders the engine's `bar` series, which draws OHLC bars rather than
//! candles — useful for instruments where candle bodies add noise (e.g. illiquid
//! tenors, yield curves plotted as price bars).

use crate::canvas::engine::{CanvasEngine, SeriesKind, SeriesStyle};
use crate::types::CandleData;
use wasm_bindgen::JsValue;

/// A live OHLC bar chart with a bounded bar buffer.
pub struct StreamingBarChart {
    engine: CanvasEngine,
    bars: Vec<CandleData>,
    max_bars: usize,
}

impl StreamingBarChart {
    /// Create a chart bound to `canvas`, keeping at most `max_bars` bars.
    pub fn new(canvas: web_sys::HtmlCanvasElement, max_bars: usize) -> Self {
        Self {
            engine: CanvasEngine::new(
                canvas,
                SeriesKind::Bar,
                SeriesStyle::candlestick("#10b981", "#ef4444"),
            ),
            bars: Vec::new(),
            max_bars,
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

    /// Append a new bar, evicting the oldest when the buffer is full.
    pub fn push(&mut self, bar: CandleData) {
        if self.bars.len() >= self.max_bars {
            self.bars.remove(0);
        }
        self.bars.push(bar);
    }

    /// Replace the most recent bar in place (live tick revision).
    pub fn update_last(&mut self, bar: CandleData) {
        if let Some(last) = self.bars.last_mut() {
            *last = bar;
        } else {
            self.bars.push(bar);
        }
    }

    /// Replace the entire buffer.
    pub fn set_bars(&mut self, bars: Vec<CandleData>) {
        self.bars = bars;
        if self.bars.len() > self.max_bars {
            let drop = self.bars.len() - self.max_bars;
            self.bars.drain(..drop);
        }
    }

    pub fn len(&self) -> usize {
        self.bars.len()
    }
    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }
    pub fn bars(&self) -> &[CandleData] {
        &self.bars
    }

    /// Push the buffered bars to the engine via `setSeries`.
    pub fn commit(&self) -> Result<bool, JsValue> {
        self.engine.set_series(&self.bars)
    }

    /// Stream only the last bar via `updateLast`.
    pub fn commit_last(&self) -> Result<bool, JsValue> {
        match self.bars.last() {
            Some(bar) => self.engine.update_last(bar),
            None => Ok(false),
        }
    }
}

impl std::fmt::Debug for StreamingBarChart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamingBarChart")
            .field("engine", &self.engine)
            .field("bars", &self.bars.len())
            .field("max_bars", &self.max_bars)
            .finish()
    }
}
