//! Screener flyout tests: index catalog membership, prompt parsing/labels, and
//! the `market_index` filter criteria (red-first target for the parallel
//! `types::FilterCriteria::market_index` change).

use std::collections::HashSet;

use honba_kao_app::features::screener::{filters, fixtures, indices, prompt_ast, types};
use rust_decimal_macros::dec;

/// Market-cap-descending golden order of the 30-row demo universe.
const CAP_DESC: [&str; 30] = [
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
];

/// Symbol set of the 30-row stock fixture universe.
fn universe_symbols() -> HashSet<String> {
    fixtures::stock_universe()
        .iter()
        .map(|record| record.symbol.clone())
        .collect()
}

/// Identity strings of filtered universe records, preserving input order.
fn symbol_strings(records: &[types::ScreenerRecord]) -> Vec<String> {
    records
        .iter()
        .map(|record| record.symbol().to_string())
        .collect()
}

#[test]
fn index_catalog_shape() {
    let catalog = indices::INDEX_CATALOG;
    assert_eq!(catalog.len(), 11);
    let broad = catalog
        .iter()
        .filter(|benchmark| matches!(benchmark.class, indices::IndexClass::Broad))
        .count();
    let sectoral = catalog
        .iter()
        .filter(|benchmark| matches!(benchmark.class, indices::IndexClass::Sectoral))
        .count();
    assert_eq!(broad, 7);
    assert_eq!(sectoral, 4);
    let keys: Vec<&str> = catalog.iter().map(|benchmark| benchmark.key).collect();
    let unique: HashSet<&str> = keys.iter().copied().collect();
    assert_eq!(unique.len(), keys.len());
    assert!(catalog.iter().all(|benchmark| !benchmark.label.is_empty()));
}

#[test]
fn index_members_are_universe_symbols() {
    let universe = universe_symbols();
    for benchmark in indices::INDEX_CATALOG {
        for member in benchmark.members {
            assert!(
                universe.contains(*member),
                "{member} missing from stock_universe"
            );
        }
    }
}

#[test]
fn nifty50_is_cap_desc_prefix() {
    let nifty50 = indices::benchmark_for("NIFTY50").expect("NIFTY50 catalog entry");
    assert_eq!(nifty50.members, &CAP_DESC[..14]);
    let next50 = indices::benchmark_for("NIFTYNEXT50").expect("NIFTYNEXT50 catalog entry");
    assert_eq!(next50.members, &CAP_DESC[14..23]);
}

#[test]
fn nifty500_and_200_cover_universe() {
    let universe = universe_symbols();
    for key in ["NIFTY200", "NIFTY500"] {
        let benchmark = indices::benchmark_for(key).expect("catalog entry");
        assert_eq!(benchmark.members.len(), 30);
        let members: HashSet<String> = benchmark
            .members
            .iter()
            .map(|symbol| (*symbol).to_string())
            .collect();
        assert_eq!(members, universe);
    }
}

#[test]
fn midcap_smallcap_split_mid_ranks() {
    let midcap = indices::benchmark_for("NIFTYMIDCAP150").expect("NIFTYMIDCAP150 entry");
    assert_eq!(midcap.members, &CAP_DESC[15..25]);
    let smallcap = indices::benchmark_for("NIFTYSMALLCAP250").expect("NIFTYSMALLCAP250 entry");
    assert_eq!(smallcap.members, &CAP_DESC[25..30]);
}

#[test]
fn benchmark_lookup_and_membership() {
    let bank = indices::benchmark_for("NIFTYBANK").expect("NIFTYBANK entry");
    assert_eq!(bank.label, "NIFTY Bank");
    assert!(indices::benchmark_for("NOPE").is_none());
    assert!(indices::index_contains("NIFTYBANK", "SBIN"));
    assert!(!indices::index_contains("NIFTYBANK", "PFC"));
    assert!(!indices::index_contains("NOPE", "TCS"));
    assert_eq!(indices::member_count("NIFTYIT"), 5);
    assert_eq!(indices::member_count("NOPE"), 0);
}

#[test]
fn parse_bounds_and_ranges() {
    let ast = prompt_ast::parse_prompt("pe < 25, rvol > 2, price 250-600");
    assert_eq!(
        ast.terms,
        vec![
            prompt_ast::PromptTerm::PeMax(dec!(25)),
            prompt_ast::PromptTerm::RvolMin(dec!(2)),
            prompt_ast::PromptTerm::PriceMin(dec!(250)),
            prompt_ast::PromptTerm::PriceMax(dec!(600)),
        ]
    );
}

#[test]
fn parse_between_form() {
    assert_eq!(
        prompt_ast::parse_prompt("price between 250 and 600").terms,
        vec![
            prompt_ast::PromptTerm::PriceMin(dec!(250)),
            prompt_ast::PromptTerm::PriceMax(dec!(600)),
        ]
    );
    assert_eq!(
        prompt_ast::parse_prompt("rsi 55 to 70").terms,
        vec![
            prompt_ast::PromptTerm::RsiMin(dec!(55)),
            prompt_ast::PromptTerm::RsiMax(dec!(70)),
        ]
    );
}

