//! Curated index-benchmark catalog backing the screener's market-index filter.
//!
//! All members are curated subsets of the 30-row demo equity universe exposed by
//! [`super::fixtures::stock_universe`]; NIFTY 200 and NIFTY 500 cover that
//! universe entirely, while the NIFTY 50 / Next 50 / midcap / smallcap slices
//! partition it by market-cap rank.

/// Classification of an [`IndexBenchmark`]: broad-market vs sectoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexClass {
    /// Multi-sector benchmark spanning the whole market.
    Broad,
    /// Single-sector benchmark focused on one industry.
    Sectoral,
}

/// A static index benchmark: stable key, display label, class and members.
pub struct IndexBenchmark {
    /// Exact catalog key used in `FilterCriteria::market_index` (e.g. `NIFTY50`).
    pub key: &'static str,
    /// Human-readable display label (e.g. `NIFTY 50`).
    pub label: &'static str,
    /// Broad vs sectoral classification.
    pub class: IndexClass,
    /// Curated member symbols ordered by market-cap descending rank.
    pub members: &'static [&'static str],
}

/// Catalog of supported index benchmarks: 7 broad + 4 sectoral, in stable order.
///
/// `members` are curated subsets of the 30-row demo universe; NIFTY 200 and
/// NIFTY 500 cover that universe entirely, while NIFTY 50 / NIFTY Next 50 /
/// midcap / smallcap entries are rank-ordered slices of it.
pub const INDEX_CATALOG: &[IndexBenchmark] = &[
    IndexBenchmark {
        key: "NIFTY50",
        label: "NIFTY 50",
        class: IndexClass::Broad,
        members: &[
            "RELIANCE",
            "TCS",
            "HDFCBANK",
            "ICICIBANK",
            "INFY",
            "SBIN",
            "LICI",
            "ITC",
            "HINDUNILVR",
            "HCLTECH",
            "TATAMOTORS",
            "NTPC",
            "ONGC",
            "SUNPHARMA",
        ],
    },
    IndexBenchmark {
        key: "NIFTYNEXT50",
        label: "NIFTY Next 50",
        class: IndexClass::Broad,
        members: &[
            "WIPRO",
            "BEL",
            "TATASTEEL",
            "NMDC",
            "TECHM",
            "BPCL",
            "PFC",
            "TATAPOWER",
            "NATIONALUM",
        ],
    },
    IndexBenchmark {
        key: "NIFTY100",
        label: "NIFTY 100",
        class: IndexClass::Broad,
        members: &[
            "RELIANCE",
            "TCS",
            "HDFCBANK",
            "ICICIBANK",
            "INFY",
            "SBIN",
            "LICI",
            "ITC",
            "HINDUNILVR",
            "HCLTECH",
            "TATAMOTORS",
            "NTPC",
            "ONGC",
            "SUNPHARMA",
            "WIPRO",
            "BEL",
            "TATASTEEL",
            "NMDC",
            "TECHM",
            "BPCL",
            "PFC",
            "TATAPOWER",
            "NATIONALUM",
        ],
    },
    IndexBenchmark {
        key: "NIFTY200",
        label: "NIFTY 200",
        class: IndexClass::Broad,
        members: &[
            "RELIANCE",
            "TCS",
            "HDFCBANK",
            "ICICIBANK",
            "INFY",
            "SBIN",
            "LICI",
            "ITC",
            "HINDUNILVR",
            "HCLTECH",
            "TATAMOTORS",
            "NTPC",
            "ONGC",
            "SUNPHARMA",
            "WIPRO",
            "BEL",
            "TATASTEEL",
            "NMDC",
            "TECHM",
            "BPCL",
            "PFC",
            "TATAPOWER",
            "NATIONALUM",
            "AMBUJACEM",
            "ASHOKLEY",
            "UPL",
            "JSWSTEEL",
            "DLF",
            "MOTHERSON",
            "SAIL",
        ],
    },
    IndexBenchmark {
        key: "NIFTY500",
        label: "NIFTY 500",
        class: IndexClass::Broad,
        members: &[
            "RELIANCE",
            "TCS",
            "HDFCBANK",
            "ICICIBANK",
            "INFY",
            "SBIN",
            "LICI",
            "ITC",
            "HINDUNILVR",
            "HCLTECH",
            "TATAMOTORS",
            "NTPC",
            "ONGC",
            "SUNPHARMA",
            "WIPRO",
            "BEL",
            "TATASTEEL",
            "NMDC",
            "TECHM",
            "BPCL",
            "PFC",
            "TATAPOWER",
            "NATIONALUM",
            "AMBUJACEM",
            "ASHOKLEY",
            "UPL",
            "JSWSTEEL",
            "DLF",
            "MOTHERSON",
            "SAIL",
        ],
    },
    IndexBenchmark {
        key: "NIFTYMIDCAP150",
        label: "NIFTY Midcap 150",
        class: IndexClass::Broad,
        members: &[
            "BEL",
            "TATASTEEL",
            "NMDC",
            "TECHM",
            "BPCL",
            "PFC",
            "TATAPOWER",
            "NATIONALUM",
            "AMBUJACEM",
            "ASHOKLEY",
        ],
    },
    IndexBenchmark {
        key: "NIFTYSMALLCAP250",
        label: "NIFTY Smallcap 250",
        class: IndexClass::Broad,
        members: &["UPL", "JSWSTEEL", "DLF", "MOTHERSON", "SAIL"],
    },
    IndexBenchmark {
        key: "NIFTYBANK",
        label: "NIFTY Bank",
        class: IndexClass::Sectoral,
        members: &["HDFCBANK", "ICICIBANK", "SBIN"],
    },
    IndexBenchmark {
        key: "NIFTYIT",
        label: "NIFTY IT",
        class: IndexClass::Sectoral,
        members: &["TCS", "INFY", "WIPRO", "HCLTECH", "TECHM"],
    },
    IndexBenchmark {
        key: "NIFTYAUTO",
        label: "NIFTY Auto",
        class: IndexClass::Sectoral,
        members: &["TATAMOTORS", "MOTHERSON", "ASHOKLEY"],
    },
    IndexBenchmark {
        key: "NIFTYPHARMA",
        label: "NIFTY Pharma",
        class: IndexClass::Sectoral,
        members: &["SUNPHARMA"],
    },
];

/// Exact key lookup in [`INDEX_CATALOG`]; unknown keys return `None`.
pub fn benchmark_for(key: &str) -> Option<&'static IndexBenchmark> {
    INDEX_CATALOG.iter().find(|benchmark| benchmark.key == key)
}

/// Membership test: `index_contains("NIFTYBANK", "SBIN")` is `true`,
/// `index_contains("NIFTYBANK", "PFC")` is `false`; unknown keys are `false`.
pub fn index_contains(key: &str, symbol: &str) -> bool {
    benchmark_for(key).is_some_and(|benchmark| benchmark.members.contains(&symbol))
}

/// Member count for `key`; `0` for unknown keys.
pub fn member_count(key: &str) -> usize {
    benchmark_for(key).map_or(0, |benchmark| benchmark.members.len())
}
