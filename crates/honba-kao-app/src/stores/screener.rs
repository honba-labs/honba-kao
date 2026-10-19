//! Screener store — the Leptos translation of the labs app's
//! `core/store/use-screener-store.ts` (Zustand) over the bundled instrument
//! dataset. Also carries the filter pipeline, quick presets, screen catalog
//! (`BUILTIN_SCREENS` + custom screens in localStorage), sorting special
//! cases and CSV export — everything client-side, exactly like the labs app.

use std::sync::OnceLock;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

use crate::data::indices::instrument_belongs_to_index;
use crate::data::instruments::{self, Instrument};

// ---------------------------------------------------------------------------
// Screener asset classes
// ---------------------------------------------------------------------------

/// `stocks | etf | bonds | mf` (`ScreenerType`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreenerType {
    Stocks,
    Etf,
    Bonds,
    #[serde(rename = "mf")]
    Mf,
}

pub const ALL_SCREENER_TYPES: [ScreenerType; 4] = [
    ScreenerType::Stocks,
    ScreenerType::Etf,
    ScreenerType::Bonds,
    ScreenerType::Mf,
];

impl ScreenerType {
    pub const fn as_id(self) -> &'static str {
        match self {
            ScreenerType::Stocks => "stocks",
            ScreenerType::Etf => "etf",
            ScreenerType::Bonds => "bonds",
            ScreenerType::Mf => "mf",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id {
            "etf" => ScreenerType::Etf,
            "bonds" => ScreenerType::Bonds,
            "mf" => ScreenerType::Mf,
            _ => ScreenerType::Stocks,
        }
    }

