//! Universal symbol search — the port of the labs app's `core/symbol-search.ts`
//! over the bundled universe. The pool merges: market indices, the 304 rich
//! instruments and the 3,701 raw bhavcopy rows (~4,000 instruments).

use std::collections::HashMap;
use std::sync::OnceLock;

use super::bhavcopy_pool;
use crate::features::screener::data::indices::{self, is_index_symbol};
use crate::features::screener::data::instruments;

/// Results grouped by segment, like `getCategoryOf`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolCategory {
    All,
    Index,
    Cash,
    FuturesOptions,
    Commodity,
    Crypto,
}

impl SymbolCategory {
    pub const ALL: [SymbolCategory; 6] = [
        SymbolCategory::All,
        SymbolCategory::Index,
        SymbolCategory::Cash,
        SymbolCategory::FuturesOptions,
        SymbolCategory::Commodity,
        SymbolCategory::Crypto,
    ];

    pub const fn as_id(self) -> &'static str {
        match self {
            SymbolCategory::All => "all",
            SymbolCategory::Index => "index",
            SymbolCategory::Cash => "cash",
            SymbolCategory::FuturesOptions => "fo",
            SymbolCategory::Commodity => "commodity",
            SymbolCategory::Crypto => "crypto",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            SymbolCategory::All => "All",
            SymbolCategory::Index => "Index",
            SymbolCategory::Cash => "Cash",
            SymbolCategory::FuturesOptions => "F&O",
            SymbolCategory::Commodity => "Commodity",
            SymbolCategory::Crypto => "Crypto",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id {
            "index" => SymbolCategory::Index,
            "cash" => SymbolCategory::Cash,
            "fo" => SymbolCategory::FuturesOptions,
            "commodity" => SymbolCategory::Commodity,
            "crypto" => SymbolCategory::Crypto,
            _ => SymbolCategory::All,
        }
    }
}

/// The labs `SearchResultItem` equivalent, minus the dataset row it came from.
#[derive(Clone, Debug, PartialEq)]
pub struct SearchItem {
    pub symbol: String,
    pub name: String,
    pub exchange: String,
    pub category: SymbolCategory,
    /// Short type badge: EQ / SME / INDEX / ETF / MF / BOND / FUT / CE / PE / COMM / CRYPTO / REIT / INVIT.
    pub type_badge: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    /// Mega / large caps sort ahead on blank queries.
    pub is_priority: bool,
    pub rank_order: usize,
}

struct Pool {
    list: Vec<SearchItem>,
    by_symbol: HashMap<String, usize>,
}

fn category_of(exchange: &str, symbol: &str) -> SymbolCategory {
    let ex = exchange.to_uppercase();
    let sym = symbol.to_uppercase();

    if ex.ends_with("_INDEX")
        || ex == "INDEX"
        || matches!(
            sym.as_str(),
            "NIFTY" | "BANKNIFTY" | "SENSEX" | "FINNIFTY" | "MIDCPNIFTY" | "INDIAVIX"
        )
        || is_index_symbol(&sym)
    {
        SymbolCategory::Index
    } else if ex == "NFO" || ex == "BFO" || sym.ends_with("FUT") || ends_with_option_digits(&sym) {
        SymbolCategory::FuturesOptions
    } else if ex == "MCX"
        || ex == "NCDEX"
        || matches!(
            sym.as_str(),
            "GOLD"
                | "GOLDM"
                | "SILVER"
                | "SILVERM"
                | "CRUDEOIL"
                | "NATURALGAS"
                | "COPPER"
                | "ZINC"
                | "ALUMINIUM"
                | "LEAD"
        )
    {
        SymbolCategory::Commodity
    } else if ex == "CRYPTO"
        || sym.ends_with("USDT")
        || matches!(sym.as_str(), "BTCUSD" | "ETHUSD" | "SOLUSD" | "BTC" | "ETH")
    {
        SymbolCategory::Crypto
    } else {
        SymbolCategory::Cash
    }
}

fn ends_with_option_digits(symbol: &str) -> bool {
    let bytes = symbol.as_bytes();
    if bytes.len() < 3 {
        return false;
    }
    let (digits, tail) = bytes.split_at(bytes.len() - 2);
    (tail == b"CE" || tail == b"PE") && digits.iter().all(|b| b.is_ascii_digit())
}

