//! L2 depth ladder & cumulative depth chart.
//!
//! Computes cumulative volume depth from bid/ask levels, exposes the mid-market
//! price, spread, and click-to-order dispatch (which level a price taps into).
//! Also emits engine-friendly series for a canvas cumulative-depth chart.

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

use crate::types::TimeSeriesPoint;

/// A single book level.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DepthLevel {
    pub price: Decimal,
    pub size: Decimal,
    pub cumulative_size: Decimal,
    pub order_count: u32,
}

/// Result of tapping a price on the depth ladder: the level to trade and side.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderDispatch {
    pub price: Decimal,
    pub size: Decimal,
    pub is_bid: bool, // true = hitting a bid (sell), false = lifting an ask (buy)
}

/// Cumulative depth ladder: bids and asks with running cumulative size.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DepthLadder {
    pub bids: Vec<DepthLevel>, // sorted descending by price
    pub asks: Vec<DepthLevel>, // sorted ascending by price
    pub mid: Option<Decimal>,
}

impl DepthLadder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the bid side. Levels are sorted descending by price (best bid
    /// first) regardless of input order, so callers do not need to pre-sort.
    pub fn with_bids(mut self, mut bids: Vec<DepthLevel>) -> Self {
        bids.sort_by_key(|l| std::cmp::Reverse(l.price));
        self.bids = bids;
        self
    }

    /// Sets the ask side. Levels are sorted ascending by price (best ask
    /// first) regardless of input order, so callers do not need to pre-sort.
    pub fn with_asks(mut self, mut asks: Vec<DepthLevel>) -> Self {
        asks.sort_by_key(|l| l.price);
        self.asks = asks;
        self
    }
    pub fn with_mid(mut self, mid: Decimal) -> Self {
        self.mid = Some(mid);
        self
    }

    /// Recompute running cumulative size on both sides (bids highest-first,
    /// asks lowest-first) so the ladder is display-ready.
    pub fn compute_cumulative(mut self) -> Self {
        let mut run = Decimal::ZERO;
        for level in self.bids.iter_mut() {
            run += level.size;
            level.cumulative_size = run;
        }
        run = Decimal::ZERO;
        for level in self.asks.iter_mut() {
            run += level.size;
            level.cumulative_size = run;
        }
        self
    }

    /// Best bid (highest) and best ask (lowest).
    pub fn top_of_book(&self) -> (Option<Decimal>, Option<Decimal>) {
        let best_bid = self.bids.iter().max_by_key(|l| l.price);
        let best_ask = self.asks.iter().min_by_key(|l| l.price);
        (best_bid.map(|l| l.price), best_ask.map(|l| l.price))
    }

    /// Bid-ask spread in absolute terms; `None` when either side is empty.
    pub fn spread(&self) -> Option<Decimal> {
        let (bid, ask) = self.top_of_book();
        match (bid, ask) {
            (Some(b), Some(a)) if a > b => Some(a - b),
            _ => None,
        }
    }

    /// Mid-market price: explicit `mid`, else the top-of-book midpoint.
    pub fn mid_price(&self) -> Option<Decimal> {
        if let Some(m) = self.mid {
            return Some(m);
        }
        let (bid, ask) = self.top_of_book();
        match (bid, ask) {
            (Some(b), Some(a)) => Some((a + b) / Decimal::TWO),
            _ => None,
        }
    }

    /// Tap a price: returns the level nearest to the tapped price on the
    /// matching side (bids when selling / hit-bid, asks when buying / lift-ask),
    /// ties resolving to the lower price so the aggressor never gets the worst
    /// level on the side. Falls back to nothing only when the side is empty.
    /// (Previously this inverted ask execution and matched the worst ask.)
    pub fn dispatch_at(&self, price: Decimal, is_bid: bool) -> Option<OrderDispatch> {
        let side = if is_bid { &self.bids } else { &self.asks };
        if side.is_empty() {
            return None;
        }
        let chosen = side.iter().min_by(|a, b| {
            let da = if a.price > price { a.price - price } else { price - a.price };
            let db = if b.price > price { b.price - price } else { price - b.price };
            da.cmp(&db).then_with(|| a.price.cmp(&b.price))
        }).unwrap();
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
                time: (l.price * Decimal::ONE_HUNDRED)
                    .round()
                    .to_i64()
                    .unwrap_or_default(),
                value: l.cumulative_size.to_f64().unwrap_or_default(),
            })
            .collect()
    }

    /// Ask side as canvas series: x = price (scaled *100), y = cumulative size.
    pub fn ask_depth_series(&self) -> Vec<TimeSeriesPoint> {
        self.asks
            .iter()
            .map(|l| TimeSeriesPoint {
                time: (l.price * Decimal::ONE_HUNDRED)
                    .round()
                    .to_i64()
                    .unwrap_or_default(),
                value: l.cumulative_size.to_f64().unwrap_or_default(),
            })
            .collect()
    }
}
