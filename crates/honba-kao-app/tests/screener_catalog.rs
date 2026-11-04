//! Tests for the screener screen catalog (`features/screener/screen_catalog.rs`).

use honba_kao_app::features::screener::screen_catalog::{
    CustomScreen, builtin_groups, builtin_screens, delete_screen, new_screen_id, snapshot_screen,
    upsert_screen,
};
use honba_kao_app::features::screener::types::FilterCriteria;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Sort column keys accepted by `filters.rs::sort_records`.
const SORT_COLS: [&str; 7] = [
    "symbol",
    "price",
    "change_pct",
    "volume",
    "market_cap_cr",
    "pe_ratio",
    "rsi_14",
];

/// Feature tabs the screener page can render.
const TABS: [&str; 6] = [
    "overview",
    "performance",
    "technicals",
    "valuation",
    "profitability",
    "dividends",
];

/// Minimal constructible screen used by the mutation tests.
fn screen(id: &str) -> CustomScreen {
    CustomScreen {
        id: id.to_string(),
        name: format!("Screen {id}"),
        description: format!("Description {id}"),
        tab: "overview".to_string(),
        sort_col: "symbol".to_string(),
        sort_ascending: true,
        criteria: FilterCriteria::default(),
        visible_columns: Vec::new(),
    }
}

#[test]
fn t1_builtin_screens_has_twelve_entries() {
    assert_eq!(builtin_screens().len(), 12);
}

#[test]
fn t2_builtin_ids_unique_and_prefixed() {
    let screens = builtin_screens();
    let mut ids: Vec<&str> = screens.iter().map(|s| s.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), screens.len(), "duplicate builtin ids");
    for s in &screens {
        assert!(
            s.id.starts_with("builtin."),
            "id {} lacks the builtin. prefix",
            s.id
        );
    }
}

#[test]
fn t3_groups_partition_builtins_exactly_once() {
    let groups = builtin_groups();
    let shelves: Vec<&str> = groups.iter().map(|(name, _)| *name).collect();
    assert!(
        shelves.contains(&"Popular"),
        "missing Popular shelf: {shelves:?}"
    );

    let mut grouped_ids: Vec<String> = groups
        .iter()
        .flat_map(|(_, v)| v.iter().map(|s| s.id.clone()))
        .collect();
    grouped_ids.sort_unstable();

    let mut deduped = grouped_ids.clone();
    deduped.dedup();
    assert_eq!(
        deduped.len(),
        grouped_ids.len(),
        "a screen appears in more than one shelf"
    );

    let mut direct_ids: Vec<String> = builtin_screens().iter().map(|s| s.id.clone()).collect();
    direct_ids.sort_unstable();
    assert_eq!(
        grouped_ids, direct_ids,
        "shelves must partition builtin_screens() exactly once"
    );
}

#[test]
fn t4_descriptions_non_empty_and_distinct_from_names() {
    for s in builtin_screens() {
        assert!(
            !s.description.trim().is_empty(),
            "{} has an empty description",
            s.id
        );
        assert_ne!(
            s.description, s.name,
            "{} description must not repeat the name",
            s.id
        );
    }
}

#[test]
fn t5_criteria_and_sort_coverage() {
    let screens = builtin_screens();

    // (a) at least one P/E upper bound is active.
    assert!(screens.iter().any(|s| s.criteria.max_pe != Decimal::ZERO));
    // (b) at least one RVOL floor is active.
    assert!(screens.iter().any(|s| s.criteria.min_rvol != Decimal::ZERO));
    // (c) F&O-only screen exists.
    assert!(screens.iter().any(|s| s.criteria.fno_only));
    // (d) watchlist-only screen exists.
    assert!(screens.iter().any(|s| s.criteria.watchlist_only));
    // (e) Small Cap category screen exists.
    assert!(
        screens
            .iter()
            .any(|s| s.criteria.market_cap_category == "Small Cap")
    );

    // (f) exactly two change_pct screens, opposite directions.
    let change: Vec<&CustomScreen> = screens
        .iter()
        .filter(|s| s.sort_col == "change_pct")
        .collect();
    assert_eq!(change.len(), 2, "expected exactly two change_pct screens");
    assert!(change.iter().any(|s| s.sort_ascending));
    assert!(change.iter().any(|s| !s.sort_ascending));

    // (g)(h) every sort column and tab stays inside the page's known key sets.
    for s in &screens {
        assert!(
            SORT_COLS.contains(&s.sort_col.as_str()),
            "{} sorts by unknown column {}",
            s.id,
            s.sort_col
        );
        assert!(
            TABS.contains(&s.tab.as_str()),
            "{} uses unknown tab {}",
            s.id,
            s.tab
        );
    }
}

#[test]
fn t6_upsert_replaces_in_place_and_appends_new() {
    let mut screens = vec![screen("a"), screen("b"), screen("c")];

    let mut replacement = screen("b");
    replacement.name = "Replaced B".to_string();
    upsert_screen(&mut screens, replacement);

    assert_eq!(screens.len(), 3, "upsert must not grow the list");
    assert_eq!(screens[0].id, "a");
    assert_eq!(screens[1].id, "b");
    assert_eq!(screens[1].name, "Replaced B");
    assert_eq!(screens[2].id, "c");

    upsert_screen(&mut screens, screen("d"));
    assert_eq!(screens.len(), 4, "unknown id must append");
    assert_eq!(screens[3].id, "d");
}

#[test]
fn t7_delete_removes_only_target_id() {
    let mut screens = vec![screen("a"), screen("b"), screen("c")];
    delete_screen(&mut screens, "b");
    let ids: Vec<&str> = screens.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, vec!["a", "c"]);

    delete_screen(&mut screens, "missing");
    assert_eq!(screens.len(), 2, "deleting an absent id is a no-op");
}

#[test]
fn t8_new_screen_id_unique_and_non_empty() {
    let first = new_screen_id();
    let second = new_screen_id();
    assert!(!first.is_empty());
    assert!(!second.is_empty());
    assert_ne!(first, second, "screen ids must be distinct");
}

#[test]
fn t9_snapshot_roundtrips_through_serde_json() {
    let criteria = FilterCriteria {
        max_pe: dec!(20.0),
        min_rvol: dec!(2),
        ..FilterCriteria::default()
    };
    let original = snapshot_screen(
        "My Screen",
        "A saved screen",
        "valuation",
        "pe_ratio",
        true,
        criteria,
        &["price".to_string(), "pe_ratio".to_string()],
    );
    assert!(!original.id.is_empty());

    let json = serde_json::to_string(&original).expect("serialize CustomScreen");
    let restored: CustomScreen = serde_json::from_str(&json).expect("deserialize CustomScreen");

    assert_eq!(restored.id, original.id);
    assert_eq!(restored.name, "My Screen");
    assert_eq!(restored.description, "A saved screen");
    assert_eq!(restored.tab, "valuation");
    assert_eq!(restored.sort_col, "pe_ratio");
    assert!(restored.sort_ascending);
    assert_eq!(restored.criteria.max_pe, dec!(20.0));
    assert_eq!(restored.criteria.min_rvol, dec!(2));
    assert_eq!(
        restored.criteria.market_cap_category,
        original.criteria.market_cap_category
    );
    assert_eq!(restored.visible_columns, original.visible_columns);
    assert_eq!(restored.visible_columns.len(), 2);
}