fn type_badge(
    symbol: &str,
    exchange: &str,
    raw_badge: Option<&str>,
    asset_type: Option<&str>,
) -> String {
    let sym = symbol.to_uppercase();
    let ex = exchange.to_uppercase();

    if ex.ends_with("_INDEX")
        || ex == "INDEX"
        || matches!(
            sym.as_str(),
            "NIFTY" | "BANKNIFTY" | "SENSEX" | "FINNIFTY" | "MIDCPNIFTY" | "INDIAVIX"
        )
        || is_index_symbol(&sym)
        || asset_type == Some("index")
    {
        return "INDEX".to_string();
    }
    if let Some(badge) = raw_badge
        && !badge.is_empty()
    {
        return badge.to_string();
    }
    if asset_type == Some("etf") {
        return "ETF".to_string();
    }
    if asset_type == Some("mf") || ex == "AMFI" {
        return "MF".to_string();
    }
    if asset_type == Some("bonds") {
        return "BOND".to_string();
    }
    if sym.ends_with("CE") && ends_with_option_digits(&sym) {
        return "CE".to_string();
    }
    if sym.ends_with("PE") && ends_with_option_digits(&sym) {
        return "PE".to_string();
    }
    if sym.ends_with("FUT") {
        return "FUT".to_string();
    }
    if ex == "CRYPTO" {
        return "CRYPTO".to_string();
    }
    if ex == "MCX" || ex == "MCX" || ex == "NCDEX" {
        return "COMM".to_string();
    }
    "EQ".to_string()
}

/// The authoritative index prices from the labs app (`KNOWN_INDEX_PRICES`).
const KNOWN_INDEX_PRICES: &[(&str, f64)] = &[
    ("NIFTY", 24_850.0),
    ("SENSEX", 81_650.0),
    ("BANKNIFTY", 51_800.0),
    ("FINNIFTY", 23_600.0),
    ("MIDCPNIFTY", 12_950.0),
    ("INDIAVIX", 13.5),
    ("CNXFINANCE", 23_600.0),
    ("CNX500", 22_940.0),
    ("NIFTYJR", 73_420.0),
    ("CNXAUTO", 25_840.0),
    ("CNXPHARMA", 22_910.0),
    ("CNXPSUBANK", 6_780.0),
    ("CNXIT", 42_150.0),
    ("CNXMETAL", 9_640.0),
    ("CNXFMCG", 61_820.0),
    ("CNXREALTY", 1_045.0),
    ("NIFTYFINSRV25_50", 24_890.0),
    ("SPX", 5_840.0),
    ("NDX", 20_350.0),
    ("DJI", 43_050.0),
    ("RUT", 2_245.0),
    ("SOX", 5_280.0),
    ("XLK", 232.0),
    ("XLF", 47.2),
    ("XLE", 91.5),
    ("XLV", 152.8),
    ("N225", 39_600.0),
    ("TOPIX", 2_735.0),
    ("JPX400", 24_850.0),
    ("FTSE100", 8_290.0),
    ("FTSE250", 20_980.0),
    ("FTSEALL", 4_520.0),
];

fn known_index_price(symbol: &str) -> f64 {
    KNOWN_INDEX_PRICES
        .iter()
        .find(|(code, _)| *code == symbol)
        .map_or(20_000.0, |(_, price)| *price)
}

fn index_exchange_for(country: &str) -> &'static str {
    match country {
        "US" => "US_INDEX",
        "JP" => "TSE_INDEX",
        "UK" => "LSE_INDEX",
        _ => "NSE_INDEX",
    }
}

