//! Screen catalog & persistence (screener-design-doc.md §5.2): the
//! built-in screen library honba-fe's `OpenScreenModal` offers — grouped
//! Popular / Fundamental / Valuation / Technicals — plus user screens
//! kept as JSON under one storage key. Persistence goes through
//! `nsh_core::storage::Storage` (browser localStorage on wasm, the
//! platform keeper on native); an unreadable payload degrades to zero
//! custom screens, never a panic.

use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::features::screener::types::{
    FilterCriteria, MarketCapTier, QuickPreset, ScreenerAssetType, ScreenerViewTab,
};

/// One screen: a name, a description of what it isolates, and the state
/// it restores — asset class, view tab, filter criteria and the visible
/// column ids. Built-ins and user screens share one shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomScreen {
    pub id: String,
    pub name: String,
    pub description: String,
    pub asset_type: ScreenerAssetType,
    pub tab: ScreenerViewTab,
    /// Optional top-shelf preset the screen also arms (`None` = All).
    #[serde(default)]
    pub preset: Option<QuickPreset>,
    pub criteria: FilterCriteria,
    /// Empty = the tab's default column set.
    #[serde(default)]
    pub visible_columns: Vec<String>,
}

impl CustomScreen {
    fn from(
        id: &'static str,
        name: &'static str,
        description: &'static str,
        asset_type: ScreenerAssetType,
        tab: ScreenerViewTab,
        preset: Option<QuickPreset>,
        criteria: FilterCriteria,
    ) -> CustomScreen {
        CustomScreen {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            asset_type,
            tab,
            preset,
            criteria,
            visible_columns: Vec::new(),
        }
    }
}

/// Tier-only criteria builder so the catalog rows read plainly.
fn tier_criteria(tier: MarketCapTier) -> FilterCriteria {
    FilterCriteria {
        market_cap_tier: tier,
        ..FilterCriteria::default()
    }
}

/// The built-in screen catalog (design doc §5.2's shelves).
pub fn builtin_screens() -> Vec<CustomScreen> {
    vec![
        CustomScreen::from(
            "builtin.all",
            "All Stocks",
            "The complete 30-instrument equity fixture universe.",
            ScreenerAssetType::Stocks,
            ScreenerViewTab::Overview,
            Some(QuickPreset::All),
            FilterCriteria::default(),
        ),
        CustomScreen::from(
            "builtin.gainers",
            "Top Gainers",
            "Intraday leaders: day change strictly above zero.",
            ScreenerAssetType::Stocks,
            ScreenerViewTab::Performance,
            Some(QuickPreset::Gainers),
            FilterCriteria::default(),
        ),
        CustomScreen::from(
            "builtin.losers",
            "Top Losers",
            "Intraday laggards: day change below zero.",
            ScreenerAssetType::Stocks,
            ScreenerViewTab::Performance,
            Some(QuickPreset::Losers),
            FilterCriteria::default(),
        ),
        CustomScreen::from(
            "builtin.capitalized",
            "Most Capitalized",
            "Mega caps: ₹2,00,000 Cr and above by market value.",
            ScreenerAssetType::Stocks,
            ScreenerViewTab::Overview,
            Some(QuickPreset::MostCapitalized),
            tier_criteria(MarketCapTier::Mega),
        ),
        CustomScreen::from(
            "builtin.volume",
            "Volume Leaders",
            "Turnover breadth: 20,00,000 shares traded or more.",
            ScreenerAssetType::Stocks,
            ScreenerViewTab::Overview,
            Some(QuickPreset::MostActive),
            FilterCriteria {
                min_volume: Some(2_000_000),
                ..FilterCriteria::default()
            },
        ),
        CustomScreen::from(
            "builtin.dividend",
            "High Dividend Yield (>2%)",
            "Income candidates: dividend yield above 2%.",
            ScreenerAssetType::Stocks,
            ScreenerViewTab::Dividends,
            None,
            FilterCriteria {
                min_dividend: Some(dec!(2.0)),
                ..FilterCriteria::default()
            },
        ),
        CustomScreen::from(
            "builtin.compounders",
            "Consistent Compounders (ROE > 20%)",
            "Quality screens: return on equity above 20%.",
            ScreenerAssetType::Stocks,
            ScreenerViewTab::Profitability,
            None,
            FilterCriteria {
                min_roe: Some(dec!(20.0)),
                ..FilterCriteria::default()
            },
        ),
        CustomScreen::from(
            "builtin.value",
            "Value Stocks (P/E < 20, P/B < 3)",
            "Cheap against earnings and book: P/E under 20, P/B under 3.",
            ScreenerAssetType::Stocks,
            ScreenerViewTab::Valuation,
            None,
            FilterCriteria {
                max_pe: Some(dec!(20.0)),
                max_pb: Some(dec!(3.0)),
                ..FilterCriteria::default()
            },
        ),
        CustomScreen::from(
            "builtin.fcf",
            "High Free Cash Flow Yield",
            "Cash-rich economics: FCF yield above 3%.",
            ScreenerAssetType::Stocks,
            ScreenerViewTab::CashFlow,
            None,
            FilterCriteria {
                min_fcf_yield: Some(dec!(3.0)),
                ..FilterCriteria::default()
            },
        ),
        CustomScreen::from(
            "builtin.momentum",
            "Bullish Momentum (RSI > 55)",
            "RSI above 55, trend confirmation via the SMA 200 column.",
            ScreenerAssetType::Stocks,
            ScreenerViewTab::Technicals,
            Some(QuickPreset::Momentum),
            FilterCriteria {
                min_rsi: Some(dec!(55.0)),
                ..FilterCriteria::default()
            },
        ),
        CustomScreen::from(
            "builtin.oversold",
            "Oversold Bounce Candidates (RSI < 30)",
            "Washed-out books: RSI below 30.",
            ScreenerAssetType::Stocks,
            ScreenerViewTab::Technicals,
            None,
            FilterCriteria {
                max_rsi: Some(dec!(30.0)),
                ..FilterCriteria::default()
            },
        ),
        CustomScreen::from(
            "builtin.52w",
            "52W Breakouts",
            "Trading within 5% of the 52-week high (preset: High 52W).",
            ScreenerAssetType::Stocks,
            ScreenerViewTab::Performance,
            Some(QuickPreset::High52),
            FilterCriteria::default(),
        ),
    ]
}

