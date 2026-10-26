//! Screener store — the Leptos translation of the labs app's
//! `core/store/use-screener-store.ts` (Zustand) over the bundled instrument
//! dataset: the reactive signal state, the screen catalog (`BUILTIN_SCREENS`
//! + custom screens in localStorage) and the CSV download. The static
//! registries live in [`super::columns`]; filtering, sorting and the CSV
//! body live in [`super::filter_engine`].

use std::sync::OnceLock;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

use super::columns::{ColumnDef, ScreenerType, columns_for_screener, preset_for_tab};
use super::filter_engine::{
    ALL, AdvancedFilters, SortDirection, build_csv, filter_instruments, normalize_filters,
    sort_instruments,
};
use crate::features::screener::data::instruments::{self, Instrument};

// ---------------------------------------------------------------------------
// Screen catalog (ScreenDefinition + BUILTIN_SCREENS)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScreenDefinition {
    pub id: String,
    pub name: String,
    pub screener_type: ScreenerType,
    pub category: String,
    pub description: String,
    pub is_builtin: bool,
    pub filters: AdvancedFilters,
    pub quick_preset: String,
    pub sort_field: String,
    pub sort_direction: SortDirection,
}

impl Default for ScreenDefinition {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            screener_type: ScreenerType::Stocks,
            category: "custom".to_string(),
            description: String::new(),
            is_builtin: false,
            filters: AdvancedFilters::default(),
            quick_preset: "all".to_string(),
            sort_field: "marketCap".to_string(),
            sort_direction: SortDirection::Desc,
        }
    }
}