fn build_pool() -> Pool {
    let mut list: Vec<SearchItem> = Vec::new();
    let mut by_symbol: HashMap<String, usize> = HashMap::new();
    let mut order = 0usize;

    let push = |list: &mut Vec<SearchItem>,
                by_symbol: &mut HashMap<String, usize>,
                item: SearchItem,
                order: &mut usize| {
        let upper = item.symbol.trim().to_uppercase();
        if upper.is_empty() {
            return;
        }
        match by_symbol.get(&upper) {
            Some(&index) => list[index] = item,
            None => {
                let pos = list.len();
                by_symbol.insert(upper, pos);
                list.push(item);
                *order += 1;
            }
        }
    };

    // 1. The MARKET_INDICES registry across all four markets.
    for index in indices::MARKET_INDICES {
        let sym = index.code.trim().to_uppercase();
        let ex = if sym == "SENSEX" {
            "BSE_INDEX"
        } else {
            index_exchange_for(index.country)
        };
        let price = known_index_price(index.code);
        push(
            &mut list,
            &mut by_symbol,
            SearchItem {
                symbol: index.code.to_string(),
                name: index.name.to_string(),
                exchange: ex.to_string(),
                category: SymbolCategory::Index,
                type_badge: "INDEX".to_string(),
                price,
                change: (price * 0.0042 * 100.0).round() / 100.0,
                change_percent: 0.42,
                is_priority: true,
                rank_order: order,
            },
            &mut order,
        );
    }

    // 2. Rich instruments from the bundled universe (registry index entries win).
    for instrument in instruments::universe() {
        let sym = instrument.symbol.trim().to_uppercase();
        if sym.is_empty()
            || by_symbol
                .get(&sym)
                .map_or(false, |&pos| list[pos].category == SymbolCategory::Index)
        {
            continue;
        }
        let category = category_of(&instrument.exchange, &sym);
        let badge = type_badge(
            &sym,
            &instrument.exchange,
            None,
            Some(&instrument.asset_type),
        );
        let is_priority = category == SymbolCategory::Index
            || instrument.market_cap_tier == "mega"
            || instrument.market_cap_tier == "large";
        push(
            &mut list,
            &mut by_symbol,
            SearchItem {
                symbol: instrument.symbol.clone(),
                name: if instrument.name.is_empty() {
                    instrument.symbol.clone()
                } else {
                    instrument.name.clone()
                },
                exchange: instrument.exchange.clone(),
                category,
                type_badge: badge,
                price: instrument.price,
                change: instrument.change,
                change_percent: instrument.change_percent,
                is_priority,
                rank_order: order,
            },
            &mut order,
        );
    }

    // 3. The raw bhavcopy pool (missing names get filled, rich entries win).
    for row in bhavcopy_pool::pool() {
        let sym_upper = row.symbol.to_uppercase();
        if let Some(&pos) = by_symbol.get(&sym_upper) {
            let existing = &mut list[pos];
            let is_stub_name = existing.name.is_empty() || existing.name == existing.symbol;
            if is_stub_name && !row.name.is_empty() {
                existing.name = row.name.clone();
            }
            if existing.price <= 0.0 && row.price > 0.0 {
                existing.price = row.price;
                existing.change = row.change;
                existing.change_percent = row.change_percent;
            }
            continue;
        }
        let category = category_of(&row.exchange, &sym_upper);
        let badge = type_badge(&sym_upper, &row.exchange, Some(&row.badge), None);
        push(
            &mut list,
            &mut by_symbol,
            SearchItem {
                symbol: row.symbol.clone(),
                name: row.name.clone(),
                exchange: row.exchange.clone(),
                category,
                type_badge: badge,
                price: row.price,
                change: row.change,
                change_percent: row.change_percent,
                is_priority: rank_by_order(order) || category != SymbolCategory::Cash,
                rank_order: order,
            },
            &mut order,
        );
    }

    Pool { list, by_symbol }
}

fn rank_by_order(order: usize) -> bool {
    order < 120
}

static POOL: OnceLock<Pool> = OnceLock::new();

fn ensure_pool() -> &'static Pool {
    POOL.get_or_init(build_pool)
}

/// Deterministic quote for an arbitrary (custom) symbol — the labs
/// `resolveSymbolPrice` hash fallback.
fn hash_to_price(symbol: &str) -> f64 {
    let mut hash: i32 = 0;
    for byte in symbol.bytes() {
        hash = hash
            .wrapping_shl(5)
            .wrapping_sub(hash)
            .wrapping_add(i32::from(byte));
    }
    let base = 120 + (hash.abs() % 2400);
    (base as f64 * 100.0).round() / 100.0
}

/// Resolve the tradeable price for any symbol known to the pool.
pub fn resolve_symbol_price(symbol: &str) -> f64 {
    let clean = symbol
        .split(':')
        .last()
        .unwrap_or(symbol)
        .trim()
        .to_uppercase();
    if let Some(&pos) = ensure_pool().by_symbol.get(&clean) {
        return ensure_pool().list[pos].price;
    }
    hash_to_price(&clean)
}

/// Look up metadata for one pool symbol.
pub fn metadata(symbol: &str) -> Option<&'static SearchItem> {
    let clean = symbol
        .split(':')
        .last()
        .unwrap_or(symbol)
        .trim()
        .to_uppercase();
    ensure_pool()
        .by_symbol
        .get(&clean)
        .map(|&pos| &ensure_pool().list[pos])
}

