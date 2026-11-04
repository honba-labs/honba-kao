use honba_kao_app::features::screener::{filters, fixtures, types};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn symbols(records: &[types::ScreenerEquityRecord]) -> Vec<&str> {
    records.iter().map(|r| r.symbol.as_str()).collect()
}

fn fake_row(symbol: &str, name: &str) -> types::ScreenerEquityRecord {
    fixtures::equity_fixture(
        symbol,
        name,
        "NSE",
        "Industrials",
        "Diversified",
        "Small Cap",
        false,
        false,
        dec!(100),
        dec!(0),
        1000,
        1000,
        dec!(0),
        dec!(0),
        dec!(0),
        dec!(1000),
        dec!(99),
        dec!(101),
        dec!(90),
        dec!(110),
        dec!(50),
    )
}

#[test]
fn seed_fixtures_deterministic_and_complete() {
    let a = fixtures::seed_fixtures();
    let b = fixtures::seed_fixtures();
    assert_eq!(a, b);
    assert_eq!(a.len(), 12);
    let mut seen = std::collections::HashSet::new();
    for record in &a {
        assert!(
            seen.insert(record.symbol.clone()),
            "duplicate symbol {}",
            record.symbol
        );
        assert!(record.price > Decimal::ZERO);
    }
}

#[test]
fn fixture_derived_metrics_match_definition() {
    let rows = fixtures::seed_fixtures();
    let r = rows
        .iter()
        .find(|row| row.symbol == "RELIANCE")
        .expect("RELIANCE fixture");
    assert_eq!(r.rvol, dec!(2.10));
    assert_eq!(r.turnover_cr, dec!(1193.85));
    assert_eq!(r.change.round_dp(6), dec!(53.154750));
    assert_eq!(r.return_1d, r.change_pct);
}

#[test]
fn filter_by_search_sector_cap_and_flags() {
    let seed = fixtures::seed_fixtures();

    let criteria = types::FilterCriteria {
        search_query: "bank".to_string(),
        ..Default::default()
    };
    assert_eq!(
        symbols(&filters::filter_records(&seed, &criteria)),
        ["HDFCBANK", "ICICIBANK", "SBIN"]
    );

    let criteria = types::FilterCriteria {
        sector: "Financial Services".to_string(),
        ..Default::default()
    };
    assert_eq!(
        symbols(&filters::filter_records(&seed, &criteria)),
        ["HDFCBANK", "ICICIBANK", "SBIN", "PFC", "LICI"]
    );

    let criteria = types::FilterCriteria {
        market_cap_category: "Mid Cap".to_string(),
        ..Default::default()
    };
    assert_eq!(
        symbols(&filters::filter_records(&seed, &criteria)),
        ["PFC", "SAIL", "MOTHERSON"]
    );

    let criteria = types::FilterCriteria {
        fno_only: true,
        ..Default::default()
    };
    let got = filters::filter_records(&seed, &criteria);
    assert_eq!(got.len(), 10);
    assert!(!got.iter().any(|r| r.symbol == "SAIL" || r.symbol == "LICI"));

    let criteria = types::FilterCriteria {
        watchlist_only: true,
        ..Default::default()
    };
    assert_eq!(
        symbols(&filters::filter_records(&seed, &criteria)),
        ["RELIANCE", "HDFCBANK", "INFY", "ITC"]
    );
}

#[test]
fn filter_price_pe_rvol_rsi_bounds() {
    let seed = fixtures::seed_fixtures();

    let criteria = types::FilterCriteria {
        min_price: dec!(500),
        ..Default::default()
    };
    let got = filters::filter_records(&seed, &criteria);
    assert_eq!(
        symbols(&got),
        [
            "RELIANCE",
            "TCS",
            "HDFCBANK",
            "INFY",
            "ICICIBANK",
            "TATAMOTORS",
            "SBIN",
            "LICI"
        ]
    );
    assert_eq!(got.len(), 8);
    assert!(!got.iter().any(|r| r.symbol == "PFC"));

    let criteria = types::FilterCriteria {
        max_pe: dec!(15),
        ..Default::default()
    };
    assert_eq!(
        symbols(&filters::filter_records(&seed, &criteria)),
        ["TATAMOTORS", "SBIN", "PFC", "LICI"]
    );

    let criteria = types::FilterCriteria {
        min_rvol: dec!(2),
        ..Default::default()
    };
    assert_eq!(
        symbols(&filters::filter_records(&seed, &criteria)),
        ["RELIANCE", "INFY", "PFC"]
    );

    let criteria = types::FilterCriteria {
        min_pe: dec!(25),
        ..Default::default()
    };
    assert_eq!(
        symbols(&filters::filter_records(&seed, &criteria)),
        ["TCS", "INFY", "ITC"]
    );
}