/// Built-in TradingView-style screens (`BUILTIN_SCREENS`). Filters unset in
/// the labs app inherit `DEFAULT_ADVANCED_FILTERS` when applied.
fn builtin_screens() -> Vec<ScreenDefinition> {
    use ScreenerType::*;

    let none = AdvancedFilters::new();

    macro_rules! screen {
        ($id:expr, $name:expr, $ty:expr, $cat:expr, $desc:expr, $filters:expr, $preset:expr, $sort:expr, $dir:expr) => {
            ScreenDefinition {
                id: $id.to_string(),
                name: $name.to_string(),
                screener_type: $ty,
                category: $cat.to_string(),
                description: $desc.to_string(),
                is_builtin: true,
                filters: AdvancedFilters {
                    sector: ALL.into(),
                    market_cap_tier: ALL.into(),
                    exchange: ALL.into(),
                    technical_rating: ALL.into(),
                    asset_class: ALL.into(),
                    focus: ALL.into(),
                    brand: ALL.into(),
                    issuer_type: ALL.into(),
                    issuer: ALL.into(),
                    coupon_type: ALL.into(),
                    credit_rating: ALL.into(),
                    mf_category: ALL.into(),
                    mf_scheme_type: ALL.into(),
                    riskometer: ALL.into(),
                    plan_type: ALL.into(),
                    ..$filters
                },
                quick_preset: $preset.to_string(),
                sort_field: $sort.to_string(),
                sort_direction: $dir,
            }
        };
    }

    vec![
        // ===== Stocks =====
        screen!(
            "stock_all",
            "All stocks",
            Stocks,
            "popular",
            "Complete list of stocks by market cap",
            none.clone(),
            "all",
            "marketCap",
            SortDirection::Desc
        ),
        screen!(
            "stock_most_capitalized",
            "Most capitalized",
            Stocks,
            "popular",
            "Largest companies by market cap",
            AdvancedFilters {
                market_cap_tier: "mega".into(),
                ..none.clone()
            },
            "all",
            "marketCap",
            SortDirection::Desc
        ),
        screen!(
            "stock_highest_net_income",
            "Highest net income",
            Stocks,
            "popular",
            "Stocks raking in the cash with strong net margins",
            AdvancedFilters {
                min_net_margin: Some(15.0),
                ..none.clone()
            },
            "all",
            "marketCap",
            SortDirection::Desc
        ),
        screen!(
            "stock_highest_cash",
            "Highest cash",
            Stocks,
            "popular",
            "Companies with industry-leading balance sheet resilience",
            AdvancedFilters {
                min_roce: Some(18.0),
                ..none.clone()
            },
            "all",
            "marketCap",
            SortDirection::Desc
        ),
        screen!(
            "stock_close_to_high_undervalued",
            "Close to new-high and undervalued",
            Stocks,
            "popular",
            "Stocks breaking new local high backed by attractive valuation",
            AdvancedFilters {
                near_52_week_high: true,
                pe_max: Some(25.0),
                ..none.clone()
            },
            "all",
            "changePercent",
            SortDirection::Desc
        ),
        screen!(
            "stock_healthy_growth_tech",
            "Healthy growth technology stocks",
            Stocks,
            "fundamental",
            "Tech stocks growing quickly and not fueled by cash burning",
            AdvancedFilters {
                sector: "Technology".into(),
                min_net_margin: Some(12.0),
                ..none.clone()
            },
            "all",
            "marketCap",
            SortDirection::Desc
        ),
        screen!(
            "stock_high_capital_efficiency",
            "High capital efficiency stocks",
            Stocks,
            "fundamental",
            "Stocks with industry-high ROCE and capital efficiency ratios",
            AdvancedFilters {
                min_roce: Some(22.0),
                ..none.clone()
            },
            "all",
            "marketCap",
            SortDirection::Desc
        ),
        screen!(
            "stock_strong_small_caps",
            "Strong performance small caps",
            Stocks,
            "fundamental",
            "Small-cap stocks with strong fundamentals and high momentum",
            AdvancedFilters {
                market_cap_tier: "small".into(),
                technical_rating: "Buy".into(),
                ..none.clone()
            },
            "all",
            "changePercent",
            SortDirection::Desc
        ),
        screen!(
            "stock_dividend_kings",
            "Dividend kings",
            Stocks,
            "valuation",
            "Predictable and high dividend yield stocks for resilient portfolios",
            AdvancedFilters {
                min_dividend_yield: Some(2.5),
                ..none.clone()
            },
            "dividend",
            "dividendYield",
            SortDirection::Desc
        ),
        screen!(
            "stock_bullish_undervalued",
            "Bullish and undervalued",
            Stocks,
            "technical",
            "Technically strong buy candidates trading below reasonable multiples",
            AdvancedFilters {
                technical_rating: "Buy".into(),
                pe_max: Some(22.0),
                ..none.clone()
            },
            "momentum",
            "changePercent",
            SortDirection::Desc
        ),
        // ===== ETF =====
        screen!(
            "etf_vault",
            "ETF vault",
            Etf,
            "popular",
            "All exchange-traded funds ranked by assets under management",
            none.clone(),
            "all",
            "marketCap",
            SortDirection::Desc
        ),
        screen!(
            "etf_lowest_expense",
            "Ultra-low cost index ETFs",
            Etf,
            "popular",
            "Institutional-grade ETFs with minimal expense ratios (<0.20%)",
            AdvancedFilters {
                max_expense_ratio: Some(0.20),
                ..none.clone()
            },
            "all",
            "marketCap",
            SortDirection::Desc
        ),
        screen!(
            "etf_tech_growth",
            "Broad-market & sectoral ETF trackers",
            Etf,
            "fundamental",
            "Wide-coverage listed trackers: index funds, commodity ETFs and diversified schemes",
            AdvancedFilters { ..none.clone() },
            "all",
            "changePercent",
            SortDirection::Desc
        ),
        screen!(
            "etf_dividend_income",
            "High yield dividend ETFs",
            Etf,
            "valuation",
            "Exchange-traded funds optimized for quarterly cash distributions",
            AdvancedFilters {
                min_dividend_yield: Some(3.0),
                ..none.clone()
            },
            "all",
            "dividendYield",
            SortDirection::Desc
        ),
        screen!(
            "etf_commodity_gold",
            "Gold & precious metals trackers",
            Etf,
            "popular",
            "Physical and synthetic bullion ETFs protecting purchasing power",
            AdvancedFilters {
                asset_class: "Commodities".into(),
                ..none.clone()
            },
            "all",
            "marketCap",
            SortDirection::Desc
        ),
        // ===== Bonds =====
        screen!(
            "bond_all",
            "All bonds",
            Bonds,
            "popular",
            "Fixed income instruments and debt securities",
            none.clone(),
            "all",
            "pe",
            SortDirection::Desc
        ),
        screen!(
            "bond_sovereign",
            "Sovereign & benchmark gilts",
            Bonds,
            "popular",
            "Government backed risk-free treasury debt across 5Y, 10Y, and 30Y tenors",
            AdvancedFilters {
                issuer_type: "Government Bonds".into(),
                ..none.clone()
            },
            "all",
            "pe",
            SortDirection::Desc
        ),
        screen!(
            "bond_aaa_corporate",
            "AAA prime corporate bonds",
            Bonds,
            "fundamental",
            "Highest credit rating corporate paper with attractive spread over G-Sec",
            AdvancedFilters {
                credit_rating: "AAA".into(),
                ..none.clone()
            },
            "all",
            "pe",
            SortDirection::Desc
        ),
        // ===== MF =====
        screen!(
            "mf_all",
            "All mutual funds",
            Mf,
            "popular",
            "Comprehensive directory of equity, debt, and hybrid mutual fund schemes",
            none.clone(),
            "all",
            "marketCap",
            SortDirection::Desc
        ),
        screen!(
            "mf_consistent_large_cap",
            "Top 5Y CAGR large cap funds",
            Mf,
            "fundamental",
            "Consistently outperforming tier-1 funds with >15% annualized 5Y CAGR",
            AdvancedFilters {
                mf_category: "Equity: Large Cap".into(),
                min_cagr5y: Some(15.0),
                ..none.clone()
            },
            "all",
            "changePercent",
            SortDirection::Desc
        ),
        screen!(
            "mf_consistent_cagr",
            "High 3Y CAGR schemes",
            Mf,
            "fundamental",
            "Funds whose 3-year CAGR clearance (a proxy from 1Y return) lifts them above the pack",
            AdvancedFilters {
                min_cagr3y: Some(12.0),
                ..none.clone()
            },
            "all",
            "marketCap",
            SortDirection::Desc
        ),
    ]
}

