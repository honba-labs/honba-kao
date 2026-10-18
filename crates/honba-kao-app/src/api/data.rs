//! Market data API namespace. Mirrors `web/src/lib/api/data.ts`.

use crate::data::demo_data;
use crate::types::MarketQuote;

/// Snapshot of the instruments the terminal tracks.
pub fn get_market_quotes() -> Vec<MarketQuote> {
    demo_data::demo_quotes()
}

/// Health of each market-data feed, shown on the data page.
#[derive(Clone, Debug, PartialEq)]
pub struct FeedHealth {
    pub provider: &'static str,
    pub segment: &'static str,
    pub latency_ms: u32,
    pub healthy: bool,
}

pub fn get_feed_health() -> Vec<FeedHealth> {
    vec![
        FeedHealth {
            provider: "NSE (TBT)",
            segment: "Equities",
            latency_ms: 8,
            healthy: true,
        },
        FeedHealth {
            provider: "BSE (TBT)",
            segment: "Equities",
            latency_ms: 11,
            healthy: true,
        },
        FeedHealth {
            provider: "NSE (FO)",
            segment: "Derivatives",
            latency_ms: 14,
            healthy: true,
        },
        FeedHealth {
            provider: "News wire",
            segment: "Sentiment",
            latency_ms: 240,
            healthy: false,
        },
    ]
}