    /// `defaultTitle` — the "All stocks" / "ETF vault" strings.
    pub const fn default_title(self) -> &'static str {
        match self {
            ScreenerType::Stocks => "All stocks",
            ScreenerType::Etf => "ETF vault",
            ScreenerType::Bonds => "All bonds",
            ScreenerType::Mf => "All mutual funds",
        }
    }

    pub const fn stock_name(self) -> &'static str {
        match self {
            ScreenerType::Stocks => "Stock Screener",
            ScreenerType::Etf => "ETF Screener",
            ScreenerType::Bonds => "Bond Screener",
            ScreenerType::Mf => "MF Screener",
        }
    }

    /// `defaultSort`.
    pub const fn default_sort(self) -> (&'static str, SortDirection) {
        match self {
            ScreenerType::Stocks => ("marketCap", SortDirection::Desc),
            ScreenerType::Etf => ("marketCap", SortDirection::Desc),
            ScreenerType::Bonds => ("pe", SortDirection::Desc),
            ScreenerType::Mf => ("marketCap", SortDirection::Desc),
        }
    }

    /// `tabs` of the screener (`TabConfig[]`).
    pub fn tabs(self) -> Vec<(&'static str, &'static str)> {
        match self {
            ScreenerType::Stocks => vec![
                ("overview", "Overview"),
                ("performance", "Performance"),
                ("technicals", "Technicals"),
                ("extended_hours", "Extended hours"),
                ("forecasts", "Forecasts"),
                ("valuation", "Valuation"),
                ("dividends", "Dividends"),
                ("profitability", "Profitability"),
                ("income_statement", "Income statement"),
                ("balance_sheet", "Balance sheet"),
                ("cash_flow", "Cash flow"),
                ("per_share", "Per share"),
            ],
            ScreenerType::Etf => vec![
                ("overview", "Overview"),
                ("performance", "Performance"),
                ("technicals", "Technicals"),
                ("extended_hours", "Extended hours"),
                ("fund_flows", "Fund flows"),
                ("dividends", "Dividends"),
                ("nav_performance", "NAV performance"),
                ("holdings", "Holdings"),
                ("risk", "Risk"),
            ],
            ScreenerType::Bonds => vec![
                ("overview", "Overview"),
                ("security_info", "Security info"),
                ("interest_rate_risk", "Interest rate risk"),
                ("spreads", "Spreads"),
                ("amounts", "Amounts"),
                ("bond_features", "Bond features"),
                ("coupon", "Coupon"),
                ("issuer", "Issuer"),
                ("ratings", "Ratings"),
            ],
            ScreenerType::Mf => vec![
                ("overview", "Overview"),
                ("returns_cagr", "Returns & CAGR"),
                ("portfolio_holdings", "Portfolio Holdings"),
                ("sector_allocation", "Sector Allocation"),
                ("risk_ratios", "Risk & Ratios"),
                ("fees_loads", "Fees & Loads"),
            ],
        }
    }

    /// `pillConfigs` ids for this asset class.
    pub fn pills(self) -> Vec<(&'static str, &'static str)> {
        match self {
            ScreenerType::Stocks => vec![
                ("market", "Market"),
                ("watchlist", "Watchlist"),
                ("index", "Index"),
                ("price", "Price"),
                ("chg", "Chg %"),
                ("mktcap", "Mkt cap"),
                ("pe", "P/E"),
                ("epsGrowth", "EPS dil growth"),
                ("dividend", "Div yield %"),
                ("sector", "Sector"),
                ("rating", "Analyst rating"),
            ],
            ScreenerType::Etf => vec![
                ("market", "Market"),
                ("watchlist", "Watchlist"),
                ("assetClass", "Asset class"),
                ("focus", "Focus"),
                ("brand", "Brand / AMC"),
                ("aum", "AUM"),
                ("expenseRatio", "Expense ratio"),
                ("navReturn", "NAV return"),
                ("divYield", "Div yield %"),
                ("perf", "Perf %"),
            ],
            ScreenerType::Bonds => vec![
                ("watchlist", "Watchlist"),
                ("issuerType", "Issuer type"),
                ("issuer", "Issuer"),
                ("ytw", "YTW %"),
                ("coupon", "Coupon %"),
                ("couponType", "Coupon type"),
                ("maturity", "Maturity date"),
                ("rating", "Credit rating"),
            ],
            ScreenerType::Mf => vec![
                ("watchlist", "Watchlist"),
                ("schemeType", "Type (Liquid/Debt/Hybrid/Equity)"),
                ("category", "Category"),
                ("amc", "AMC / Fund House"),
                ("aum", "AUM"),
                ("cagr3y", "3Y Return %"),
                ("cagr5y", "5Y Return %"),
                ("expenseRatio", "Exp Ratio (Base+Total)"),
                ("sharpeRatio", "Sharpe ratio"),
                ("riskometer", "Riskometer"),
                ("plan", "Direct / Regular"),
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Filters (AdvancedFilterState + countActiveFilters)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AdvancedFilters {
    // Common / descriptive
    pub sector: String,
    pub market_cap_tier: String,
    pub exchange: String,
    pub indices: Vec<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub min_change_percent: Option<f64>,
    pub max_change_percent: Option<f64>,
    pub min_volume: Option<f64>,

    // Valuation & fundamentals (stocks)
    pub pe_min: Option<f64>,
    pub pe_max: Option<f64>,
    pub min_dividend_yield: Option<f64>,
    pub min_roce: Option<f64>,
    pub min_net_margin: Option<f64>,
    pub technical_rating: String,

    // Technical indicators
    pub rsi_min: Option<f64>,
    pub rsi_max: Option<f64>,
    pub price_above_200_sma: bool,
    pub price_above_50_sma: bool,
    pub near_52_week_high: bool,

    // ETF specific
    pub asset_class: String,
    pub focus: String,
    pub brand: String,
    pub min_aum: Option<f64>,
    pub max_aum: Option<f64>,
    pub max_expense_ratio: Option<f64>,
    pub min_nav_return: Option<f64>,

    // Bond specific
    pub issuer_type: String,
    pub issuer: String,
    pub min_ytw: Option<f64>,
    pub max_ytw: Option<f64>,
    pub min_coupon: Option<f64>,
    pub max_coupon: Option<f64>,
    pub coupon_type: String,
    pub maturity_year: Option<i64>,
    pub credit_rating: String,

    // Mutual fund specific
    pub mf_category: String,
    pub mf_scheme_type: String,
    pub min_cagr3y: Option<f64>,
    pub min_cagr5y: Option<f64>,
    pub base_expense_ratio: Option<f64>,
    pub total_expense_ratio: Option<f64>,
    pub cat_total_expense_ratio: Option<f64>,
    pub min_sharpe_ratio: Option<f64>,
    pub cat_sharpe_ratio: Option<f64>,
    pub riskometer: String,
    pub plan_type: String,
}

const ALL: &str = "all";

#[allow(dead_code)]
fn d_all() -> String {
    ALL.to_string()
}

#[allow(dead_code)]
fn d_indices() -> Vec<String> {
    Vec::new()
}

impl AdvancedFilters {
    pub const fn new() -> Self {
        Self {
            sector: String::new(),
            market_cap_tier: String::new(),
            exchange: String::new(),
            indices: Vec::new(),
            min_price: None,
            max_price: None,
            min_change_percent: None,
            max_change_percent: None,
            min_volume: None,
            pe_min: None,
            pe_max: None,
            min_dividend_yield: None,
            min_roce: None,
            min_net_margin: None,
            technical_rating: String::new(),
            rsi_min: None,
            rsi_max: None,
            price_above_200_sma: false,
            price_above_50_sma: false,
            near_52_week_high: false,
            asset_class: String::new(),
            focus: String::new(),
            brand: String::new(),
            min_aum: None,
            max_aum: None,
            max_expense_ratio: None,
            min_nav_return: None,
            issuer_type: String::new(),
            issuer: String::new(),
            min_ytw: None,
            max_ytw: None,
            min_coupon: None,
            max_coupon: None,
            coupon_type: String::new(),
            maturity_year: None,
            credit_rating: String::new(),
            mf_category: String::new(),
            mf_scheme_type: String::new(),
            min_cagr3y: None,
            min_cagr5y: None,
            base_expense_ratio: None,
            total_expense_ratio: None,
            cat_total_expense_ratio: None,
            min_sharpe_ratio: None,
            cat_sharpe_ratio: None,
            riskometer: String::new(),
            plan_type: String::new(),
        }
    }

    /// `DEFAULT_ADVANCED_FILTERS`.
    pub fn default_filters() -> Self {
        Self {
            sector: ALL.into(),
            market_cap_tier: ALL.into(),
            exchange: ALL.into(),
            indices: Vec::new(),
            min_price: None,
            max_price: None,
            min_change_percent: None,
            max_change_percent: None,
            min_volume: None,
            pe_min: None,
            pe_max: None,
            min_dividend_yield: None,
            min_roce: None,
            min_net_margin: None,
            technical_rating: ALL.into(),
            rsi_min: None,
            rsi_max: None,
            price_above_200_sma: false,
            price_above_50_sma: false,
            near_52_week_high: false,
            asset_class: ALL.into(),
            focus: ALL.into(),
            brand: ALL.into(),
            min_aum: None,
            max_aum: None,
            max_expense_ratio: None,
            min_nav_return: None,
            issuer_type: ALL.into(),
            issuer: ALL.into(),
            min_ytw: None,
            max_ytw: None,
            min_coupon: None,
            max_coupon: None,
            coupon_type: ALL.into(),
            maturity_year: None,
            credit_rating: ALL.into(),
            mf_category: ALL.into(),
            mf_scheme_type: ALL.into(),
            min_cagr3y: None,
            min_cagr5y: None,
            base_expense_ratio: None,
            total_expense_ratio: None,
            cat_total_expense_ratio: None,
            min_sharpe_ratio: None,
            cat_sharpe_ratio: None,
            riskometer: ALL.into(),
            plan_type: ALL.into(),
        }
    }
}

impl Default for AdvancedFilters {
    fn default() -> Self {
        Self::default_filters()
    }
}

/// `countActiveFilters`.
pub fn count_active_filters(f: &AdvancedFilters) -> usize {
    let mut count = 0usize;
    if f.sector != ALL {
        count += 1;
    }
    if f.market_cap_tier != ALL {
        count += 1;
    }
    if f.exchange != ALL {
        count += 1;
    }
    if !f.indices.is_empty() {
        count += 1;
    }
    if f.min_price.is_some() || f.max_price.is_some() {
        count += 1;
    }
    if f.min_change_percent.is_some() {
        count += 1;
    }
    if f.max_change_percent.is_some() {
        count += 1;
    }
    if f.min_volume.is_some() {
        count += 1;
    }
    if f.pe_min.is_some() || f.pe_max.is_some() {
        count += 1;
    }
    if f.min_dividend_yield.is_some() {
        count += 1;
    }
    if f.min_roce.is_some() {
        count += 1;
    }
    if f.min_net_margin.is_some() {
        count += 1;
    }
    if f.technical_rating != ALL {
        count += 1;
    }
    if f.rsi_min.is_some() || f.rsi_max.is_some() {
        count += 1;
    }
    if f.price_above_200_sma {
        count += 1;
    }
    if f.price_above_50_sma {
        count += 1;
    }
    if f.near_52_week_high {
        count += 1;
    }
    if f.asset_class != ALL {
        count += 1;
    }
    if f.focus != ALL {
        count += 1;
    }
    if f.brand != ALL {
        count += 1;
    }
    if f.min_aum.is_some() {
        count += 1;
    }
    if f.max_expense_ratio.is_some() {
        count += 1;
    }
    if f.min_nav_return.is_some() {
        count += 1;
    }
    if f.issuer_type != ALL {
        count += 1;
    }
    if f.issuer != ALL {
        count += 1;
    }
    if f.min_ytw.is_some() {
        count += 1;
    }
    if f.min_coupon.is_some() {
        count += 1;
    }
    if f.coupon_type != ALL {
        count += 1;
    }
    if f.credit_rating != ALL {
        count += 1;
    }
    if f.mf_category != ALL {
        count += 1;
    }
    if f.mf_scheme_type != ALL {
        count += 1;
    }
    if f.min_cagr3y.is_some() {
        count += 1;
    }
    if f.min_cagr5y.is_some() {
        count += 1;
    }
    if f.base_expense_ratio.is_some() {
        count += 1;
    }
    if f.total_expense_ratio.is_some() {
        count += 1;
    }
    if f.min_sharpe_ratio.is_some() {
        count += 1;
    }
    if f.riskometer != ALL {
        count += 1;
    }
    if f.plan_type != ALL {
        count += 1;
    }
    count
}

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

#[allow(dead_code)]
fn d_quick_preset() -> String {
    "all".to_string()
}

#[allow(dead_code)]
fn d_sort_field() -> String {
    "marketCap".to_string()
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

impl SortDirection {
    pub const fn as_id(self) -> &'static str {
        match self {
            SortDirection::Asc => "asc",
            SortDirection::Desc => "desc",
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
// Filter pipeline + sorting
// ---------------------------------------------------------------------------

/// Quick preset ids surfaced on the filter bar.
pub const QUICK_PRESETS: [(&str, &str); 11] = [
    ("all", "All"),
    ("watchlist", "Watchlist"),
    ("gainers", "Gainers"),
    ("losers", "Losers"),
    ("most_active", "Most active"),
    ("high52", "Near 52W high"),
    ("oversold", "Oversold"),
    ("overbought", "Overbought"),
    ("dividend", "Dividend"),
    ("value", "Value"),
    ("momentum", "Momentum"),
];

/// The full screener filter pipeline — the faithful port of the labs
/// `ScreenerApp.filteredInstruments` memo (search → quick preset → advanced).
pub fn filter_instruments(
    source: &[&'static Instrument],
    search_query: &str,
    quick_preset: &str,
    filters: &AdvancedFilters,
    shortlist: &[String],
) -> Vec<&'static Instrument> {
    let query = search_query.trim().to_lowercase();

    source
        .iter()
        .copied()
        .filter(|inst| {
            // 1. Text search (symbol / name / sector).
            if !query.is_empty() {
                let matches = inst.symbol.to_lowercase().contains(&query)
                    || inst.name.to_lowercase().contains(&query)
                    || inst.sector.to_lowercase().contains(&query);
                if !matches {
                    return false;
                }
            }

            // 2. Quick presets.
            match quick_preset {
                "watchlist" if !shortlist.iter().any(|s| s == &inst.symbol) => return false,
                "gainers" if inst.change_percent <= 0.0 => return false,
                "losers" if inst.change_percent >= 0.0 => return false,
                "most_active" if inst.volume < 2_000_000.0 => return false,
                "high52" if inst.price < inst.high52 * 0.95 => return false,
                "oversold" if inst.rsi14 >= 35.0 => return false,
                "overbought" if inst.rsi14 <= 70.0 => return false,
                "dividend" if inst.dividend_yield < 1.5 => return false,
                "value" if !(inst.pe > 0.0 && inst.pe <= 25.0) => return false,
                "momentum"
                    if inst.technical_rating != "Strong Buy" && inst.technical_rating != "Buy" =>
                {
                    return false;
                }
                _ => {}
            }

            // 3. Advanced filters (descriptive).
            if filters.sector != ALL && inst.sector != filters.sector {
                return false;
            }
            if filters.exchange != ALL && inst.exchange != filters.exchange {
                return false;
            }

            if !filters.indices.is_empty()
                && !filters
                    .indices
                    .iter()
                    .any(|code| instrument_belongs_to_index(inst, code))
            {
                return false;
            }

            if filters.market_cap_tier != ALL {
                let cap = inst.market_cap;
                let keep = match filters.market_cap_tier.as_str() {
                    "mega" => cap >= 2_000_000_000_000.0,
                    "large" => cap >= 500_000_000_000.0,
                    "mid" => (100_000_000_000.0..500_000_000_000.0).contains(&cap),
                    "small" => cap < 100_000_000_000.0,
                    _ => true,
                };
                if !keep {
                    return false;
                }
            }

            if filters.min_price.is_some_and(|min| inst.price < min) {
                return false;
            }
            if filters.max_price.is_some_and(|max| inst.price > max) {
                return false;
            }
            if filters
                .min_change_percent
                .is_some_and(|min| inst.change_percent < min)
            {
                return false;
            }
            if filters
                .max_change_percent
                .is_some_and(|max| inst.change_percent > max)
            {
                return false;
            }
            if filters.min_volume.is_some_and(|min| inst.volume < min) {
                return false;
            }
            if filters
                .pe_min
                .is_some_and(|min| !(inst.pe > 0.0) || inst.pe < min)
            {
                return false;
            }
            if filters
                .pe_max
                .is_some_and(|max| !(inst.pe > 0.0) || inst.pe > max)
            {
                return false;
            }
            if filters
                .min_dividend_yield
                .is_some_and(|min| !(inst.dividend_yield > 0.0) || inst.dividend_yield < min)
            {
                return false;
            }
            if filters
                .min_roce
                .is_some_and(|min| !(inst.roce > 0.0) || inst.roce < min)
            {
                return false;
            }
            if filters
                .min_net_margin
                .is_some_and(|min| !(inst.net_margin > 0.0) || inst.net_margin < min)
            {
                return false;
            }

            if filters.technical_rating != ALL {
                let keep = match filters.technical_rating.as_str() {
                    "Buy" => {
                        inst.technical_rating == "Buy" || inst.technical_rating == "Strong Buy"
                    }
                    "Sell" => {
                        inst.technical_rating == "Sell" || inst.technical_rating == "Strong Sell"
                    }
                    rating => inst.technical_rating == rating,
                };
                if !keep {
                    return false;
                }
            }

            if filters.rsi_min.is_some_and(|min| inst.rsi14 < min) {
                return false;
            }
            if filters.rsi_max.is_some_and(|max| inst.rsi14 > max) {
                return false;
            }
            if filters.price_above_200_sma && inst.price <= inst.sma200 {
                return false;
            }
            if filters.price_above_50_sma && inst.price <= inst.sma50 {
                return false;
            }
            if filters.near_52_week_high && inst.price < inst.high52 * 0.95 {
                return false;
            }

            // 4. ETF specific.
            if filters.asset_class != ALL && inst.sector != filters.asset_class {
                return false;
            }
            if filters.focus != ALL && inst.industry != filters.focus {
                return false;
            }
            if filters.brand != ALL
                && !inst
                    .name
                    .to_lowercase()
                    .contains(&filters.brand.to_lowercase())
            {
                return false;
            }

            // 5. Bond specific.
            if filters.issuer_type != ALL && inst.sector != filters.issuer_type {
                return false;
            }
            if filters.credit_rating != ALL {
                // The bundled universe carries no instrument credit rating;
                // the closest wire signal is the issuer family in
                // `industry` (`AAA PSU Bond`), so AAA matches those rows.
                let keep = inst.technical_rating == filters.credit_rating
                    || inst
                        .industry
                        .to_lowercase()
                        .contains(&filters.credit_rating.to_lowercase());
                if !keep {
                    return false;
                }
            }

            // 6. Mutual-fund specific.
            if filters.mf_category != ALL
                && inst.industry != filters.mf_category
                && inst.sector != filters.mf_category
            {
                return false;
            }
            if filters.mf_scheme_type != ALL && inst.industry != filters.mf_scheme_type {
                return false;
            }
            if filters
                .min_cagr3y
                .is_some_and(|min| static_cagr3y(inst) < min)
            {
                return false;
            }
            if filters
                .min_cagr5y
                .is_some_and(|min| static_cagr5y(inst) < min)
            {
                return false;
            }

            true
        })
        .collect()
}

/// The ETF/MF `cagr3y` fallback (labs columns: `perf1Y * 0.8`).
pub fn static_cagr3y(inst: &Instrument) -> f64 {
    if inst.perf1y > 0.0 {
        inst.perf1y * 0.8
    } else {
        0.0
    }
}

/// The ETF/MF `cagr5y` fallback (`perf1Y * 0.72`).
pub fn static_cagr5y(inst: &Instrument) -> f64 {
    if inst.perf1y > 0.0 {
        inst.perf1y * 0.72
    } else {
        0.0
    }
}

/// Numeric value behind a sort field, with the labs' special cases
/// (`relVol`, `epsGrowth → revenueGrowth`, `analystRating`, `range52`).
fn sort_value(inst: &Instrument, field: &str) -> f64 {
    match field {
        "relVol" => inst.rel_vol(),
        "epsGrowth" => inst.revenue_growth,
        "range52" => inst.range52(),
        "changePercent" => inst.change_percent,
        "change" => inst.change,
        "price" => inst.price,
        "volume" => inst.volume,
        "marketCap" => inst.market_cap,
        "pe" => inst.pe,
        "forwardPe" => inst.forward_pe,
        "pb" => inst.pb,
        "eps" => inst.eps,
        "dividendYield" => inst.dividend_yield,
        "rsi14" => inst.rsi14,
        "sma200" => inst.sma200,
        "sma50" => inst.sma50,
        "sma20" => inst.sma20,
        "high52" => inst.high52,
        "low52" => inst.low52,
        "perf1W" => inst.perf1w,
        "perf1M" => inst.perf1m,
        "perf3M" => inst.perf3m,
        "perf1Y" => inst.perf1y,
        "revenueGrowth" => inst.revenue_growth,
        "netMargin" => inst.net_margin,
        "roce" => inst.roce,
        "roe" => inst.roe,
        "debtToEquity" => inst.debt_to_equity,
        _ => 0.0,
    }
}

fn rating_rank(rating: &str) -> i32 {
    match rating {
        "Strong Buy" => 4,
        "Buy" => 3,
        "Neutral" => 2,
        "Sell" => 1,
        "Strong Sell" => 0,
        _ => -1,
    }
}

fn sort_field_is_string(field: &str) -> bool {
    matches!(
        field,
        "analystRating" | "technicalRating" | "symbol" | "name"
    )
}

/// Stable sort with tie-breaking on symbol.
pub fn sort_instruments(
    rows: &mut Vec<&'static Instrument>,
    field: &str,
    direction: SortDirection,
) {
    let invert = direction == SortDirection::Desc;

    rows.sort_by(|a, b| {
        let mut ordering = if sort_field_is_string(field) {
            let raw = match field {
                "analystRating" | "technicalRating" => {
                    rating_rank(&a.technical_rating).cmp(&rating_rank(&b.technical_rating))
                }
                "symbol" => a.symbol.cmp(&b.symbol),
                "name" => a.name.cmp(&b.name),
                _ => std::cmp::Ordering::Equal,
            };
            // String sorts honour the direction too (`Desc` was dropped here,
            // so A→Z showed while the header arrow pointed ↓).
            if invert { raw.reverse() } else { raw }
        } else {
            let (va, vb) = (sort_value(a, field), sort_value(b, field));
            let cmp = compare_f64(va, vb);
            if invert { cmp.reverse() } else { cmp }
        };
        if ordering == std::cmp::Ordering::Equal {
            ordering = a.symbol.cmp(&b.symbol);
        }
        ordering
    });
}

fn compare_f64(a: f64, b: f64) -> std::cmp::Ordering {
    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
}

/// Sort toggle cycle: field switch → `desc`; same field → desc → asc → none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortState {
    Desc,
    Asc,
    None,
}

// ---------------------------------------------------------------------------
// CSV export
// ---------------------------------------------------------------------------

/// Build the screener CSV (the labs `handleExportCSV`).
pub fn build_csv(rows: &[&'static Instrument]) -> String {
    let mut lines =
        vec!["Symbol,Name,Exchange,Price,Change%,Volume,MarketCap,PE,RSI14,Rating".to_string()];
    for inst in rows {
        lines.push(format!(
            "{},{},{},{},{},{},{},{},{},\"{}\"",
            inst.symbol,
            format_csv_cell(&inst.name),
            inst.exchange,
            inst.price,
            inst.change_percent,
            inst.volume,
            inst.market_cap,
            if inst.pe > 0.0 {
                format!("{}", inst.pe)
            } else {
                String::new()
            },
            inst.rsi14,
            inst.technical_rating,
        ));
    }
    lines.join("\n")
}

fn format_csv_cell(raw: &str) -> String {
    format!("\"{}\"", raw.replace('"', "\"\""))
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
// Column registry (port of core/columns.tsx)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnCategory {
    Overview,
    Performance,
    Valuation,
    Technicals,
    Fundamentals,
    Funds,
    Bonds,
}

impl ColumnCategory {
    pub const fn as_id(self) -> &'static str {
        match self {
            ColumnCategory::Overview => "overview",
            ColumnCategory::Performance => "performance",
            ColumnCategory::Valuation => "valuation",
            ColumnCategory::Technicals => "technicals",
            ColumnCategory::Fundamentals => "fundamentals",
            ColumnCategory::Funds => "funds",
            ColumnCategory::Bonds => "bonds",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            ColumnCategory::Overview => "Overview",
            ColumnCategory::Performance => "Performance",
            ColumnCategory::Valuation => "Valuation",
            ColumnCategory::Technicals => "Technicals",
            ColumnCategory::Fundamentals => "Fundamentals",
            ColumnCategory::Funds => "Fund Metrics",
            ColumnCategory::Bonds => "Bond Details",
        }
    }
}

/// One structural column (`ColumnDef` in the labs app minus the renderer —
/// rendering lives in the table component keyed by id).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColumnDef {
    pub id: &'static str,
    pub label: &'static str,
    pub category: ColumnCategory,
    /// `Some([..])` restricts the column to certain screener types.
    pub applicable: Option<&'static [ScreenerType]>,
    pub align: ColumnAlign,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnAlign {
    Left,
    Right,
    Center,
}

impl ColumnAlign {
    pub const fn as_class(self) -> &'static str {
        match self {
            ColumnAlign::Left => "align-left",
            ColumnAlign::Right => "align-right",
            ColumnAlign::Center => "align-center",
        }
    }
}

use ScreenerType::{Bonds as TBond, Etf as TEtf, Mf as TMf, Stocks as TStock};

const STOCKS_ONLY: &[ScreenerType] = &[TStock];
const STOCKS_ETFS: &[ScreenerType] = &[TStock, TEtf];
const ALL_ASSETS: &[ScreenerType] = &[TStock, TEtf, TBond, TMf];
const ETFS_MFS: &[ScreenerType] = &[TEtf, TMf];
const ETFS_MFS_BONDS: &[ScreenerType] = &[TEtf, TMf, TBond];
const MFS_ONLY: &[ScreenerType] = &[TMf];
const BONDS_ONLY: &[ScreenerType] = &[TBond];
const ETFS_ONLY: &[ScreenerType] = &[TEtf];

/// The structural column registry core (`ALL_COLUMNS`); the debt/fund
/// addendum lives in `columns_registry()`.
pub const ALL_COLUMNS: [ColumnDef; 35] = [
    ColumnDef {
        id: "symbol",
        label: "Symbol",
        category: ColumnCategory::Overview,
        applicable: None,
        align: ColumnAlign::Left,
    },
    ColumnDef {
        id: "exchange",
        label: "Exchange",
        category: ColumnCategory::Overview,
        applicable: None,
        align: ColumnAlign::Center,
    },
    ColumnDef {
        id: "country",
        label: "Country",
        category: ColumnCategory::Overview,
        applicable: None,
        align: ColumnAlign::Center,
    },
    ColumnDef {
        id: "price",
        label: "Price",
        category: ColumnCategory::Overview,
        applicable: None,
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "changePercent",
        label: "Chg %",
        category: ColumnCategory::Overview,
        applicable: None,
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "change",
        label: "Change (Pts)",
        category: ColumnCategory::Performance,
        applicable: None,
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "volume",
        label: "Vol",
        category: ColumnCategory::Overview,
        applicable: None,
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "relVol",
        label: "Rel Vol",
        category: ColumnCategory::Overview,
        applicable: None,
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "perf1W",
        label: "Perf 1W %",
        category: ColumnCategory::Performance,
        applicable: None,
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "perf1M",
        label: "Perf 1M %",
        category: ColumnCategory::Performance,
        applicable: None,
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "perf3M",
        label: "Perf 3M %",
        category: ColumnCategory::Performance,
        applicable: None,
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "perf1Y",
        label: "Perf 1Y %",
        category: ColumnCategory::Performance,
        applicable: None,
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "technicalRating",
        label: "Technical Rating",
        category: ColumnCategory::Technicals,
        applicable: None,
        align: ColumnAlign::Center,
    },
    ColumnDef {
        id: "rsi14",
        label: "RSI (14)",
        category: ColumnCategory::Technicals,
        applicable: None,
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "range52",
        label: "52W Range Bar",
        category: ColumnCategory::Technicals,
        applicable: None,
        align: ColumnAlign::Left,
    },
    ColumnDef {
        id: "sma200",
        label: "200 SMA",
        category: ColumnCategory::Technicals,
        applicable: None,
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "sparkline",
        label: "7D Trend",
        category: ColumnCategory::Technicals,
        applicable: None,
        align: ColumnAlign::Left,
    },
    ColumnDef {
        id: "high52",
        label: "52W High",
        category: ColumnCategory::Technicals,
        applicable: None,
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "low52",
        label: "52W Low",
        category: ColumnCategory::Technicals,
        applicable: None,
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "marketCap",
        label: "Mkt Cap",
        category: ColumnCategory::Overview,
        applicable: Some(STOCKS_ONLY),
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "pe",
        label: "P/E",
        category: ColumnCategory::Overview,
        applicable: Some(STOCKS_ONLY),
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "forwardPe",
        label: "Forward P/E",
        category: ColumnCategory::Valuation,
        applicable: Some(STOCKS_ONLY),
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "pb",
        label: "Price to Book",
        category: ColumnCategory::Valuation,
        applicable: Some(STOCKS_ONLY),
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "eps",
        label: "EPS Dil TTM",
        category: ColumnCategory::Overview,
        applicable: Some(STOCKS_ONLY),
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "epsGrowth",
        label: "EPS Dil Growth",
        category: ColumnCategory::Overview,
        applicable: Some(STOCKS_ONLY),
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "dividendYield",
        label: "Div Yield %",
        category: ColumnCategory::Overview,
        applicable: Some(STOCKS_ETFS),
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "sector",
        label: "Sector",
        category: ColumnCategory::Overview,
        applicable: Some(ALL_ASSETS),
        align: ColumnAlign::Left,
    },
    ColumnDef {
        id: "analystRating",
        label: "Analyst Rating",
        category: ColumnCategory::Overview,
        applicable: Some(STOCKS_ONLY),
        align: ColumnAlign::Center,
    },
    ColumnDef {
        id: "netMargin",
        label: "Net Margin %",
        category: ColumnCategory::Fundamentals,
        applicable: Some(STOCKS_ONLY),
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "roce",
        label: "ROCE %",
        category: ColumnCategory::Fundamentals,
        applicable: Some(STOCKS_ONLY),
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "roe",
        label: "ROE %",
        category: ColumnCategory::Fundamentals,
        applicable: Some(STOCKS_ONLY),
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "debtToEquity",
        label: "Debt / Equity",
        category: ColumnCategory::Fundamentals,
        applicable: Some(STOCKS_ONLY),
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "aum",
        label: "AUM",
        category: ColumnCategory::Funds,
        applicable: Some(ETFS_MFS_BONDS),
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "expenseRatio",
        label: "Exp Ratio %",
        category: ColumnCategory::Funds,
        applicable: Some(ETFS_MFS),
        align: ColumnAlign::Right,
    },
    ColumnDef {
        id: "sharpeRatio",
        label: "Sharpe Ratio",
        category: ColumnCategory::Funds,
        applicable: Some(ETFS_MFS),
        align: ColumnAlign::Right,
    },
];

/// The 35th column, kept separately as a const because the array literal
/// above already has 35 rows — use `columns_registry()` instead.
pub fn columns_registry() -> &'static [ColumnDef] {
    static REGISTRY: OnceLock<Vec<ColumnDef>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut registry = ALL_COLUMNS.to_vec();
        // Debt / fund columns referenced by the bond and MF tab presets.
        registry.extend_from_slice(&[
            ColumnDef {
                id: "cagr3y",
                label: "3Y Return %",
                category: ColumnCategory::Performance,
                applicable: Some(ETFS_MFS),
                align: ColumnAlign::Right,
            },
            ColumnDef {
                id: "cagr5y",
                label: "5Y Return %",
                category: ColumnCategory::Performance,
                applicable: Some(ETFS_MFS),
                align: ColumnAlign::Right,
            },
            ColumnDef {
                id: "ytw",
                label: "YTW %",
                category: ColumnCategory::Bonds,
                applicable: Some(BONDS_ONLY),
                align: ColumnAlign::Right,
            },
            ColumnDef {
                id: "coupon",
                label: "Coupon %",
                category: ColumnCategory::Bonds,
                applicable: Some(BONDS_ONLY),
                align: ColumnAlign::Right,
            },
            ColumnDef {
                id: "couponType",
                label: "Coupon type",
                category: ColumnCategory::Bonds,
                applicable: Some(BONDS_ONLY),
                align: ColumnAlign::Center,
            },
            ColumnDef {
                id: "maturityYear",
                label: "Maturity",
                category: ColumnCategory::Bonds,
                applicable: Some(BONDS_ONLY),
                align: ColumnAlign::Center,
            },
            ColumnDef {
                id: "creditRating",
                label: "Credit rating",
                category: ColumnCategory::Bonds,
                applicable: Some(BONDS_ONLY),
                align: ColumnAlign::Center,
            },
            ColumnDef {
                id: "schemeType",
                label: "Type",
                category: ColumnCategory::Funds,
                applicable: Some(MFS_ONLY),
                align: ColumnAlign::Center,
            },
            ColumnDef {
                id: "totalExpenseRatio",
                label: "Exp ratio (total) %",
                category: ColumnCategory::Funds,
                applicable: Some(MFS_ONLY),
                align: ColumnAlign::Right,
            },
            ColumnDef {
                id: "catSharpeRatio",
                label: "Cat. Sharpe",
                category: ColumnCategory::Funds,
                applicable: Some(MFS_ONLY),
                align: ColumnAlign::Right,
            },
            ColumnDef {
                id: "catTotalExpenseRatio",
                label: "Cat. TER %",
                category: ColumnCategory::Funds,
                applicable: Some(MFS_ONLY),
                align: ColumnAlign::Right,
            },
            ColumnDef {
                id: "planType",
                label: "Plan",
                category: ColumnCategory::Funds,
                applicable: Some(MFS_ONLY),
                align: ColumnAlign::Center,
            },
            ColumnDef {
                id: "riskometer",
                label: "Riskometer",
                category: ColumnCategory::Funds,
                applicable: Some(MFS_ONLY),
                align: ColumnAlign::Center,
            },
            ColumnDef {
                id: "assetClass",
                label: "Asset class",
                category: ColumnCategory::Funds,
                applicable: Some(ETFS_ONLY),
                align: ColumnAlign::Center,
            },
            ColumnDef {
                id: "focus",
                label: "Focus",
                category: ColumnCategory::Funds,
                applicable: Some(ETFS_ONLY),
                align: ColumnAlign::Left,
            },
            ColumnDef {
                id: "brand",
                label: "Brand / AMC",
                category: ColumnCategory::Funds,
                applicable: Some(ETFS_MFS),
                align: ColumnAlign::Left,
            },
            ColumnDef {
                id: "navReturn",
                label: "NAV return %",
                category: ColumnCategory::Funds,
                applicable: Some(ETFS_ONLY),
                align: ColumnAlign::Right,
            },
        ]);
        registry
    })
}

/// Columns applicable to one screener type (`getColumnsForScreenerType`).
pub fn columns_for_screener(screener_type: ScreenerType) -> Vec<ColumnDef> {
    columns_registry()
        .iter()
        .filter(|column| {
            column
                .applicable
                .map_or(true, |assets| assets.contains(&screener_type))
        })
        .copied()
        .collect()
}

/// `TAB_COLUMN_PRESETS` + `getPresetForScreenerTab`.
pub fn preset_for_tab(screener_type: ScreenerType, tab_id: &str) -> Option<Vec<&'static str>> {
    let overview = |etf: &'static [&'static str],
                    bonds: &'static [&'static str],
                    mf: &'static [&'static str],
                    stocks: &'static [&'static str]|
     -> Vec<&'static str> {
        match screener_type {
            ScreenerType::Etf => etf.to_vec(),
            ScreenerType::Bonds => bonds.to_vec(),
            ScreenerType::Mf => mf.to_vec(),
            _ => stocks.to_vec(),
        }
    };

    Some(match (screener_type, tab_id) {
        (_, "overview") => overview(
            &[
                "symbol",
                "price",
                "changePercent",
                "volume",
                "aum",
                "expenseRatio",
                "dividendYield",
                "sector",
                "technicalRating",
            ],
            &[
                "symbol",
                "price",
                "changePercent",
                "ytw",
                "coupon",
                "creditRating",
                "marketCap",
                "volume",
            ],
            &[
                "symbol",
                "price",
                "changePercent",
                "aum",
                "schemeType",
                "expenseRatio",
                "cagr3y",
                "cagr5y",
                "sharpeRatio",
                "technicalRating",
            ],
            &[
                "symbol",
                "price",
                "changePercent",
                "volume",
                "relVol",
                "marketCap",
                "pe",
                "eps",
                "epsGrowth",
                "dividendYield",
                "sector",
                "analystRating",
            ],
        ),
        (_, "performance") => vec![
            "symbol",
            "price",
            "changePercent",
            "change",
            "perf1W",
            "perf1M",
            "perf1Y",
            "volume",
            "relVol",
            "high52",
            "low52",
        ],
        (_, "technicals") => vec![
            "symbol",
            "price",
            "changePercent",
            "technicalRating",
            "rsi14",
            "sma200",
            "range52",
            "sparkline",
        ],
        (_, "valuation") => vec![
            "symbol",
            "price",
            "marketCap",
            "pe",
            "forwardPe",
            "pb",
            "eps",
            "dividendYield",
            "revenueGrowth",
        ],
        (_, "dividends") => vec![
            "symbol",
            "price",
            "dividendYield",
            "eps",
            "pe",
            "marketCap",
            "sector",
        ],
        (_, "margins") => vec![
            "symbol",
            "price",
            "netMargin",
            "roce",
            "debtToEquity",
            "revenueGrowth",
            "sector",
        ],
        (_, "extended_hours") => vec!["symbol", "price", "changePercent", "volume", "marketCap"],
        (_, "forecasts") => vec![
            "symbol",
            "price",
            "pe",
            "forwardPe",
            "eps",
            "revenueGrowth",
            "analystRating",
        ],
        (_, "profitability") => vec![
            "symbol",
            "price",
            "netMargin",
            "roce",
            "roe",
            "debtToEquity",
            "revenueGrowth",
        ],
        (_, "income_statement") => vec![
            "symbol",
            "price",
            "eps",
            "epsGrowth",
            "revenueGrowth",
            "netMargin",
        ],
        (_, "balance_sheet") => vec!["symbol", "price", "marketCap", "debtToEquity", "roce", "pb"],
        (_, "cash_flow") => vec!["symbol", "price", "netMargin", "roce", "dividendYield"],
        (_, "per_share") => vec!["symbol", "price", "eps", "pb", "dividendYield"],
        // ETF tabs
        (ScreenerType::Etf, "fund_flows") => vec![
            "symbol",
            "price",
            "changePercent",
            "volume",
            "aum",
            "perf1W",
            "perf1M",
            "perf1Y",
        ],
        (ScreenerType::Etf, "nav_performance") => vec![
            "symbol",
            "price",
            "changePercent",
            "perf1M",
            "perf1Y",
            "cagr3y",
            "aum",
            "expenseRatio",
        ],
        (ScreenerType::Etf, "holdings") => vec![
            "symbol",
            "price",
            "changePercent",
            "sector",
            "aum",
            "expenseRatio",
            "dividendYield",
        ],
        (ScreenerType::Etf, "risk") => vec![
            "symbol",
            "price",
            "rsi14",
            "sma200",
            "sharpeRatio",
            "range52",
            "technicalRating",
        ],
        // Bond tabs
        (ScreenerType::Bonds, "security_info") => vec![
            "symbol",
            "price",
            "ytw",
            "coupon",
            "creditRating",
            "sector",
            "volume",
        ],
        (ScreenerType::Bonds, "interest_rate_risk") => {
            vec!["symbol", "price", "ytw", "coupon", "sma200", "creditRating"]
        }
        (ScreenerType::Bonds, "spreads") => {
            vec!["symbol", "price", "ytw", "coupon", "creditRating"]
        }
        (ScreenerType::Bonds, "amounts") => vec!["symbol", "price", "volume", "marketCap", "aum"],
        (ScreenerType::Bonds, "bond_features") => {
            vec!["symbol", "price", "coupon", "ytw", "creditRating", "sector"]
        }
        // MF tabs
        (ScreenerType::Mf, "returns_cagr") => vec![
            "symbol",
            "price",
            "changePercent",
            "cagr3y",
            "cagr5y",
            "perf1M",
            "perf1Y",
            "aum",
        ],
        (ScreenerType::Mf, "portfolio_holdings") => vec![
            "symbol",
            "price",
            "changePercent",
            "sector",
            "schemeType",
            "aum",
        ],
        (ScreenerType::Mf, "sector_allocation") => {
            vec!["symbol", "price", "sector", "aum", "schemeType"]
        }
        (ScreenerType::Mf, "risk_ratios") => vec![
            "symbol",
            "price",
            "sharpeRatio",
            "catSharpeRatio",
            "rsi14",
            "technicalRating",
        ],
        (ScreenerType::Mf, "fees_loads") => vec![
            "symbol",
            "price",
            "expenseRatio",
            "totalExpenseRatio",
            "catTotalExpenseRatio",
            "schemeType",
            "aum",
        ],
        // Tabs without a dedicated preset fall through to the overview.
        _ => return None,
    })
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

    /// Fetch from context (defined at `crate::pages::screener_page`).
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

