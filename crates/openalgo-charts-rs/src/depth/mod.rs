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

    /// Tap a price: returns the nearest level on the matching side that can
    /// fill the tap — bids when selling (hit bid), asks when buying (lift ask).
    /// Dispatches to the nearest level at-or-beyond the tapped price.
    pub fn dispatch_at(&self, _price: f64, is_bid: bool) -> Option<OrderDispatch> {
        let side = if is_bid { &self.bids } else { &self.asks };
        if side.is_empty() {
            return None;
        }
        let best = if is_bid {
            // Hitting a bid: the best (highest) bid at or below the tapped price
            // is the most aggressive price the buyer will accept; tap the top.
            side.iter().max_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
        } else {
            // Lifting an ask: the best (lowest) ask at or above the tapped price.
            side.iter().min_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
        };
        best.map(|l| OrderDispatch {
            price: l.price,
            size: l.size,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn lvl(price: f64, size: f64, count: u32) -> DepthLevel {
        DepthLevel { price, size, cumulative_size: 0.0, order_count: count }
    }

    #[test]
    fn cumulative_and_top_of_book() {
        let ladder = DepthLadder::new()
            .with_bids(vec![lvl(99.0, 10.0, 1), lvl(98.0, 20.0, 2), lvl(97.0, 30.0, 3)])
            .with_asks(vec![lvl(101.0, 5.0, 1), lvl(102.0, 15.0, 2)])
            .with_mid(100.0)
            .compute_cumulative();
        assert_eq!(ladder.bids[0].cumulative_size, 10.0);
        assert_eq!(ladder.bids[1].cumulative_size, 30.0);
        assert_eq!(ladder.bids[2].cumulative_size, 60.0);
        assert_eq!(ladder.asks[0].cumulative_size, 5.0);
        assert_eq!(ladder.asks[1].cumulative_size, 20.0);
        assert_eq!(ladder.top_of_book(), (Some(99.0), Some(101.0)));
        assert!((ladder.spread().unwrap() - 2.0).abs() < f64::EPSILON);
        assert!((ladder.mid_price().unwrap() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mid_price_falls_back_to_top_of_book() {
        let ladder = DepthLadder::new()
            .with_bids(vec![lvl(99.0, 10.0, 1)])
            .with_asks(vec![lvl(101.0, 5.0, 1)])
            .compute_cumulative();
        assert!((ladder.mid_price().unwrap() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn dispatch_hits_best_bid_and_asks() {
        let ladder = DepthLadder::new()
            .with_bids(vec![lvl(99.0, 10.0, 1), lvl(98.0, 20.0, 2)])
            .with_asks(vec![lvl(101.0, 5.0, 1), lvl(102.0, 15.0, 2)])
            .compute_cumulative();
        let hit = ladder.dispatch_at(98.5, true).unwrap();
        assert!(hit.is_bid);
        assert!((hit.price - 99.0).abs() < f64::EPSILON);
        let lift = ladder.dispatch_at(101.5, false).unwrap();
        assert!(!lift.is_bid);
        assert!((lift.price - 101.0).abs() < f64::EPSILON);
        assert!(ladder.dispatch_at(50.0, true).is_some()); // still top of book
        let empty: DepthLadder = DepthLadder::new();
        assert!(empty.dispatch_at(100.0, true).is_none());
    }

    #[test]
    fn depth_series_are_scaled() {
        let ladder = DepthLadder::new()
            .with_bids(vec![lvl(99.5, 10.0, 1)])
            .with_asks(vec![lvl(101.25, 5.0, 1)])
            .compute_cumulative();
        let bids = ladder.bid_depth_series();
        assert_eq!(bids[0].time, 9950); // 99.5 * 100
        assert!((bids[0].value - 10.0).abs() < f64::EPSILON);
        let asks = ladder.ask_depth_series();
        assert_eq!(asks[0].time, 10125);
    }
}