/// The readonly built-in catalog.
pub fn builtin_screens_registry() -> &'static [ScreenDefinition] {
    static REGISTRY: OnceLock<Vec<ScreenDefinition>> = OnceLock::new();
    REGISTRY.get_or_init(builtin_screens)
}

// ---------------------------------------------------------------------------
// Custom screens persistence (`honba_custom_screens_v1`)
// ---------------------------------------------------------------------------

const CUSTOM_SCREENS_KEY: &str = "honba_custom_screens_v1";

fn storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|window| window.local_storage().ok().flatten())
}

fn load_custom_screens() -> Vec<ScreenDefinition> {
    storage()
        .and_then(|storage| storage.get_item(CUSTOM_SCREENS_KEY).ok().flatten())
        .and_then(|raw| serde_json::from_str::<Vec<ScreenDefinition>>(&raw).ok())
        .unwrap_or_default()
}

fn save_custom_screens(screens: &[ScreenDefinition]) {
    if let Some(storage) = storage() {
        if let Ok(json) = serde_json::to_string(screens) {
            let _ = storage.set_item(CUSTOM_SCREENS_KEY, &json);
        }
    }
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

/// The screener reactive state. Provided on the screener page scope.
#[derive(Clone, Copy)]
pub struct ScreenerStore {
    pub market: RwSignal<&'static str>,
    pub screener_type: RwSignal<ScreenerType>,
    pub search_query: RwSignal<String>,
    pub quick_preset: RwSignal<String>,
    pub advanced_filters: RwSignal<AdvancedFilters>,
    pub active_symbol: RwSignal<String>,
    pub shortlist: RwSignal<Vec<String>>,
    pub active_tab: RwSignal<&'static str>,
    pub visible_columns: RwSignal<Vec<String>>,
    pub sort_field: RwSignal<String>,
    pub sort_order: RwSignal<Option<SortDirection>>,
    pub page: RwSignal<usize>,
    pub page_size: RwSignal<usize>,
    pub view_mode: RwSignal<ViewMode>,
    pub custom_screens: RwSignal<Vec<ScreenDefinition>>,
    pub open_screen_modal: RwSignal<bool>,
    pub filter_modal: RwSignal<bool>,
    pub column_modal: RwSignal<bool>,
    pub detail_drawer_open: RwSignal<bool>,
    pub detail_tab: RwSignal<DetailTab>,
    pub screen_title: RwSignal<String>,
    pub screen_id: RwSignal<String>,
    /// Live-tick toggle indicator (badge only — lab app's simulation lives).
    pub is_live: RwSignal<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Table,
    Heatmap,
    FactorHeatmap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DetailTab {
    #[default]
    Overview,
    Technicals,
    Financials,
}

impl ScreenerStore {
    pub fn new() -> Self {
        let preset: Vec<String> = preset_for_tab(ScreenerType::Stocks, "overview")
            .unwrap_or_default()
            .into_iter()
            .map(str::to_string)
            .collect();

        Self {
            market: RwSignal::new("IN"),
            screener_type: RwSignal::new(ScreenerType::Stocks),
            search_query: RwSignal::new(String::new()),
            quick_preset: RwSignal::new("all".to_string()),
            advanced_filters: RwSignal::new(AdvancedFilters::default_filters()),
            active_symbol: RwSignal::new(String::new()),
            shortlist: RwSignal::new(Vec::new()),
            active_tab: RwSignal::new("overview"),
            visible_columns: RwSignal::new(preset),
            sort_field: RwSignal::new("marketCap".to_string()),
            sort_order: RwSignal::new(Some(SortDirection::Desc)),
            page: RwSignal::new(1),
            page_size: RwSignal::new(100),
            view_mode: RwSignal::new(ViewMode::Table),
            custom_screens: RwSignal::new(load_custom_screens()),
            open_screen_modal: RwSignal::new(false),
            filter_modal: RwSignal::new(false),
            column_modal: RwSignal::new(false),
            detail_drawer_open: RwSignal::new(false),
            detail_tab: RwSignal::new(DetailTab::Overview),
            screen_title: RwSignal::new(ScreenerType::Stocks.default_title().to_string()),
            screen_id: RwSignal::new("stocks_all".to_string()),
            is_live: RwSignal::new(true),
        }
    }

    pub fn provide(self) {
        provide_context(self);
    }

    /// Fetch from context (defined at `crate::features::screener::screener_page`).
    pub fn take() -> Self {
        use_context().expect("ScreenerStore: provide it above the component")
    }

    /// Instruments for the current market + screener type.
    pub fn universe(&self) -> Vec<&'static Instrument> {
        instruments::universe_for_screener(self.market.get(), self.screener_type.get().as_id())
    }

    /// The filtered rows (search + quick preset + advanced filters).
    pub fn filtered(&self) -> Memo<Vec<&'static Instrument>> {
        let store = *self;
        Memo::new(move |_| {
            filter_instruments(
                &store.universe(),
                &store.search_query.get(),
                &store.quick_preset.get(),
                &store.advanced_filters.get(),
                &store.shortlist.get(),
            )
        })
    }

    /// Filtered + sorted, without pagination (the canonical row order).
    pub fn sorted(&self) -> Memo<Vec<&'static Instrument>> {
        let filtered = self.filtered();
        let store = *self;
        Memo::new(move |_| {
            let mut rows = filtered.get();
            if let Some(direction) = store.sort_order.get() {
                sort_instruments(&mut rows, &store.sort_field.get(), direction);
            }
            rows
        })
    }

    /// The visible page slice.
    pub fn page_rows(&self) -> Memo<Vec<&'static Instrument>> {
        let sorted = self.sorted();
        let store = *self;
        Memo::new(move |_| {
            let rows = sorted.get();
            let size = store.page_size.get();
            if size == 0 || size >= rows.len() {
                return rows;
            }
            let start = store.page.get().saturating_sub(1) * size;
            rows.get(start..).unwrap_or(&[]).to_vec()
        })
    }

    // -- Actions -------------------------------------------------------------

    pub fn set_market(&self, market: &'static str) {
        self.market.set(market);
        self.page.set(1);
        if let Some(first) = self.universe().first() {
            self.active_symbol.set(first.symbol.clone());
        }
    }

    /// Switch asset class: resets filters/tab/columns to the class defaults.
    pub fn set_screener_type(&self, screener_type: ScreenerType) {
        let config_tabs = screener_type.tabs();
        let tab = config_tabs.first().map_or("overview", |(id, _)| *id);
        let preset: Vec<String> = preset_for_tab(screener_type, tab)
            .unwrap_or_else(|| {
                screener_type
                    .tabs()
                    .first()
                    .map_or(Vec::new(), |_| Vec::new())
            })
            .into_iter()
            .map(str::to_string)
            .collect();

        self.screener_type.set(screener_type);
        self.screen_title
            .set(screener_type.default_title().to_string());
        self.screen_id.set(format!("{}_all", screener_type.as_id()));
        self.quick_preset.set("all".to_string());
        self.advanced_filters
            .set(AdvancedFilters::default_filters());
        self.active_tab.set(tab);
        self.visible_columns.set(preset);
        let (field, direction) = screener_type.default_sort();
        self.sort_field.set(field.to_string());
        self.sort_order.set(Some(direction));
        self.page.set(1);
        if let Some(first) = self.universe().first() {
            self.active_symbol.set(first.symbol.clone());
        }
    }

    pub fn set_search_query(&self, query: String) {
        self.search_query.set(query);
        self.page.set(1);
    }

    pub fn set_active_tab(&self, tab: &'static str) {
        self.active_tab.set(tab);
        if let Some(preset) = preset_for_tab(self.screener_type.get_untracked(), tab) {
            self.visible_columns
                .set(preset.into_iter().map(str::to_string).collect());
        }
        self.page.set(1);
    }

    pub fn set_quick_preset(&self, preset: &str) {
        let screener_type = self.screener_type.get_untracked();
        let matching = builtin_screens_registry()
            .iter()
            .find(|screen| screen.screener_type == screener_type && screen.quick_preset == preset);
        self.quick_preset.set(preset.to_string());
        self.page.set(1);
        if let Some(screen) = matching {
            self.screen_title.set(screen.name.clone());
            self.screen_id.set(screen.id.clone());
        }
    }

    pub fn set_advanced_filters(&self, filters: AdvancedFilters) {
        self.advanced_filters.set(filters);
        self.page.set(1);
    }

    pub fn reset_filters(&self) {
        self.quick_preset.set("all".to_string());
        self.screen_title.set(
            self.screener_type
                .get_untracked()
                .default_title()
                .to_string(),
        );
        self.search_query.set(String::new());
        self.advanced_filters
            .set(AdvancedFilters::default_filters());
        self.page.set(1);
    }

    pub fn toggle_shortlist(&self, symbol: &str) {
        self.shortlist.update(|list| {
            if list.iter().any(|entry| entry == symbol) {
                list.retain(|entry| entry != symbol);
            } else {
                list.push(symbol.to_string());
            }
        });
    }

    pub fn clear_shortlist(&self) {
        self.shortlist.set(Vec::new());
    }

    pub fn set_symbol(&self, symbol: &str) {
        self.active_symbol.set(symbol.to_uppercase());
        self.detail_drawer_open.set(true);
    }

    /// Sort toggle across fields (desc → asc → none), the labs `setSort`.
    pub fn set_sort(&self, field: &str) {
        if self.sort_field.get_untracked() == field {
            match self.sort_order.get_untracked() {
                Some(SortDirection::Desc) => self.sort_order.set(Some(SortDirection::Asc)),
                Some(SortDirection::Asc) => {
                    self.sort_order.set(None);
                    self.sort_field.set(String::new());
                }
                None => self.sort_order.set(Some(SortDirection::Desc)),
            }
        } else {
            self.sort_field.set(field.to_string());
            self.sort_order.set(Some(SortDirection::Desc));
        }
        self.page.set(1);
    }

    pub fn set_page(&self, page: usize) {
        self.page.set(page.max(1));
    }

    pub fn set_page_size(&self, size: usize) {
        self.page_size.set(size);
        self.page.set(1);
    }

    // -- Screen catalog --------------------------------------------------------

    pub fn toggle_open_screen_modal(&self, open: bool) {
        self.open_screen_modal.set(open);
    }

    /// Apply a screen from the catalog (builtin or custom).
    pub fn select_screen(&self, screen: &ScreenDefinition) {
        self.screener_type.set(screen.screener_type);
        self.screen_title.set(screen.name.clone());
        self.screen_id.set(screen.id.clone());
        self.quick_preset.set(screen.quick_preset.clone());
        self.advanced_filters
            .set(normalize_filters(&screen.filters));
        self.sort_field.set(screen.sort_field.clone());
        self.sort_order.set(Some(screen.sort_direction));
        // A screen may jump asset classes: its old class' columns and tab do
        // not exist on the new table, so reseed both from the class defaults.
        let tab = screen
            .screener_type
            .tabs()
            .first()
            .map_or("overview", |(id, _)| *id);
        self.active_tab.set(tab);
        let preset = preset_for_tab(screen.screener_type, tab)
            .unwrap_or_default()
            .into_iter()
            .map(str::to_string)
            .collect();
        self.visible_columns.set(preset);
        self.open_screen_modal.set(false);
        self.page.set(1);
    }

    /// Save the current filter/sort state as a new custom screen.
    pub fn save_current_as_new_screen(&self, name: &str) {
        let id = format!("custom_{}", web_time_now());
        let screen = ScreenDefinition {
            id,
            name: if name.trim().is_empty() {
                "Untitled Screen".to_string()
            } else {
                name.trim().to_string()
            },
            screener_type: self.screener_type.get_untracked(),
            category: "custom".to_string(),
            description: "User customized screen".to_string(),
            is_builtin: false,
            filters: self.advanced_filters.get_untracked(),
            quick_preset: self.quick_preset.get_untracked(),
            sort_field: self.sort_field.get_untracked(),
            sort_direction: self
                .sort_order
                .get_untracked()
                .unwrap_or(SortDirection::Desc),
        };
        self.custom_screens
            .update(|screens| screens.insert(0, screen.clone()));
        save_custom_screens(&self.custom_screens.get_untracked());
        self.screen_title.set(screen.name.clone());
        self.screen_id.set(screen.id.clone());
    }

    pub fn delete_custom_screen(&self, screen_id: &str) {
        self.custom_screens.update(|screens| {
            screens.retain(|screen| screen.id != screen_id);
        });
        save_custom_screens(&self.custom_screens.get_untracked());
        if self.screen_id.get_untracked() == screen_id {
            self.screen_title.set(
                self.screener_type
                    .get_untracked()
                    .default_title()
                    .to_string(),
            );
        }
    }

    // -- Columns --------------------------------------------------------------

    /// Column defs for the current type, flagged `visible` per the store list.
    pub fn columns(&self) -> Memo<Vec<(ColumnDef, bool)>> {
        let store = *self;
        Memo::new(move |_| {
            let visible = store.visible_columns.get();
            columns_for_screener(store.screener_type.get())
                .into_iter()
                .map(|column| {
                    let is_visible = visible.iter().any(|id| id == column.id);
                    (column, is_visible)
                })
                .collect()
        })
    }

    pub fn set_visible_columns(&self, ids: Vec<String>) {
        self.visible_columns.set(ids);
    }

    // -- Drawers + modals -------------------------------------------------------

    pub fn open_detail(&self, symbol: &str, tab: DetailTab) {
        self.active_symbol.set(symbol.to_string());
        self.detail_tab.set(tab);
        self.detail_drawer_open.set(true);
    }

    pub fn close_detail(&self) {
        self.detail_drawer_open.set(false);
    }

    pub fn toggle_live(&self) {
        self.is_live.update(|live| *live = !*live);
    }

    /// Download the current filtered set as CSV (data-URI anchor; avoids the
    /// optional web-sys `BlobPropertyBag` feature).
    pub fn export_csv(&self) {
        let rows = self.sorted().get_untracked();
        let csv = build_csv(&rows);
        let window = web_sys::window().expect("window for CSV export");
        let document = window.document().expect("document for CSV export");
        let url = format!("data:text/csv;charset=utf-8,{}", percent_encode(&csv));
        let link = document
            .create_element("a")
            .expect("anchor for CSV export")
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .expect("anchor element");
        link.set_href(&url);
        link.set_download(&format!("honba_screener_{}.csv", web_time_now()));
        let _ = link.click();
    }
}

impl Default for ScreenerStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal percent-encoding for `data:` URIs (CSV content is ASCII).
fn percent_encode(raw: &str) -> String {
    let mut encoded = String::with_capacity(raw.len());
    for byte in raw.as_bytes() {
        match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'.'
            | b'_'
            | b'~'
            | b','
            | b'\n'
            | b'\r'
            | b'('
            | b')'
            | b'!'
            | b'$'
            | b'*'
            | b'\'' => {
                encoded.push(*byte as char);
            }
            other => encoded.push_str(&format!("%{other:02X}")),
        }
    }
    encoded
}

/// Millisecond wall-clock timestamp for `custom_${Date.now()}` ids.
fn web_time_now() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_millis())
        .unwrap_or(1_700_000_000_001)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
}
