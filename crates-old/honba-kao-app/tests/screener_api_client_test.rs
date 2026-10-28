//! Tests for the screener API client bindings (Milestone 2, "Reactive State
//! & API Integration"). These exercise `LocalScreenerApiClient` as a plain
//! struct/trait, and `SavedView` as a plain serde round-trip — no live
//! `ScreenerStore` instantiation (see `screener_store_test.rs` for why).

use honba_kao_app::features::screener::stores::api_client::{
    LocalScreenerApiClient, ScreenQuery, ScreenerApiClient,
};
use honba_kao_app::features::screener::stores::columns::ScreenerType;
use honba_kao_app::features::screener::stores::filter_engine::AdvancedFilters;
use honba_kao_app::features::screener::stores::saved_views::SavedView;
use honba_kao_app::features::screener::stores::screener_store::{ScreenView, SetupTab};

/// A query against the full "all stocks" India universe (no restricting
/// filters), matching the `stock_all` builtin screen's intent — guaranteed
/// non-empty against the bundled instrument data.
fn sample_query() -> ScreenQuery {
    ScreenQuery {
        screener_type: ScreenerType::Stocks,
        market: "IN",
        universe_id: "nse_500".to_string(),
        search_query: String::new(),
        quick_preset: "all".to_string(),
        filters: AdvancedFilters::default_filters(),
    }
}

#[test]
fn run_screen_returns_non_empty_rows_with_symbols() {
    let query = sample_query();
    let execution = LocalScreenerApiClient.run_screen(&query);
    assert!(!execution.rows.is_empty());
    assert!(execution.rows.iter().all(|row| !row.symbol.is_empty()));
}

#[test]
fn run_custom_formula_rejects_empty_expression() {
    let query = sample_query();
    let result = LocalScreenerApiClient.run_custom_formula("", &query);
    assert!(result.is_err());
}

#[test]
fn run_custom_formula_accepts_non_empty_expression() {
    let query = sample_query();
    let result = LocalScreenerApiClient.run_custom_formula("pe < 20", &query);
    assert!(result.is_ok());
}

#[test]
fn saved_view_round_trips_through_json() {
    let view = SavedView {
        id: "view_123".to_string(),
        name: "My saved view".to_string(),
        screener_type: ScreenerType::Etf,
        setup_tab: SetupTab::Saved,
        screen_view: ScreenView::Scores,
        selected_preset_id: Some("etf_vault".to_string()),
        selected_universe_id: "us_large_cap".to_string(),
        query: "pe < 20".to_string(),
        created_at_ms: 1_700_000_000_123,
    };

    let json = serde_json::to_string(&view).expect("serialize SavedView");
    let round_tripped: SavedView = serde_json::from_str(&json).expect("deserialize SavedView");

    assert_eq!(round_tripped, view);
}