/// The built-ins grouped for the Open Screen modal's shelves.
pub fn builtin_groups() -> Vec<(&'static str, Vec<CustomScreen>)> {
    let all = builtin_screens();
    let group = |ids: &[&str]| -> Vec<CustomScreen> {
        all.iter()
            .filter(|screen| ids.contains(&screen.id.as_str()))
            .cloned()
            .collect()
    };
    vec![
        (
            "Popular",
            group(&[
                "builtin.all",
                "builtin.gainers",
                "builtin.losers",
                "builtin.capitalized",
                "builtin.volume",
            ]),
        ),
        (
            "Fundamental",
            group(&["builtin.dividend", "builtin.compounders"]),
        ),
        ("Valuation", group(&["builtin.value", "builtin.fcf"])),
        (
            "Technicals",
            group(&["builtin.momentum", "builtin.oversold", "builtin.52w"]),
        ),
    ]
}

/// The storage key every custom screen list lives under.
const SCREENS_KEY: &str = "screener.screens";

/// A fresh id for a saved screen (user data, not fixture determinism).
pub fn new_screen_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// The user's saved screens; an unreadable payload reads as none.
pub fn load_custom_screens() -> Vec<CustomScreen> {
    nsh_core::storage::Storage::get_item(SCREENS_KEY)
        .and_then(
            |raw| match serde_json::from_str::<Vec<CustomScreen>>(&raw) {
                Ok(screens) => Some(screens),
                Err(_) => {
                    tracing::warn!("screener screens: stored JSON unreadable; starting fresh");
                    None
                }
            },
        )
        .unwrap_or_default()
}

/// Replaces the user's saved screens with `screens` (the modal edits a
/// working list and saves it whole).
pub fn save_custom_screens(screens: &[CustomScreen]) {
    match serde_json::to_string(screens) {
        Ok(json) => {
            if let Err(full) = nsh_core::storage::Storage::set_item(SCREENS_KEY, json) {
                tracing::warn!(bytes = full.bytes, "screener screens: storage full");
            }
        }
        Err(_) => tracing::warn!("screener screens: JSON serialization refused"),
    }
}

/// Adds or replaces (by id) one screen and persists.
pub fn upsert_screen(screen: CustomScreen) {
    let mut screens = load_custom_screens();
    match screens.iter_mut().find(|slot| slot.id == screen.id) {
        Some(slot) => *slot = screen,
        None => screens.push(screen),
    }
    save_custom_screens(&screens);
}

/// Removes one screen by id and persists.
pub fn delete_screen(id: &str) {
    let screens: Vec<CustomScreen> = load_custom_screens()
        .into_iter()
        .filter(|s| s.id != id)
        .collect();
    save_custom_screens(&screens);
}

/// Shift+N's Save flow: snapshots the page's live state into a named
/// screen (the caller passes the page's signals' current values).
pub fn snapshot_screen(
    name: &str,
    description: &str,
    asset_type: ScreenerAssetType,
    tab: ScreenerViewTab,
    preset: Option<QuickPreset>,
    criteria: FilterCriteria,
    visible_columns: Vec<String>,
) -> CustomScreen {
    CustomScreen {
        id: new_screen_id(),
        name: name.to_string(),
        description: description.to_string(),
        asset_type,
        tab,
        preset,
        criteria,
        visible_columns,
    }
}