/// Merge a screen's (possibly sparse, lower-case) filters onto the defaults,
/// like `{ ...DEFAULT_ADVANCED_FILTERS, ...screen.filters }`.
pub fn normalize_filters(filters: &AdvancedFilters) -> AdvancedFilters {
    let mut merged = AdvancedFilters::default();
    if filters.sector != ALL {
        merged.sector = filters.sector.clone();
    }
    if filters.market_cap_tier != ALL {
        merged.market_cap_tier = filters.market_cap_tier.clone();
    }
    if filters.exchange != ALL {
        merged.exchange = filters.exchange.clone();
    }
    merged.indices = filters.indices.clone();
    merged.min_price = filters.min_price;
    merged.max_price = filters.max_price;
    merged.min_change_percent = filters.min_change_percent;
    merged.max_change_percent = filters.max_change_percent;
    merged.min_volume = filters.min_volume;
    merged.pe_min = filters.pe_min;
    merged.pe_max = filters.pe_max;
    merged.min_dividend_yield = filters.min_dividend_yield;
    merged.min_roce = filters.min_roce;
    merged.min_net_margin = filters.min_net_margin;
    if filters.technical_rating != ALL {
        merged.technical_rating = filters.technical_rating.clone();
    }
    merged.rsi_min = filters.rsi_min;
    merged.rsi_max = filters.rsi_max;
    merged.price_above_200_sma = filters.price_above_200_sma;
    merged.price_above_50_sma = filters.price_above_50_sma;
    merged.near_52_week_high = filters.near_52_week_high;
    if filters.asset_class != ALL {
        merged.asset_class = filters.asset_class.clone();
    }
    if filters.focus != ALL {
        merged.focus = filters.focus.clone();
    }
    if filters.brand != ALL {
        merged.brand = filters.brand.clone();
    }
    merged.min_aum = filters.min_aum;
    merged.max_aum = filters.max_aum;
    merged.max_expense_ratio = filters.max_expense_ratio;
    merged.min_nav_return = filters.min_nav_return;
    if filters.issuer_type != ALL {
        merged.issuer_type = filters.issuer_type.clone();
    }
    if filters.issuer != ALL {
        merged.issuer = filters.issuer.clone();
    }
    merged.min_ytw = filters.min_ytw;
    merged.max_ytw = filters.max_ytw;
    merged.min_coupon = filters.min_coupon;
    merged.max_coupon = filters.max_coupon;
    if filters.coupon_type != ALL {
        merged.coupon_type = filters.coupon_type.clone();
    }
    merged.maturity_year = filters.maturity_year;
    if filters.credit_rating != ALL {
        merged.credit_rating = filters.credit_rating.clone();
    }
    if filters.mf_category != ALL {
        merged.mf_category = filters.mf_category.clone();
    }
    if filters.mf_scheme_type != ALL {
        merged.mf_scheme_type = filters.mf_scheme_type.clone();
    }
    merged.min_cagr3y = filters.min_cagr3y;
    merged.min_cagr5y = filters.min_cagr5y;
    merged.base_expense_ratio = filters.base_expense_ratio;
    merged.total_expense_ratio = filters.total_expense_ratio;
    merged.cat_total_expense_ratio = filters.cat_total_expense_ratio;
    merged.min_sharpe_ratio = filters.min_sharpe_ratio;
    merged.cat_sharpe_ratio = filters.cat_sharpe_ratio;
    if filters.riskometer != ALL {
        merged.riskometer = filters.riskometer.clone();
    }
    if filters.plan_type != ALL {
        merged.plan_type = filters.plan_type.clone();
    }
    merged
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

    #[test]
    fn quick_preset_filters_match_the_labs_gates() {
        let rows = instruments::universe_for_screener("IN", "stocks");
        let gainers = filter_instruments(&rows, "", "gainers", &AdvancedFilters::default(), &[]);
        assert!(gainers.iter().all(|inst| inst.change_percent > 0.0));
        let losers = filter_instruments(&rows, "", "losers", &AdvancedFilters::default(), &[]);
        assert!(losers.iter().all(|inst| inst.change_percent < 0.0));
    }

    #[test]
    fn sorting_special_cases_use_the_labs_columns() {
        let mut rows = instruments::universe_for_screener("IN", "stocks");
        sort_instruments(&mut rows, "relVol", SortDirection::Desc);
        assert!(
            rows.windows(2)
                .all(|pair| pair[0].rel_vol() >= pair[1].rel_vol())
        );
        sort_instruments(&mut rows, "marketCap", SortDirection::Desc);
        assert!(
            rows.windows(2)
                .all(|pair| pair[0].market_cap >= pair[1].market_cap)
        );
    }

    #[test]
    fn csv_export_matches_the_labs_shape() {
        let rows = instruments::universe_for_screener("IN", "stocks");
        let csv = build_csv(&rows[..1]);
        assert!(csv.starts_with("Symbol,Name,Exchange,Price,Change%"));
        assert_eq!(csv.lines().count(), 2);
    }
}
