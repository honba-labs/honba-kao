use honba_kao_app::features::screener::{filters, fixtures, heatmap, types};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Collects the identity string of each union record in order.
fn sym(records: Vec<types::ScreenerRecord>) -> Vec<String> {
    records
        .iter()
        .map(|record| record.symbol().to_string())
        .collect()
}

#[test]
fn universe_fixture_counts_and_determinism() {
    assert_eq!(fixtures::seed_fixtures().len(), 12);
    assert_eq!(fixtures::stock_universe().len(), 30);

    assert_eq!(fixtures::stock_universe(), fixtures::stock_universe());
    assert_eq!(fixtures::etf_fixtures(), fixtures::etf_fixtures());
    assert_eq!(fixtures::bond_fixtures(), fixtures::bond_fixtures());
    assert_eq!(fixtures::mf_fixtures(), fixtures::mf_fixtures());

    assert_eq!(
        fixtures::universe_for(types::ScreenerAssetType::Stocks).len(),
        30
    );
    assert_eq!(
        fixtures::universe_for(types::ScreenerAssetType::Etf).len(),
        15
    );
    assert_eq!(
        fixtures::universe_for(types::ScreenerAssetType::Bonds).len(),
        10
    );
    assert_eq!(
        fixtures::universe_for(types::ScreenerAssetType::MutualFunds).len(),
        10
    );
}

#[test]
fn stock_universe_extends_seed_in_order() {
    let seed: Vec<String> = fixtures::seed_fixtures()
        .iter()
        .map(|record| record.symbol.clone())
        .collect();
    let head: Vec<String> = fixtures::stock_universe()
        .iter()
        .take(12)
        .map(|record| record.symbol.clone())
        .collect();
    assert_eq!(head, seed);
}