/// The full search over the unified pool: exact > prefix > contains > tokens,
/// priority ahead, with a custom "Chart …" row for brand-new tickers.
pub fn search(query: &str, category: SymbolCategory, limit: usize) -> Vec<SearchItem> {
    let raw_query = query.trim();
    let q = raw_query.to_uppercase();
    let (exchange_filter, clean_q) = match q.split_once(':') {
        Some((exchange, rest)) => (exchange.trim().to_string(), rest.trim().to_string()),
        None => (String::new(), q.clone()),
    };

    let tokens: Vec<String> = clean_q.split_whitespace().map(str::to_string).collect();

    let mut matched: Vec<&SearchItem> = ensure_pool()
        .list
        .iter()
        .filter(|entry| {
            if category != SymbolCategory::All && entry.category != category {
                return false;
            }
            if !exchange_filter.is_empty()
                && !entry.exchange.to_uppercase().starts_with(&exchange_filter)
            {
                return false;
            }
            let exact = entry.symbol.to_uppercase() == clean_q || clean_q.is_empty() || {
                let sym = entry.symbol.to_uppercase();
                let name = entry.name.to_uppercase();
                sym.starts_with(&clean_q)
                    || name.starts_with(&clean_q)
                    || sym.contains(&clean_q)
                    || name.contains(&clean_q)
                    || (tokens.len() > 1
                        && tokens.iter().all(|token| {
                            entry.symbol.to_uppercase().contains(token)
                                || entry.name.to_uppercase().contains(token)
                        }))
            };
            exact
        })
        .collect();

    matched.sort_by(|a, b| {
        let rank = |entry: &SearchItem| {
            if clean_q.is_empty() {
                return if entry.category == SymbolCategory::Index {
                    0
                } else if entry.is_priority {
                    1
                } else {
                    5
                };
            }
            let sym = entry.symbol.to_uppercase();
            let name = entry.name.to_uppercase();
            if sym == clean_q {
                0
            } else if sym.starts_with(&clean_q) {
                1
            } else if name.starts_with(&clean_q) {
                2
            } else if sym.contains(&clean_q) {
                3
            } else if name.contains(&clean_q) {
                4
            } else {
                5
            }
        };
        rank(a)
            .cmp(&rank(b))
            .then_with(|| {
                if clean_q.is_empty() {
                    a.rank_order.cmp(&b.rank_order)
                } else if a.is_priority != b.is_priority {
                    if a.is_priority {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                } else {
                    a.symbol.cmp(&b.symbol)
                }
            })
            .then_with(|| {
                b.price
                    .partial_cmp(&a.price)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    matched.truncate(limit);
    let custom_item = if !clean_q.is_empty() && !ensure_pool().by_symbol.contains_key(&clean_q) {
        Some(custom_entry(raw_query))
    } else {
        None
    };

    let mut results: Vec<SearchItem> = Vec::new();
    if clean_q.is_empty() && custom_item.is_some() {
        results.extend(custom_item.clone());
    }
    results.extend(matched.into_iter().cloned());
    if !clean_q.is_empty() && custom_item.is_some() && results.is_empty() {
        // No exact or prefix match landed: surface the custom row anyway.
        results.extend(custom_item);
    }
    results
}

/// The "Chart \"{query}\" (Custom / Direct Symbol)" row for unknown tickers.
fn custom_entry(query: &str) -> SearchItem {
    let clean = query
        .split(':')
        .last()
        .unwrap_or(query)
        .trim()
        .to_uppercase();
    let ex = query
        .split(':')
        .next()
        .unwrap_or("NSE")
        .trim()
        .to_uppercase();
    let price = resolve_symbol_price(&clean);
    SearchItem {
        symbol: clean.clone(),
        name: format!("Chart \"{query}\" (Custom / Direct Symbol)"),
        exchange: ex,
        category: category_of(&String::new(), &clean),
        type_badge: "CUSTOM".to_string(),
        price,
        change: (price * 0.005 * 100.0).round() / 100.0,
        change_percent: 0.5,
        is_priority: false,
        rank_order: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_merges_all_three_sources() {
        let pool = ensure_pool();
        assert!(!pool.list.is_empty());
        // Indices registry entries win over raw rows.
        let nifty = metadata("NIFTY").unwrap();
        assert_eq!(nifty.name, "Nifty 50");
        assert_eq!(nifty.category, SymbolCategory::Index);
    }

    #[test]
    fn search_ranks_exact_and_prefix_ahead() {
        let results = search("RELI", SymbolCategory::All, 50);
        assert!(!results.is_empty());
        assert_eq!(results[0].symbol, "RELIANCE");
        // Custom symbol entry surfaces itself.
        let custom = search("ZZZQSTIT", SymbolCategory::All, 50);
        assert!(custom.iter().any(|item| item.type_badge == "CUSTOM"));
    }

    #[test]
    fn category_filter_narrows_the_pool() {
        let all = search("", SymbolCategory::All, 1_000_000);
        let indices = search("", SymbolCategory::Index, 1_000_000);
        assert!(indices.len() < all.len());
        assert!(
            indices
                .iter()
                .all(|item| item.category == SymbolCategory::Index)
        );
    }
}
