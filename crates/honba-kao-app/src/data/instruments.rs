//! The bundled instrument universe, ported from the labs frontend's
//! `core/market-data.ts` (`INSTRUMENTS_DATABASE`). The raw rows are extracted
//! to `data/instruments.json` by the labs repo's build tooling and parsed once
//! on first access.

use std::sync::OnceLock;

use serde::Deserialize;

/// One instrument row (camelCase JSON matches the labs dataset).
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Instrument {
    pub symbol: String,
    pub name: String,
    pub country: String,
    pub exchange: String,
    pub sector: String,
    pub industry: String,
    pub asset_type: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: f64,
    pub avg_volume_30d: f64,
    pub market_cap: f64,
    pub market_cap_tier: String,
    pub pe: f64,
    pub forward_pe: f64,
    pub pb: f64,
    pub eps: f64,
    pub dividend_yield: f64,
    pub high52: f64,
    pub low52: f64,
    pub rsi14: f64,
    pub sma20: f64,
    pub sma50: f64,
    pub sma200: f64,
    pub technical_rating: String,
    pub perf1w: f64,
    pub perf1m: f64,
    pub perf3m: f64,
    pub perf1y: f64,
    pub revenue_growth: f64,
    pub net_margin: f64,
    pub roce: f64,
    pub roe: f64,
    pub debt_to_equity: f64,
    pub sparkline: Vec<f64>,
}

impl Default for Instrument {
    fn default() -> Self {
        // Field-level `#[serde(default)]` feeds from this impl, so it must
        // not itself parse JSON (that would recurse forever on every row).
        Self {
            symbol: String::new(),
            name: String::new(),
            country: String::new(),
            exchange: String::new(),
            sector: String::new(),
            industry: String::new(),
            asset_type: String::new(),
            price: 0.0,
            change: 0.0,
            change_percent: 0.0,
            volume: 0.0,
            avg_volume_30d: 0.0,
            market_cap: 0.0,
            market_cap_tier: String::new(),
            pe: 0.0,
            forward_pe: 0.0,
            pb: 0.0,
            eps: 0.0,
            dividend_yield: 0.0,
            high52: 0.0,
            low52: 0.0,
            rsi14: 0.0,
            sma20: 0.0,
            sma50: 0.0,
            sma200: 0.0,
            technical_rating: String::new(),
            perf1w: 0.0,
            perf1m: 0.0,
            perf3m: 0.0,
            perf1y: 0.0,
            revenue_growth: 0.0,
            net_margin: 0.0,
            roce: 0.0,
            roe: 0.0,
            debt_to_equity: 0.0,
            sparkline: Vec::new(),
        }
    }
}

impl Instrument {
    /// Relative volume versus the 30-day average (the `relVol` column).
    pub fn rel_vol(&self) -> f64 {
        if self.avg_volume_30d > 0.0 {
            self.volume / self.avg_volume_30d
        } else {
            0.0
        }
    }

    /// Position within the 52-week range, 0..1 (the `range52` column).
    pub fn range52(&self) -> f64 {
        let span = self.high52 - self.low52;
        if span.abs() > f64::EPSILON {
            (self.price - self.low52) / span
        } else {
            0.0
        }
    }

    pub fn is_index(&self) -> bool {
        self.asset_type == "index"
    }
}

/// Parsed-universe cache.
static UNIVERSE: OnceLock<Vec<Instrument>> = OnceLock::new();

fn load() -> Vec<Instrument> {
    let raw = include_str!("../../data/instruments.json");
    serde_json::from_str(raw).expect("data/instruments.json parses")
}

/// The full bundled universe (304 instruments across IN/US/JP/UK).
pub fn universe() -> &'static [Instrument] {
    UNIVERSE.get_or_init(load)
}

/// All instruments for one country market.
pub fn universe_for_market(country: &str) -> Vec<&'static Instrument> {
    universe()
        .iter()
        .filter(|instrument| instrument.country == country)
        .collect()
}

/// Filter to one screener asset class (`stocks` / `etf` / `bonds` / `mf`).
/// Bonds and mutual funds live in their own tables; everything else
/// (`stocks`, `etf`, `index`) shows up in the stock screener like the labs
/// app's `data-layer` does.
pub fn universe_for_screener(country: &str, screener_type: &str) -> Vec<&'static Instrument> {
    let want = match screener_type {
        "etf" => vec!["etf"],
        "bonds" => vec!["bonds"],
        "mf" => vec!["mf"],
        _ => vec!["stocks"],
    };
    universe()
        .iter()
        .filter(|instrument| instrument.country == country && want.iter().any(|t| instrument.asset_type == *t))
        .collect()
}

/// Look up one instrument by symbol (case-insensitive).
pub fn by_symbol(symbol: &str) -> Option<&'static Instrument> {
    let upper = symbol.to_uppercase();
    universe()
        .iter()
        .find(|instrument| instrument.symbol.to_uppercase() == upper)
}

/// Symbol list search over the enum: matches symbol / name / sector.
pub fn matches_search(instrument: &Instrument, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    instrument.symbol.to_lowercase().contains(&q)
        || instrument.name.to_lowercase().contains(&q)
        || instrument.sector.to_lowercase().contains(&q)
}

/// Distinct sectors ordered by frequency (the sector filter dropdown).
pub fn sectors_for(country: &str) -> Vec<String> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for instrument in universe_for_market(country) {
        if instrument.is_index() || instrument.asset_type == "bonds" {
            continue;
        }
        match counts.iter_mut().find(|(sector, _)| *sector == instrument.sector) {
            Some((_, count)) => *count += 1,
            None => counts.push((instrument.sector.clone(), 1)),
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    counts.into_iter().map(|(sector, _)| sector).collect()
}

/// Distinct `industry` values under one sector (a `focus`-like dropdown).
pub fn industries_under(country: &str, sector: &str) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for instrument in universe_for_market(country) {
        if instrument.sector == sector && !seen.contains(&instrument.industry) {
            seen.push(instrument.industry.clone());
        }
    }
    seen.sort();
    seen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn universe_has_the_labs_rows() {
        assert_eq!(universe().len(), 304);
        assert!(by_symbol("RELIANCE").is_some());
        assert!(by_symbol("reliance").is_some(), "lookup is case-insensitive");
    }

    #[test]
    fn derived_metrics_match_the_labs_formulas() {
        let reliance = by_symbol("RELIANCE").unwrap();
        assert!((reliance.rel_vol() - reliance.volume / reliance.avg_volume_30d).abs() < 1e-9);
        let range = (reliance.price - reliance.low52) / (reliance.high52 - reliance.low52);
        assert!((reliance.range52() - range).abs() < 1e-9);
    }

    #[test]
    fn screener_filters_split_asset_classes() {
        let etf = universe_for_screener("IN", "etf");
        assert!(etf.iter().all(|instrument| instrument.asset_type == "etf"));
        let stocks = universe_for_screener("IN", "stocks");
        assert!(stocks.iter().all(|instrument| instrument.asset_type == "stocks"));
    }
}