#[test]
fn stock_universe_golden_orderings() {
    let universe = fixtures::universe_for(types::ScreenerAssetType::Stocks);

    let by_symbol = sym(filters::sort_universe(&universe, "symbol", true));
    let expected_symbol = [
        "AMBUJACEM",
        "ASHOKLEY",
        "BEL",
        "BPCL",
        "DLF",
        "HCLTECH",
        "HDFCBANK",
        "HINDUNILVR",
        "ICICIBANK",
        "INFY",
        "ITC",
        "JSWSTEEL",
        "LICI",
        "MOTHERSON",
        "NATIONALUM",
        "NMDC",
        "NTPC",
        "ONGC",
        "PFC",
        "RELIANCE",
        "SAIL",
        "SBIN",
        "SUNPHARMA",
        "TATAMOTORS",
        "TATAPOWER",
        "TATASTEEL",
        "TCS",
        "TECHM",
        "UPL",
        "WIPRO",
    ];
    assert_eq!(by_symbol, expected_symbol);

    let by_volume = sym(filters::sort_universe(&universe, "volume", false));
    let expected_volume = [
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
        "TCS",
        "WIPRO",
        "HCLTECH",
        "TECHM",
        "HINDUNILVR",
        "SUNPHARMA",
        "BEL",
        "NTPC",
        "TATAPOWER",
        "TATASTEEL",
        "ONGC",
        "BPCL",
        "ASHOKLEY",
        "DLF",
        "JSWSTEEL",
        "NMDC",
        "NATIONALUM",
        "UPL",
        "AMBUJACEM",
    ];
    assert_eq!(by_volume, expected_volume);

    let by_cap_desc = sym(filters::sort_universe(&universe, "market_cap_cr", false));
    let expected_cap = [
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
    assert_eq!(by_cap_desc, expected_cap);

    let by_cap_asc = sym(filters::sort_universe(&universe, "market_cap_cr", true));
    let mut reversed = by_cap_desc.clone();
    reversed.reverse();
    assert_eq!(by_cap_asc, reversed);
}

#[test]
fn stock_criteria_parity_on_universe() {
    let universe = fixtures::universe_for(types::ScreenerAssetType::Stocks);

    let expensive = types::FilterCriteria {
        min_price: dec!(500),
        ..types::FilterCriteria::default()
    };
    let hits = sym(filters::filter_universe(&universe, &expensive));
    assert_eq!(
        hits,
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

    let watchlist = types::FilterCriteria {
        watchlist_only: true,
        ..types::FilterCriteria::default()
    };
    let hits = sym(filters::filter_universe(&universe, &watchlist));
    assert_eq!(hits, ["RELIANCE", "HDFCBANK", "INFY", "ITC"]);

    let fno = types::FilterCriteria {
        fno_only: true,
        ..types::FilterCriteria::default()
    };
    let hits = sym(filters::filter_universe(&universe, &fno));
    assert_eq!(hits.len(), 10);
    assert!(!hits.iter().any(|symbol| symbol == "SAIL"));
    assert!(!hits.iter().any(|symbol| symbol == "LICI"));
}

#[test]
fn heatmap_clusters_over_30_row_universe() {
    let groups = heatmap::group_sectors(&fixtures::stock_universe());
    assert_eq!(groups.len(), 12);

    let members: usize = groups.iter().map(|group| group.members.len()).sum();
    assert_eq!(members, 30);

    let financials = groups
        .iter()
        .find(|group| group.sector == "Financial Services")
        .expect("Financial Services sector group");
    assert_eq!(financials.members.len(), 5);
}

#[test]
fn etf_fixture_derives_ltp_and_universe() {
    let etfs = fixtures::etf_fixtures();
    assert_eq!(etfs[0].ltp, dec!(262.872320));

    let niftybees = types::ScreenerRecord::Etf(etfs[0].clone());
    assert_eq!(filters::record_cell(&niftybees, "ltp"), "₹262.87");
    assert_eq!(
        filters::record_cell(&niftybees, "premium_discount_pct"),
        "+0.18%"
    );

    let silverbees = types::ScreenerRecord::Etf(etfs[4].clone());
    assert_eq!(
        filters::record_cell(&silverbees, "premium_discount_pct"),
        "-0.08%"
    );
}

#[test]
fn etf_sort_and_search() {
    let universe = fixtures::universe_for(types::ScreenerAssetType::Etf);

    let sorted = sym(filters::sort_universe(&universe, "aum_cr", false));
    let expected_aum = [
        "LIQUIDBEES",
        "NIFTYBEES",
        "HNGSNGBEES",
        "GOLDBEES",
        "MON100",
        "SILVERBEES",
        "BANKBEES",
        "MID150BEES",
        "JUNIORBEES",
        "MAFANG",
        "MOM50",
        "ITBEES",
        "PSUBNKBEES",
        "GILTBEES",
        "ALPHABEES",
    ];
    assert_eq!(sorted, expected_aum);

    let gold = types::FilterCriteria {
        search_query: "gold".to_string(),
        ..types::FilterCriteria::default()
    };
    let hits = sym(filters::filter_universe(&universe, &gold));
    assert_eq!(hits, ["GOLDBEES"]);

    let nifty50 = types::FilterCriteria {
        search_query: "nifty 50".to_string(),
        ..types::FilterCriteria::default()
    };
    let hits = sym(filters::filter_universe(&universe, &nifty50));
    assert_eq!(hits, ["NIFTYBEES", "HNGSNGBEES", "MOM50"]);
}

#[test]
fn bond_sort_and_search() {
    let universe = fixtures::universe_for(types::ScreenerAssetType::Bonds);

    let sorted = sym(filters::sort_universe(&universe, "maturity_date", true));
    let expected_maturity = [
        "INE496D08M24",
        "INE062A08043",
        "IN0020190074",
        "INE181I080Q9",
        "INE053F07TR0",
        "INE476A08256",
        "IN0020150053",
        "INE436G08NB8",
        "INE0POA07RE7",
        "INE134E08KM1",
    ];
    assert_eq!(sorted, expected_maturity);

    let sdl = types::FilterCriteria {
        search_query: "sdl".to_string(),
        ..types::FilterCriteria::default()
    };
    let hits = sym(filters::filter_universe(&universe, &sdl));
    assert_eq!(hits, ["INE436G08NB8", "INE181I080Q9"]);

    let first = types::ScreenerRecord::Bond(fixtures::bond_fixtures()[0].clone());
    assert_eq!(filters::record_cell(&first, "coupon_rate"), "7.26 %");
    assert_eq!(filters::record_cell(&first, "clean_price"), "₹101.35");
    assert_eq!(filters::record_cell(&first, "maturity_date"), "2032-08-25");
    assert_eq!(filters::record_cell(&first, "credit_rating"), "SOVEREIGN");
    assert_eq!(filters::record_cell(&first, "modified_duration"), "6.10");
}

#[test]
fn mf_sort_search_and_cells() {
    let universe = fixtures::universe_for(types::ScreenerAssetType::MutualFunds);

    let sorted = sym(filters::sort_universe(&universe, "cagr_3y", false));
    assert_eq!(
        sorted,
        [
            "119793", "118959", "122649", "120714", "120465", "148103", "147946", "120503",
            "148879", "148796"
        ]
    );

    let parag = types::FilterCriteria {
        search_query: "parag".to_string(),
        ..types::FilterCriteria::default()
    };
    let hits = sym(filters::filter_universe(&universe, &parag));
    assert_eq!(hits, ["120465"]);

    let first = types::ScreenerRecord::Mf(fixtures::mf_fixtures()[0].clone());
    assert_eq!(filters::record_cell(&first, "nav"), "₹82.45");
    assert_eq!(filters::record_cell(&first, "expense_ratio"), "0.63 %");
    assert_eq!(filters::record_cell(&first, "risk_grade"), "Very High");

    let quant = types::ScreenerRecord::Mf(fixtures::mf_fixtures()[1].clone());
    assert_eq!(filters::record_cell(&quant, "cagr_3y"), "+34.20%");
}

#[test]
fn catalog_for_and_from_bar() {
    assert_eq!(
        types::catalog_for(types::ScreenerAssetType::Stocks).len(),
        9
    );
    assert_eq!(types::catalog_for(types::ScreenerAssetType::Etf).len(), 9);
    assert_eq!(types::catalog_for(types::ScreenerAssetType::Bonds).len(), 9);
    assert_eq!(
        types::catalog_for(types::ScreenerAssetType::MutualFunds).len(),
        8
    );

    let etf_keys: Vec<&str> = types::ETF_COLUMN_CATALOG.iter().map(|c| c.key).collect();
    let expected_etf = [
        "symbol",
        "nav",
        "ltp",
        "premium_discount_pct",
        "aum_cr",
        "expense_ratio",
        "tracking_error",
        "return_1y",
        "volume",
    ];
    assert_eq!(etf_keys, expected_etf);

    let bond_keys: Vec<&str> = types::BOND_COLUMN_CATALOG.iter().map(|c| c.key).collect();
    let expected_bond = [
        "symbol",
        "bond_type",
        "coupon_rate",
        "yield_to_maturity",
        "clean_price",
        "face_value",
        "maturity_date",
        "modified_duration",
        "credit_rating",
    ];
    assert_eq!(bond_keys, expected_bond);

    let mf_keys: Vec<&str> = types::MF_COLUMN_CATALOG.iter().map(|c| c.key).collect();
    let expected_mf = [
        "symbol",
        "category",
        "nav",
        "aum_cr",
        "expense_ratio",
        "cagr_3y",
        "cagr_5y",
        "risk_grade",
    ];
    assert_eq!(mf_keys, expected_mf);

    assert_eq!(
        types::ScreenerAssetType::from_bar("etf"),
        types::ScreenerAssetType::Etf
    );
    assert_eq!(
        types::ScreenerAssetType::from_bar("bonds"),
        types::ScreenerAssetType::Bonds
    );
    assert_eq!(
        types::ScreenerAssetType::from_bar("mf"),
        types::ScreenerAssetType::MutualFunds
    );
    assert_eq!(
        types::ScreenerAssetType::from_bar("stocks"),
        types::ScreenerAssetType::Stocks
    );
    assert_eq!(
        types::ScreenerAssetType::from_bar("nonsense"),
        types::ScreenerAssetType::Stocks
    );
}

#[test]
fn union_accessors_present_coherent() {
    let etf = types::ScreenerRecord::Etf(fixtures::etf_fixtures()[0].clone());
    assert_eq!(etf.symbol(), "NIFTYBEES");
    assert_eq!(etf.price(), fixtures::etf_fixtures()[0].ltp);

    let bond = types::ScreenerRecord::Bond(fixtures::bond_fixtures()[0].clone());
    assert_eq!(bond.symbol(), "IN0020150053");
    assert_eq!(bond.price(), fixtures::bond_fixtures()[0].clean_price);
    assert_eq!(bond.change_pct(), Decimal::ZERO);

    let mf = types::ScreenerRecord::Mf(fixtures::mf_fixtures()[0].clone());
    assert_eq!(mf.symbol(), "120465");
    assert_eq!(mf.price(), fixtures::mf_fixtures()[0].nav);
    assert_eq!(mf.change_pct(), Decimal::ZERO);
}
