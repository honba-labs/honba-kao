//! CommandPalette row model: which rows a query produces and how they rank.

use honba_kao_app::shared::command::palette::{PaletteTarget, build_items};

#[test]
fn empty_query_lists_navigation_first() {
    let rows = build_items("");
    assert_eq!(rows.first().map(|r| r.badge), Some("NAV"));
    assert!(rows.iter().any(|r| r.label == "Portfolio"));
}

#[test]
fn function_word_surfaces_command_row() {
    let rows = build_items("port");
    assert!(rows.iter().any(|r| r.badge == "CMD" && r.label == "PORT"));
}

#[test]
fn ticker_query_surfaces_symbol_and_hints() {
    let rows = build_items("reliance");
    assert!(
        rows.iter()
            .any(|r| r.badge == "ASSET" && r.label == "RELIANCE")
    );
    let rows = build_items("infy");
    assert!(
        rows.iter()
            .any(|r| r.badge == "FUNC" && r.label == "INFY DES")
    );
}

#[test]
fn unknown_ticker_offers_open_market_page() {
    let rows = build_items("ZZQX");
    let row = rows
        .iter()
        .find(|r| r.description == "Open market page")
        .expect("ticker row");
    assert_eq!(row.target, PaletteTarget::Command("ZZQX".into()));
}

#[test]
fn rows_are_sorted_best_first_and_capped() {
    let rows = build_items("");
    assert!(rows.len() <= 40);
    assert!(rows.windows(2).all(|w| w[0].score >= w[1].score));
}