#[test]
fn parse_upper_bounds_and_macros() {
    assert_eq!(
        prompt_ast::parse_prompt("pe <= 18.5").terms,
        vec![prompt_ast::PromptTerm::PeMax(dec!(18.5))]
    );
    assert_eq!(
        prompt_ast::parse_prompt("rsi below 30").terms,
        vec![prompt_ast::PromptTerm::RsiMax(dec!(30))]
    );
    assert_eq!(
        prompt_ast::parse_prompt("rvol > 2.1x").terms,
        vec![prompt_ast::PromptTerm::RvolMin(dec!(2.1))]
    );
}

#[test]
fn parse_index_terms() {
    assert_eq!(
        prompt_ast::parse_prompt("nifty50").terms,
        vec![prompt_ast::PromptTerm::Index("NIFTY50")]
    );
    assert_eq!(
        prompt_ast::parse_prompt("nifty bank").terms,
        vec![prompt_ast::PromptTerm::Index("NIFTYBANK")]
    );
    assert_eq!(
        prompt_ast::parse_prompt("nifty-bank").terms,
        vec![prompt_ast::PromptTerm::Index("NIFTYBANK")]
    );
    assert_eq!(
        prompt_ast::parse_prompt("MIDCAP150").terms,
        vec![prompt_ast::PromptTerm::Index("NIFTYMIDCAP150")]
    );
    assert_eq!(
        prompt_ast::parse_prompt("nifty 50").terms,
        vec![prompt_ast::PromptTerm::Index("NIFTY50")]
    );
    assert_eq!(
        prompt_ast::parse_prompt("banknifty").terms,
        vec![prompt_ast::PromptTerm::Index("NIFTYBANK")]
    );
}

#[test]
fn parse_bool_and_residual_fallback() {
    assert_eq!(
        prompt_ast::parse_prompt("fno").terms,
        vec![prompt_ast::PromptTerm::FnoOnly]
    );
    assert_eq!(
        prompt_ast::parse_prompt("watchlist only").terms,
        vec![prompt_ast::PromptTerm::WatchlistOnly]
    );
    let fallback = prompt_ast::parse_prompt("just some words");
    assert!(fallback.terms.is_empty());
    assert_eq!(fallback.residual, "just some words");
    // Greedy: a complete numeric assignment consumes the clause; trailing junk
    // ("and moon") is silently dropped, so the residual stays empty for it.
    let greedy = prompt_ast::parse_prompt("pe < 25 and moon");
    assert_eq!(greedy.terms, vec![prompt_ast::PromptTerm::PeMax(dec!(25))]);
    assert_eq!(greedy.residual, "");
}

#[test]
fn apply_prompt_multi_term() {
    let criteria = prompt_ast::apply_prompt("pe < 20, rsi > 60, nifty bank");
    assert_eq!(
        criteria,
        types::FilterCriteria {
            market_index: "NIFTYBANK".into(),
            max_pe: dec!(20),
            min_rsi: dec!(60),
            ..Default::default()
        }
    );
}

#[test]
fn apply_prompt_search_fallback() {
    let fallback = prompt_ast::apply_prompt("coal companies");
    assert_eq!(fallback.search_query, "coal companies");
    assert_eq!(fallback.min_price, dec!(0));
    assert_eq!(fallback.max_price, dec!(0));
    assert_eq!(fallback.min_pe, dec!(0));
    assert_eq!(fallback.max_pe, dec!(0));
    assert_eq!(fallback.min_rvol, dec!(0));
    assert_eq!(fallback.min_rsi, dec!(0));
    assert_eq!(fallback.max_rsi, dec!(0));
    assert_eq!(fallback.market_index, "");
    assert!(!fallback.fno_only);
    assert!(!fallback.watchlist_only);

    let mixed = prompt_ast::apply_prompt("pe < 25, coal");
    assert_eq!(mixed.search_query, "");
    assert_eq!(mixed.max_pe, dec!(25));
}

#[test]
fn prompt_label_format() {
    assert_eq!(
        prompt_ast::prompt_label("pe < 25, rvol > 2, nifty bank"),
        "PE<25 · RVOL>2 · NIFTY Bank"
    );
    assert_eq!(prompt_ast::prompt_label(""), "");
    assert_eq!(prompt_ast::prompt_label("coal"), "Search: coal");
}

#[test]
fn market_index_filter_via_matches() {
    let universe = fixtures::universe_for(types::ScreenerAssetType::Stocks);
    let bank = types::FilterCriteria {
        market_index: "NIFTYBANK".into(),
        ..Default::default()
    };
    assert_eq!(
        symbol_strings(&filters::filter_universe(&universe, &bank)),
        ["HDFCBANK", "ICICIBANK", "SBIN"]
    );

    // "ALL" and "" both mean "no restriction" (unset semantics).
    let all = types::FilterCriteria {
        market_index: "ALL".into(),
        ..Default::default()
    };
    assert_eq!(filters::filter_universe(&universe, &all).len(), 30);
    let empty = types::FilterCriteria::default();
    assert_eq!(filters::filter_universe(&universe, &empty).len(), 30);

    // Unknown keys are treated as unset: only exact catalog keys restrict.
    let unknown = types::FilterCriteria {
        market_index: "NOPE".into(),
        ..Default::default()
    };
    assert_eq!(filters::filter_universe(&universe, &unknown).len(), 30);
}

#[test]
fn market_index_active_count() {
    let bank = types::FilterCriteria {
        market_index: "NIFTYBANK".into(),
        ..Default::default()
    };
    assert_eq!(bank.active_count(), 1);
    let all = types::FilterCriteria {
        market_index: "ALL".into(),
        ..Default::default()
    };
    assert_eq!(all.active_count(), 0);
}
