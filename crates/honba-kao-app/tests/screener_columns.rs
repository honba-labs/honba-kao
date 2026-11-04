//! Phase 5: per-tab column catalogs and formatted Decimal cells (red-first
//! target for the parallel `types::` tab catalogs + `filters::` cell arms).
//! The column header menu's pure-function contract lives in
//! `screener_column_menu.rs`.

use honba_kao_app::features::screener::{filters, fixtures, types};
use rust_decimal_macros::dec;

#[test]
fn tab_registry_matches_doc() {
    assert_eq!(
        types::TAB_KEYS,
        [
            "overview",
            "performance",
            "technicals",
            "valuation",
            "dividends",
            "profitability",
            "financials",
            "income_statement",
            "balance_sheet",
            "cash_flow",
            "per_share",
            "forecasts",
        ]
    );
}

#[test]
fn overview_catalog_is_the_default() {
    // Overview (and any unknown/empty key) falls back to the stock catalog.
    // NOTE: `const` slices are inlined per use site in Rust, so identity is
    // pinned by VALUE (key sequence), not by pointer equality.
    let overview_keys: Vec<&str> = types::columns_for_tab("overview")
        .iter()
        .map(|c| c.key)
        .collect();
    let stock_keys: Vec<&str> = types::COLUMN_CATALOG.iter().map(|c| c.key).collect();
    assert_eq!(overview_keys, stock_keys);
    for tab_key in ["", "whatever", "OVERVIEW"] {
        let keys: Vec<&str> = types::columns_for_tab(tab_key)
            .iter()
            .map(|c| c.key)
            .collect();
        assert_eq!(keys, stock_keys, "unknown tab {tab_key} must fall back");
    }
    assert_eq!(stock_keys, types::DEFAULT_VISIBLE_COLUMNS);
}

#[test]
fn tab_catalogs_start_with_identity_columns() {
    for tab in types::TAB_KEYS {
        let cols = types::columns_for_tab(tab);
        assert_eq!(cols[0].key, "star", "tab {tab} must lead with star");
        assert_eq!(cols[1].key, "symbol", "tab {tab} must lead with symbol");
    }
}

#[test]
fn tab_catalog_shapes() {
    let expected: &[(&str, usize)] = &[
        ("overview", 9),
        ("performance", 11),
        ("technicals", 11),
        ("valuation", 8),
        ("dividends", 5),
        ("profitability", 6),
        ("financials", 6),
        ("income_statement", 5),
        ("balance_sheet", 6),
        ("cash_flow", 3),
        ("per_share", 5),
        ("forecasts", 4),
    ];
    for (tab, len) in expected {
        assert_eq!(types::columns_for_tab(tab).len(), *len, "tab {tab}");
    }
}

#[test]
fn tab_metric_keys_are_unique_and_known() {
    // Guard against typos: every tab column key must be one of the real
    // `ScreenerEquityRecord` sort keys (hand-verified against the record).
    const KNOWN: &[&str] = &[
        "star",
        "symbol",
        "price",
        "change_pct",
        "volume",
        "rvol",
        "market_cap_cr",
        "pe_ratio",
        "range_52w",
        "return_1d",
        "return_1w",
        "return_1m",
        "return_3m",
        "return_6m",
        "return_ytd",
        "return_1y",
        "return_3y",
        "atr_pct",
        "rsi_14",
        "macd_hist",
        "supertrend_direction",
        "sma_20",
        "sma_50",
        "sma_200",
        "ema_9",
        "ema_21",
        "bb_bandwidth",
        "forward_pe",
        "pb_ratio",
        "ps_ratio",
        "ev_ebitda",
        "peg_ratio",
        "dividend_yield",
        "payout_ratio",
        "dividend_per_share",
        "roe",
        "roce",
        "operating_margin",
        "net_margin",
        "revenue_cr",
        "revenue_growth_yoy",
        "net_profit_cr",
        "eps",
        "debt_to_equity",
        "current_ratio",
        "interest_coverage",
        "book_value_per_share",
        "fcf_cr",
    ];
    for tab in types::TAB_KEYS {
        let mut seen: Vec<&str> = Vec::new();
        for col in types::columns_for_tab(tab) {
            assert!(
                KNOWN.contains(&col.key),
                "unknown key {0} in tab {tab}",
                col.key
            );
            assert!(
                !seen.contains(&col.key),
                "duplicate key {0} in tab {tab}",
                col.key
            );
            seen.push(col.key);
        }
    }
}

