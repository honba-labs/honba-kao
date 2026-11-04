//! Screen catalog (screener-honba-leptos.md §7 `.`/Shift+N): built-in library plus user screens; custom screens snapshot the page's live criteria + sort + visible columns and reload through serde_json.

use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use super::types::FilterCriteria;

/// localStorage key backing the persisted custom screen list (wiring layer owns I/O).
pub const SCREENS_KEY: &str = "screener.screens";

/// A saved screener screen: tab + sort + criteria + optional visible columns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomScreen {
    /// Stable id (`builtin.*` for shipped screens, uuid v4 for user screens).
    pub id: String,
    /// Display name shown on the screen shelf.
    pub name: String,
    /// One-line description shown in the screen picker.
    pub description: String,
    /// Active feature tab (e.g. "overview").
    pub tab: String,
    /// Sort column key (a `filters.rs::sort_records` key).
    pub sort_col: String,
    /// Sort direction (`true` = ascending).
    pub sort_ascending: bool,
    /// Full filter criteria snapshot captured at save time.
    pub criteria: FilterCriteria,
    /// Explicit visible column order; empty = tab defaults.
    pub visible_columns: Vec<String>,
}

/// Builds a builtin screen entry with tab-default (empty) visible columns.
fn screen(
    id: &str,
    name: &str,
    description: &str,
    tab: &str,
    sort_col: &str,
    sort_ascending: bool,
    criteria: FilterCriteria,
) -> CustomScreen {
    CustomScreen {
        id: id.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        tab: tab.to_string(),
        sort_col: sort_col.to_string(),
        sort_ascending,
        criteria,
        visible_columns: Vec::new(),
    }
}

/// Returns the twelve shipped builtin screens in shelf display order.
pub fn builtin_screens() -> Vec<CustomScreen> {
    vec![
        screen(
            "builtin.all",
            "All Stocks",
            "The full instrument universe without a criteria filter.",
            "overview",
            "market_cap_cr",
            false,
            FilterCriteria::default(),
        ),
        screen(
            "builtin.gainers",
            "Top Gainers",
            "Intraday leaders sorted by day change descending.",
            "overview",
            "change_pct",
            false,
            FilterCriteria::default(),
        ),
        screen(
            "builtin.losers",
            "Top Losers",
            "Intraday laggards sorted by day change ascending.",
            "overview",
            "change_pct",
            true,
            FilterCriteria::default(),
        ),
        screen(
            "builtin.capitalized",
            "Most Capitalized",
            "Mega caps first: market cap descending.",
            "overview",
            "market_cap_cr",
            false,
            FilterCriteria::default(),
        ),
        screen(
            "builtin.value",
            "Value Stocks (P/E < 20)",
            "Cheapest on earnings: P/E under 20, cheapest first.",
            "valuation",
            "pe_ratio",
            true,
            FilterCriteria {
                max_pe: dec!(20.0),
                ..FilterCriteria::default()
            },
        ),
        screen(
            "builtin.momentum",
            "Bullish Momentum (RSI 55–70)",
            "RSI band 55 to 70 with healthiest momentum first.",
            "technicals",
            "rsi_14",
            false,
            FilterCriteria {
                min_rsi: dec!(55.0),
                max_rsi: dec!(70.0),
                ..FilterCriteria::default()
            },
        ),
        screen(
            "builtin.oversold",
            "Oversold Bounce (RSI < 30)",
            "Washed-out RSI territory under 30.",
            "technicals",
            "rsi_14",
            true,
            FilterCriteria {
                max_rsi: dec!(30.0),
                ..FilterCriteria::default()
            },
        ),
        screen(
            "builtin.volume",
            "High Volume Surge (RVOL ≥ 2)",
            "Relative volume twice the 20-day average.",
            "overview",
            "volume",
            false,
            FilterCriteria {
                min_rvol: dec!(2.0),
                ..FilterCriteria::default()
            },
        ),
        screen(
            "builtin.expensive",
            "Rich Multiples (P/E ≥ 40)",
            "Premium valuations at 40x earnings and above.",
            "valuation",
            "pe_ratio",
            false,
            FilterCriteria {
                min_pe: dec!(40.0),
                ..FilterCriteria::default()
            },
        ),
        screen(
            "builtin.fno",
            "F&O Universe",
            "Instruments carrying the derivatives badge.",
            "overview",
            "market_cap_cr",
            false,
            FilterCriteria {
                fno_only: true,
                ..FilterCriteria::default()
            },
        ),
        screen(
            "builtin.watchlist",
            "Watchlist Only",
            "Just the starred rows (★).",
            "overview",
            "market_cap_cr",
            false,
            FilterCriteria {
                watchlist_only: true,
                ..FilterCriteria::default()
            },
        ),
        screen(
            "builtin.small_chips",
            "Small Cap Chips",
            "Small-cap breadth under ₹100 per share.",
            "overview",
            "market_cap_cr",
            false,
            FilterCriteria {
                market_cap_category: "Small Cap".to_string(),
                max_price: dec!(100.0),
                ..FilterCriteria::default()
            },
        ),
    ]
}

/// Returns the shelf layout grouping every builtin exactly once.
pub fn builtin_groups() -> Vec<(&'static str, Vec<CustomScreen>)> {
    let screens = builtin_screens();
    let pick = |ids: &[&str]| -> Vec<CustomScreen> {
        ids.iter()
            .filter_map(|id| screens.iter().find(|s| s.id == *id).cloned())
            .collect()
    };
    vec![
        (
            "Popular",
            pick(&[
                "builtin.all",
                "builtin.gainers",
                "builtin.losers",
                "builtin.capitalized",
                "builtin.volume",
            ]),
        ),
        ("Value", pick(&["builtin.value", "builtin.expensive"])),
        ("Momentum", pick(&["builtin.momentum", "builtin.oversold"])),
        (
            "Universe",
            pick(&["builtin.fno", "builtin.watchlist", "builtin.small_chips"]),
        ),
    ]
}

/// Generates a fresh uuid v4 screen id for a user-created screen.
pub fn new_screen_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Captures the page's live tab, sort, criteria and visible columns into a new screen.
pub fn snapshot_screen(
    name: &str,
    description: &str,
    tab: &str,
    sort_col: &str,
    sort_ascending: bool,
    criteria: FilterCriteria,
    visible_columns: &[String],
) -> CustomScreen {
    CustomScreen {
        id: new_screen_id(),
        name: name.to_string(),
        description: description.to_string(),
        tab: tab.to_string(),
        sort_col: sort_col.to_string(),
        sort_ascending,
        criteria,
        visible_columns: visible_columns.to_vec(),
    }
}

/// Replaces the screen with the same id in place, else appends it.
pub fn upsert_screen(screens: &mut Vec<CustomScreen>, screen: CustomScreen) {
    match screens.iter_mut().find(|existing| existing.id == screen.id) {
        Some(slot) => *slot = screen,
        None => screens.push(screen),
    }
}

/// Removes the screen with the given id; other ids survive unchanged.
pub fn delete_screen(screens: &mut Vec<CustomScreen>, id: &str) {
    screens.retain(|s| s.id != id);
}

/// Serializes the screen list for localStorage, empty string on failure.
pub fn serialize_screens(screens: &[CustomScreen]) -> String {
    serde_json::to_string(screens).unwrap_or_default()
}

/// Deserializes the screen list from localStorage, empty list on failure.
pub fn deserialize_screens(json: &str) -> Vec<CustomScreen> {
    serde_json::from_str(json).unwrap_or_default()
}