#[test]
fn filter_rsi_band() {
    let seed = fixtures::seed_fixtures();
    let criteria = types::FilterCriteria {
        min_rsi: dec!(55),
        max_rsi: dec!(70),
        ..Default::default()
    };
    assert_eq!(
        symbols(&filters::filter_records(&seed, &criteria)),
        ["RELIANCE", "HDFCBANK", "INFY", "ICICIBANK", "ITC", "SBIN"]
    );
}

#[test]
fn filter_criteria_active_count() {
    let mut criteria = types::FilterCriteria::default();
    assert_eq!(criteria.active_count(), 0);

    criteria.search_query = "rel".to_string();
    assert_eq!(criteria.active_count(), 1);

    criteria.min_pe = dec!(15);
    assert_eq!(criteria.active_count(), 2);

    criteria.watchlist_only = true;
    assert_eq!(criteria.active_count(), 3);

    criteria.exchange = "NSE".to_string();
    assert_eq!(criteria.active_count(), 4);
}

#[test]
fn sort_records_market_cap_desc_and_asc() {
    let seed = fixtures::seed_fixtures();

    let desc = filters::sort_records(&seed, "market_cap_cr", false);
    assert_eq!(desc[0].symbol, "RELIANCE");
    assert_eq!(desc[1].symbol, "TCS");

    let asc = filters::sort_records(&seed, "market_cap_cr", true);
    assert_eq!(asc[0].symbol, "SAIL");
    assert_eq!(asc[1].symbol, "MOTHERSON");
}

#[test]
fn sort_records_by_symbol_and_volume_ties_stable() {
    let seed = fixtures::seed_fixtures();

    let asc = filters::sort_records(&seed, "symbol", true);
    assert_eq!(
        symbols(&asc),
        [
            "HDFCBANK",
            "ICICIBANK",
            "INFY",
            "ITC",
            "LICI",
            "MOTHERSON",
            "PFC",
            "RELIANCE",
            "SAIL",
            "SBIN",
            "TATAMOTORS",
            "TCS"
        ]
    );

    let desc = filters::sort_records(&seed, "volume", false);
    assert_eq!(
        symbols(&desc),
        [
            "MOTHERSON",
            "SAIL",
            "PFC",
            "SBIN",
            "ITC",
            "HDFCBANK",
            "ICICIBANK",
            "TATAMOTORS",
            "RELIANCE",
            "INFY",
            "LICI",
            "TCS"
        ]
    );

    // Unknown keys pass through unsorted.
    assert_eq!(filters::sort_records(&seed, "rvol", true), seed);

    // Ties keep insertion order even when sorting descending (stable sort).
    let tied = vec![fake_row("AAA", "Alpha Ltd"), fake_row("BBB", "Beta Ltd")];
    let sorted = filters::sort_records(&tied, "pe_ratio", false);
    assert_eq!(symbols(&sorted), ["AAA", "BBB"]);
}

#[test]
fn format_helpers_exact() {
    assert_eq!(filters::format_price(dec!(2842.5)), "₹2,842.50");
    assert_eq!(filters::format_price(dec!(63)), "₹63.00");

    assert_eq!(filters::format_change_pct(dec!(1.87)), "+1.87%");
    assert_eq!(filters::format_change_pct(dec!(-0.4)), "-0.40%");
    assert_eq!(filters::format_change_pct(dec!(0)), "0.00%");

    assert_eq!(filters::format_volume_compact(65_000_000), "6.5 Cr");
    assert_eq!(filters::format_volume_compact(4_200_000), "42.0 L");
    assert_eq!(filters::format_volume_compact(80_050), "80,050");
    assert_eq!(filters::format_volume_compact(10_000_000), "1.0 Cr");
    assert_eq!(filters::format_volume_compact(80_000_000), "8.0 Cr");

    assert_eq!(filters::format_mcap_rs_cr(dec!(1_920_000)), "₹19.2L Cr");
    assert_eq!(filters::format_mcap_rs_cr(dec!(725_000)), "₹7.25L Cr");
    assert_eq!(filters::format_mcap_rs_cr(dec!(63_000)), "₹63,000 Cr");
}
