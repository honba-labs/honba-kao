//! MOCK symbol universe for the command bar.
//!
//! No backend symbol-search endpoint is wired into the command bar yet, so
//! suggestions and preview cards come from this static table. Prices are
//! decimal strings for display only; the table is replaced by the search
//! client when it lands.

/// One searchable instrument (mock).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MockSymbol {
    pub ticker: &'static str,
    pub name: &'static str,
    pub exchange: &'static str,
    pub asset_class: &'static str,
    /// Last traded price, decimal string.
    pub ltp: &'static str,
    /// Day change in percent, signed decimal string.
    pub change_pct: &'static str,
    /// Mock intraday closes for the sparkline.
    pub series: &'static [f64],
}

pub const MOCK_SYMBOLS: &[MockSymbol] = &[
    MockSymbol { ticker: "RELIANCE", name: "Reliance Industries", exchange: "NSE", asset_class: "Equity", ltp: "2982.50", change_pct: "1.4", series: &[2935.0, 2941.5, 2950.0, 2944.2, 2968.0, 2972.4, 2982.5] },
    MockSymbol { ticker: "TCS", name: "Tata Consultancy Services", exchange: "NSE", asset_class: "Equity", ltp: "4012.10", change_pct: "-0.3", series: &[4030.0, 4025.0, 4018.0, 4021.5, 4010.0, 4008.0, 4012.1] },
    MockSymbol { ticker: "INFY", name: "Infosys", exchange: "NSE", asset_class: "Equity", ltp: "1652.75", change_pct: "0.6", series: &[1640.0, 1644.0, 1638.5, 1648.0, 1650.0, 1649.0, 1652.75] },
    MockSymbol { ticker: "HDFCBANK", name: "HDFC Bank", exchange: "NSE", asset_class: "Equity", ltp: "1698.30", change_pct: "0.2", series: &[1690.0, 1694.0, 1692.0, 1696.0, 1699.0, 1697.0, 1698.3] },
    MockSymbol { ticker: "SBIN", name: "State Bank of India", exchange: "NSE", asset_class: "Equity", ltp: "812.40", change_pct: "-1.1", series: &[822.0, 820.5, 818.0, 815.0, 813.0, 811.0, 812.4] },
    MockSymbol { ticker: "ITC", name: "ITC Limited", exchange: "NSE", asset_class: "Equity", ltp: "438.95", change_pct: "0.9", series: &[432.0, 434.5, 435.0, 437.0, 436.0, 438.0, 438.95] },
    MockSymbol { ticker: "NIFTY", name: "NIFTY 50 Index", exchange: "NSE", asset_class: "Index", ltp: "24850.10", change_pct: "0.4", series: &[24700.0, 24740.0, 24765.0, 24810.0, 24802.0, 24840.0, 24850.1] },
    MockSymbol { ticker: "BANKNIFTY", name: "NIFTY Bank Index", exchange: "NSE", asset_class: "Index", ltp: "52100.35", change_pct: "0.2", series: &[51900.0, 52010.0, 52040.0, 52080.0, 52020.0, 52090.0, 52100.35] },
    MockSymbol { ticker: "AAPL", name: "Apple Inc.", exchange: "NASDAQ", asset_class: "Equity", ltp: "228.10", change_pct: "0.7", series: &[224.0, 225.5, 226.0, 227.2, 226.8, 227.6, 228.1] },
    MockSymbol { ticker: "BTCUSD", name: "Bitcoin / US Dollar", exchange: "CRYPTO", asset_class: "Crypto", ltp: "67120.00", change_pct: "-2.1", series: &[68500.0, 68200.0, 67900.0, 67450.0, 67300.0, 67010.0, 67120.0] },
];

/// Symbols matching `query` by fuzzy score on ticker or name, best first.
pub fn search(query: &str, limit: usize) -> Vec<MockSymbol> {
    use super::commanding::fuzzy_score;
    let query = query.trim();
    if query.is_empty() {
        return vec![];
    }
    let mut scored: Vec<(i32, MockSymbol)> = MOCK_SYMBOLS
        .iter()
        .map(|s| (fuzzy_score(s.ticker, query).max(fuzzy_score(s.name, query)), *s))
        .filter(|(score, _)| *score >= 0)
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.ticker.cmp(b.1.ticker)));
    scored.into_iter().take(limit).map(|(_, s)| s).collect()
}

/// The mock record for an exact ticker, if any.
pub fn lookup(ticker: &str) -> Option<MockSymbol> {
    MOCK_SYMBOLS
        .iter()
        .find(|s| s.ticker.eq_ignore_ascii_case(ticker))
        .copied()
}