#[test]
fn master_column_keys_shape() {
    let keys = types::all_master_column_keys();
    // The seed order of the flags/order map is today's stock catalog.
    assert_eq!(
        &keys[..9],
        [
            "star",
            "symbol",
            "price",
            "change_pct",
            "volume",
            "rvol",
            "market_cap_cr",
            "pe_ratio",
            "range_52w",
        ]
    );
    // Deduplicated across every asset class and every tab.
    let unique: std::collections::HashSet<&&str> = keys.iter().collect();
    assert_eq!(unique.len(), keys.len(), "master keys must be deduplicated");
    for probe in [
        "ltp",
        "coupon_rate",
        "cagr_3y",
        "revenue_cr",
        "fcf_cr",
        "rsi_14",
    ] {
        assert!(keys.contains(&probe), "master keys missing {probe}");
    }
}

#[test]
fn record_cell_golden_cells() {
    let reliance = fixtures::seed_fixtures().remove(0);
    let record = types::ScreenerRecord::Stock(reliance.clone());
    let goldens: &[(&str, &str)] = &[
        // Legacy overview cells (regression).
        ("price", "₹2,842.50"),
        ("pe_ratio", "24.2"),
        ("volume", "42.0 L"),
        ("symbol", ""),
        // Percent metrics.
        ("rsi_14", "62.40 %"),
        ("roe", "9.80 %"),
        ("dividend_yield", "0.35 %"),
        ("atr_pct", "0.00 %"),
        ("bb_bandwidth", "0.00 %"),
        ("payout_ratio", "0.00 %"),
        // Signed returns.
        ("return_1d", "+1.87%"),
        ("return_1w", "0.00%"),
        ("revenue_growth_yoy", "0.00%"),
        // Plain decimal metrics.
        ("forward_pe", "0.00"),
        ("pb_ratio", "0.00"),
        ("dividend_per_share", "0.00"),
        ("macd_hist", "0.00"),
        ("sma_200", "0.00"),
        ("ema_9", "0.00"),
        ("eps", "0.00"),
        ("debt_to_equity", "0.00"),
        ("current_ratio", "0.00"),
        ("interest_coverage", "0.00"),
        ("book_value_per_share", "0.00"),
        // Crore amounts.
        ("revenue_cr", "₹0 Cr"),
        ("net_profit_cr", "₹0 Cr"),
        ("fcf_cr", "₹0 Cr"),
        // Supertrend direction.
        ("supertrend_direction", "—"),
    ];
    for (key, expected) in goldens {
        assert_eq!(
            filters::record_cell(&record, key),
            *expected,
            "golden cell for {key}"
        );
    }
    assert_eq!(reliance.supertrend_direction, 0);
}

#[test]
fn record_cell_golden_cells_positive_values() {
    // ITC carries yield 3.45 and BPCL 6.05; RELIANCE rvol=2.1 for the CR scale.
    let itc = fixtures::seed_fixtures()
        .into_iter()
        .find(|r| r.symbol == "ITC")
        .expect("ITC seed row");
    let record = types::ScreenerRecord::Stock(itc);
    assert_eq!(filters::record_cell(&record, "dividend_yield"), "3.45 %");
    assert_eq!(filters::record_cell(&record, "roe"), "27.40 %");
}

#[test]
fn formatter_helpers() {
    assert_eq!(filters::format_supertrend(1), "▲ Buy");
    assert_eq!(filters::format_supertrend(-1), "▼ Sell");
    assert_eq!(filters::format_supertrend(0), "—");
    assert_eq!(filters::format_cr_amount(dec!(725_000)), "₹725,000 Cr");
    assert_eq!(filters::format_cr_amount(dec!(0)), "₹0 Cr");
    assert_eq!(filters::format_cr_amount(dec!(-12_345)), "₹-12,345 Cr");
}

#[test]
fn tab_sorting_via_sort_decimal() {
    let universe = fixtures::universe_for(types::ScreenerAssetType::Stocks);
    // New numeric keys sort through the extended Stock arms of
    // `ScreenerRecord::sort_decimal`.
    let rsi_asc = filters::sort_universe(&universe, "rsi_14", true);
    assert_eq!(rsi_asc[0].symbol(), "AMBUJACEM"); // 41.60
    let rsi_desc = filters::sort_universe(&universe, "rsi_14", false);
    assert_eq!(rsi_desc[0].symbol(), "PFC"); // 71.20
    let yield_desc = filters::sort_universe(&universe, "dividend_yield", false);
    assert_eq!(yield_desc[0].symbol(), "BPCL"); // 6.05
    // Regression: the default sort key is untouched.
    let cap_desc = filters::sort_universe(&universe, "market_cap_cr", false);
    assert_eq!(cap_desc[0].symbol(), "RELIANCE");
    assert_eq!(cap_desc[1].symbol(), "TCS");
}

// The header menu's pure functions (`swap_in_order`, `filter_slots_for`) are
// pinned in `tests/screener_column_menu.rs`.
