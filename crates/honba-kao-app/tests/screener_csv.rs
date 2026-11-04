//! Contract tests for the screener CSV export builders
//! (`features::screener::csv_export`). Pure-logic, host-runnable; the WASM
//! downloader is exercised in the browser only.

use honba_kao_app::features::screener::csv_export::{
    csv_document, csv_escape, csv_headers, csv_rows,
};
use honba_kao_app::features::screener::filters::record_cell;
use honba_kao_app::features::screener::fixtures::equity_fixture;
use honba_kao_app::features::screener::types::{
    COLUMN_CATALOG, DEFAULT_VISIBLE_COLUMNS, ScreenerColumn, ScreenerRecord,
};
use rust_decimal_macros::dec;

/// Overview-visible columns in the contract order (star, symbol, then metrics).
fn overview_columns() -> Vec<ScreenerColumn> {
    COLUMN_CATALOG
        .iter()
        .filter(|col| DEFAULT_VISIBLE_COLUMNS.contains(&col.key))
        .copied()
        .collect()
}

#[test]
fn t1_headers_skip_star_and_match_visible_catalog() {
    let columns = overview_columns();
    let headers = csv_headers(&columns);
    assert_eq!(headers.len(), columns.len() - 1, "star column dropped");
    assert!(!headers.iter().any(|h| h.contains('★')));
    assert_eq!(
        headers[0], columns[1].label,
        "first exported column follows star"
    );
}

#[test]
fn t2_rows_use_record_cell_per_column() {
    let record = ScreenerRecord::Stock(equity_fixture(
        "RELIANCE",
        "Reliance Industries Ltd",
        "NSE",
        "Energy",
        "Oil & Gas",
        "Large Cap",
        true,
        true,
        dec!(2842.50),
        dec!(1.87),
        4_200_000,
        2_000_000,
        dec!(24.20),
        dec!(0.35),
        dec!(8.50),
        dec!(1_920_000),
        dec!(2790.00),
        dec!(2860.00),
        dec!(2220.07),
        dec!(3200.00),
        dec!(62.40),
    ));
    let columns = overview_columns();
    let rows = csv_rows(std::slice::from_ref(&record), &columns);
    assert_eq!(rows.len(), 1);
    let visible: Vec<&ScreenerColumn> = columns.iter().filter(|c| c.key != "star").collect();
    assert_eq!(rows[0].len(), visible.len());
    assert_eq!(rows[0][0], "RELIANCE", "first exported cell is the ticker");
    for (i, col) in visible.iter().enumerate() {
        let expected = match col.key {
            "symbol" => record.symbol().to_string(),
            "name" => record.name().to_string(),
            other => record_cell(&record, other),
        };
        assert_eq!(rows[0][i], expected, "cell for {col:?}");
    }
}

#[test]
fn t3_escape_quotes_comma_quote_newline_only() {
    assert_eq!(csv_escape("plain"), "plain");
    assert_eq!(csv_escape("₹725,000 Cr"), "\"₹725,000 Cr\"");
    assert_eq!(csv_escape("a\"b"), "\"a\"\"b\"");
    assert_eq!(csv_escape("line\nbreak"), "\"line\nbreak\"");
}

#[test]
fn t4_document_shape_with_trailing_newline() {
    let headers = vec!["Symbol".to_string(), "Price".to_string()];
    let rows = vec![
        vec!["RELIANCE".to_string(), "₹2,842.50".to_string()],
        vec!["TCS".to_string(), "3,920.10".to_string()],
    ];
    assert_eq!(
        csv_document(&headers, &rows),
        "Symbol,Price\nRELIANCE,\"₹2,842.50\"\nTCS,\"3,920.10\"\n"
    );
}
