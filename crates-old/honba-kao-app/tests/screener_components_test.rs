//! Pure-logic tests for the Milestone 3 screener components ("Hero Header,
//! Preset Shelf & Query Configuration Deck", `plan-screener.md`). As with
//! the rest of this crate's test suite (see `screener_store_test.rs`), these
//! exercise plain functions and data structs directly — no live Leptos
//! reactive owner is instantiated.

use honba_kao_app::features::screener::components::custom_formula::formula_token_sheet;
use honba_kao_app::features::screener::components::multi_market_scan::toggle_universe_selection;
use honba_kao_app::features::screener::components::query_builder::{
    parse_filter_value, query_builder_rows, query_status_label,
};
use honba_kao_app::features::screener::components::screen_setup::active_filter_chips;
use honba_kao_app::features::screener::stores::filter_engine::AdvancedFilters;

// ---------------------------------------------------------------------------
// screen_setup::active_filter_chips
// ---------------------------------------------------------------------------

#[test]
fn active_filter_chips_is_empty_for_default_filters() {
    assert!(active_filter_chips(&AdvancedFilters::default_filters()).is_empty());
}

#[test]
fn active_filter_chips_formats_set_fields() {
    let filters = AdvancedFilters {
        pe_max: Some(25.0),
        min_roce: Some(15.0),
        near_52_week_high: true,
        ..AdvancedFilters::default_filters()
    };
    let chips = active_filter_chips(&filters);
    assert!(chips.contains(&"P/E <= 25".to_string()));
    assert!(chips.contains(&"ROCE >= 15%".to_string()));
    assert!(chips.contains(&"Near 52W high".to_string()));
    assert_eq!(chips.len(), 3);
}

#[test]
fn active_filter_chips_formats_descriptive_selects() {
    let filters = AdvancedFilters {
        sector: "Technology".into(),
        market_cap_tier: "mega".into(),
        ..AdvancedFilters::default_filters()
    };
    let chips = active_filter_chips(&filters);
    assert!(chips.contains(&"Sector = Technology".to_string()));
    assert!(chips.contains(&"Market cap = mega".to_string()));
}

// ---------------------------------------------------------------------------
// query_builder
// ---------------------------------------------------------------------------

#[test]
fn query_status_label_reflects_emptiness() {
    assert_eq!(query_status_label(""), "Empty");
    assert_eq!(query_status_label("   "), "Empty");
    assert_eq!(query_status_label("pe < 25"), "Ready");
}

#[test]
fn parse_filter_value_clears_on_empty_or_invalid_input() {
    assert_eq!(parse_filter_value(""), None);
    assert_eq!(parse_filter_value("   "), None);
    assert_eq!(parse_filter_value("not-a-number"), None);
    assert_eq!(parse_filter_value("25"), Some(25.0));
    assert_eq!(parse_filter_value(" 12.5 "), Some(12.5));
}

#[test]
fn query_builder_rows_contract_is_stable() {
    let rows = query_builder_rows();
    assert_eq!(rows.len(), 7);
    assert!(rows.contains(&("P/E", ">=")));
    assert!(rows.contains(&("P/E", "<=")));
    assert!(rows.contains(&("RSI (14)", ">=")));
    assert!(rows.contains(&("RSI (14)", "<=")));
}

// ---------------------------------------------------------------------------
// custom_formula::formula_token_sheet
// ---------------------------------------------------------------------------

#[test]
fn formula_token_sheet_has_at_least_25_numeric_metric_tokens() {
    let tokens = formula_token_sheet();
    assert!(
        tokens.len() >= 25,
        "expected 25+ formula tokens, got {}",
        tokens.len()
    );
    assert!(tokens.contains(&"pe"));
    assert!(tokens.contains(&"roce"));
    assert!(tokens.contains(&"dividendYield"));
}

#[test]
fn formula_token_sheet_excludes_categorical_columns() {
    let tokens = formula_token_sheet();
    assert!(!tokens.contains(&"symbol"));
    assert!(!tokens.contains(&"sector"));
    assert!(!tokens.contains(&"technicalRating"));
}

// ---------------------------------------------------------------------------
// multi_market_scan::toggle_universe_selection
// ---------------------------------------------------------------------------

#[test]
fn toggle_universe_selection_adds_when_absent() {
    let selected = vec!["nse_500".to_string()];
    let next = toggle_universe_selection(&selected, "us_large_cap");
    assert_eq!(
        next,
        vec!["nse_500".to_string(), "us_large_cap".to_string()]
    );
}

#[test]
fn toggle_universe_selection_removes_when_present() {
    let selected = vec!["nse_500".to_string(), "us_large_cap".to_string()];
    let next = toggle_universe_selection(&selected, "nse_500");
    assert_eq!(next, vec!["us_large_cap".to_string()]);
}
