//! Screener store pure-logic tests (Milestone 2, "Reactive State & API
//! Integration"). Relocated here from the formerly-inline `#[cfg(test)] mod
//! tests` block at the bottom of `screener_store.rs` per the mandatory
//! parallel-test-directory rule — no in-file `#[test]` blocks in `src/`.
//!
//! As with the rest of this crate's test suite, these tests exercise pure
//! functions and plain data structs directly; none of them instantiate a
//! live `ScreenerStore` (whose fields are `RwSignal`s that expect a real
//! Leptos reactive owner).

use honba_kao_app::features::screener::data::instruments;
use honba_kao_app::features::screener::stores::filter_engine::{
    filter_instruments, normalize_filters,
};
use honba_kao_app::features::screener::stores::screener_store::{
    ScreenView, SetupTab, builtin_screens_registry,
};
use honba_kao_app::features::screener::stores::universes::default_universes;

/// Relocated verbatim from `screener_store.rs::tests::builtin_screens_apply_and_filter`.
#[test]
fn builtin_screens_apply_and_filter() {
    let overvalued = filter_instruments(
        &instruments::universe_for_screener("IN", "stocks"),
        "",
        "all",
        &normalize_filters(&builtin_screens_registry()[4].filters),
        &[],
    );
    assert!(
        overvalued
            .iter()
            .all(|inst| inst.pe > 0.0 && inst.pe <= 25.0)
    );
}

#[test]
fn default_universes_returns_the_three_expected_ids_in_order() {
    let ids: Vec<String> = default_universes().into_iter().map(|u| u.id).collect();
    assert_eq!(ids, vec!["nse_500", "all_india", "us_large_cap"]);
}

#[test]
fn setup_tab_default_is_library() {
    assert_eq!(SetupTab::default(), SetupTab::Library);
}

#[test]
fn screen_view_default_is_table() {
    assert_eq!(ScreenView::default(), ScreenView::Table);
}
