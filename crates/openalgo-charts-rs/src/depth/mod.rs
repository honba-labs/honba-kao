//! L2 depth ladder & cumulative depth chart.
//!
//! Computes cumulative volume depth from bid/ask levels, exposes the mid-market
//! price, spread, and click-to-order dispatch (which level a price taps into).
//! Also emits engine-friendly series for a canvas cumulative-depth chart.

use crate::types::TimeSeriesPoint;

/// A single book level.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DepthLevel {
    pub price: f64,
    pub size: f64,
    pub cumulative_size: f64,
    pub order_count: u32,
}

/// Result of tapping a price on the depth ladder: the level to trade and side.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderDispatch {
    pub price: f64,
    pub size: f64,
    pub is_bid: bool, // true = hitting a bid (sell), false = lifting an ask (buy)
}

/// Cumulative depth ladder: bids and asks with running cumulative size.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DepthLadder {
    pub bids: Vec<DepthLevel>, // sorted descending by price
    pub asks: Vec<DepthLevel>, // sorted ascending by price
    pub mid: Option<f64>,
}

impl DepthLadder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bids(mut self, bids: Vec<DepthLevel>) -> Self {
        self.bids = bids;
        self
    }
    pub fn with_asks(mut self, asks: Vec<DepthLevel>) -> Self {
        self.asks = asks;
        self
    }
    pub fn with_mid(mut self, mid: f64) -> Self {
        self.mid = Some(mid);
        self
    }

    /// Recompute running cumulative size on both sides (bids highest-first,
    /// asks lowest-first) so the ladder is display-ready.
    pub fn compute_cumulative(mut self) -> Self {
        let mut run = 0.0;
        for level in self.bids.iter_mut() {
            run += level.size;
            level.cumulative_size = run;
        }
        run = 0.0;
        for level in self.asks.iter_mut() {
            run += level.size;
            level.cumulative_size = run;
        }
        self
    }

    /// Best bid (highest) and best ask (lowest).
    pub fn top_of_book(&self) -> (Option<f64>, Option<f64>) {
        let best_bid = self.bids.iter().max_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        let best_ask = self.asks.iter().min_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        (best_bid.map(|l| l.price), best_ask.map(|l| l.price))
    }

    /// Bid-ask spread in absolute terms; `None` when either side is empty.
    pub fn spread(&self) -> Option<f64> {
        let (bid, ask) = self.top_of_book();
        match (bid, ask) {
            (Some(b), Some(a)) if a > b => Some(a - b),
            _ => None,
        }
    }

    /// Mid-market price: explicit `mid`, else the top-of-book midpoint.
    pub fn mid_price(&self) -> Option<f64> {
        if let Some(m) = self.mid {
            return Some(m);
        }
        let (bid, ask) = self.top_of_book();
        match (bid, ask) {
            (Some(b), Some(a)) => Some((a + b) / 2.0),
            _ => None,
        }
    }

    /// Tap a price: returns the highest level on the matching side at or below
    /// the tapped price — bids when selling (hit bid), asks when buying (lift
    /// ask). Falls back to the top of book (best price on this side) when the
    /// tap lies outside the book; `None` when the side is empty.
    pub fn dispatch_at(&self, price: f64, is_bid: bool) -> Option<OrderDispatch> {
        let side = if is_bid { &self.bids } else { &self.asks };
        if side.is_empty() {
            return None;
        }
        let top_of_book = side.iter().max_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        let best = side
            .iter()
            .filter(|level| level.price <= price)
            .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        let chosen = best.or(top_of_book).unwrap();
        Some(OrderDispatch {
            price: chosen.price,
            size: chosen.size,
            is_bid,
        })
    }

    /// Bid side as canvas series: x = price (scaled *100), y = cumulative size.
    pub fn bid_depth_series(&self) -> Vec<TimeSeriesPoint> {
        self.bids
            .iter()
            .map(|l| TimeSeriesPoint {
                time: (l.price * 100.0).round() as i64,
                value: l.cumulative_size,
            })
            .collect()
    }

    /// Ask side as canvas series: x = price (scaled *100), y = cumulative size.
    pub fn ask_depth_series(&self) -> Vec<TimeSeriesPoint> {
        self.asks
            .iter()
            .map(|l| TimeSeriesPoint {
                time: (l.price * 100.0).round() as i64,
                value: l.cumulative_size,
            })
            .collect()
    }
}
