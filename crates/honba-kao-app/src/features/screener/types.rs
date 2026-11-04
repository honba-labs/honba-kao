use crate::features::screener::indices::{benchmark_for, index_contains};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ScreenerAssetType {
    #[default]
    Stocks,
    Etf,
    Bonds,
    MutualFunds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ScreenerViewMode {
    #[default]
    Table,
    Matrix,
    Constellation,
    Heatmap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ScreenerViewTab {
    #[default]
    Overview,
    Performance,
    Technicals,
    Valuation,
    Dividends,
    Profitability,
    Financials,
    IncomeStatement,
    BalanceSheet,
    CashFlow,
    PerShare,
    Forecasts,
}

/// One row of the screener universe: identity, real-time quote, performance,
/// technicals, valuation, dividends, health and growth snapshots.
///
/// All monetary/price fields are exact decimals (zero-floats invariant).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenerEquityRecord {
    // Identity
    pub symbol: String,
    pub name: String,
    pub exchange: String,
    pub sector: String,
    pub industry: String,
    pub market_cap_category: String,
    pub is_fno: bool,
    pub is_watchlist: bool,
    // Real-time
    pub price: Decimal,
    pub change: Decimal,
    pub change_pct: Decimal,
    pub volume: u64,
    pub avg_volume_20d: u64,
    pub rvol: Decimal,
    pub turnover_cr: Decimal,
    pub day_low: Decimal,
    pub day_high: Decimal,
    pub high_52w: Decimal,
    pub low_52w: Decimal,
    // Performance
    pub return_1d: Decimal,
    pub return_1w: Decimal,
    pub return_1m: Decimal,
    pub return_3m: Decimal,
    pub return_6m: Decimal,
    pub return_ytd: Decimal,
    pub return_1y: Decimal,
    pub return_3y: Decimal,
    // Technicals
    pub rsi_14: Decimal,
    pub macd_line: Decimal,
    pub macd_signal: Decimal,
    pub macd_hist: Decimal,
    pub sma_20: Decimal,
    pub sma_50: Decimal,
    pub sma_200: Decimal,
    pub ema_9: Decimal,
    pub ema_21: Decimal,
    pub atr_14: Decimal,
    pub atr_pct: Decimal,
    pub bb_upper: Decimal,
    pub bb_lower: Decimal,
    pub bb_bandwidth: Decimal,
    pub supertrend_direction: i8,
    // Valuation
    pub market_cap_cr: Decimal,
    pub pe_ratio: Decimal,
    pub forward_pe: Decimal,
    pub pb_ratio: Decimal,
    pub ps_ratio: Decimal,
    pub ev_ebitda: Decimal,
    pub peg_ratio: Decimal,
    // Dividends
    pub dividend_yield: Decimal,
    pub dividend_per_share: Decimal,
    pub payout_ratio: Decimal,
    // Health
    pub roe: Decimal,
    pub roce: Decimal,
    pub operating_margin: Decimal,
    pub net_margin: Decimal,
    pub debt_to_equity: Decimal,
    pub current_ratio: Decimal,
    pub interest_coverage: Decimal,
    // Growth / per-share
    pub revenue_cr: Decimal,
    pub revenue_growth_yoy: Decimal,
    pub net_profit_cr: Decimal,
    pub eps: Decimal,
    pub fcf_cr: Decimal,
    pub book_value_per_share: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenerColumn {
    pub key: &'static str,
    pub label: &'static str,
    pub sortable: bool,
}

pub const COLUMN_CATALOG: &[ScreenerColumn] = &[
    ScreenerColumn {
        key: "star",
        label: "★",
        sortable: false,
    },
    ScreenerColumn {
        key: "symbol",
        label: "Ticker",
        sortable: true,
    },
    ScreenerColumn {
        key: "price",
        label: "Price",
        sortable: true,
    },
    ScreenerColumn {
        key: "change_pct",
        label: "Chg %",
        sortable: true,
    },
    ScreenerColumn {
        key: "volume",
        label: "Vol",
        sortable: true,
    },
    ScreenerColumn {
        key: "rvol",
        label: "RVOL",
        sortable: false,
    },
    ScreenerColumn {
        key: "market_cap_cr",
        label: "Cap",
        sortable: true,
    },
    ScreenerColumn {
        key: "pe_ratio",
        label: "P/E",
        sortable: true,
    },
    ScreenerColumn {
        key: "range_52w",
        label: "52W Range",
        sortable: false,
    },
];

pub const DEFAULT_VISIBLE_COLUMNS: &[&str] = &[
    "star",
    "symbol",
    "price",
    "change_pct",
    "volume",
    "rvol",
    "market_cap_cr",
    "pe_ratio",
    "range_52w",
];

/// View-tab keys in filter-bar strip order (doc `docs/screener-honba-leptos.md`
/// §6.1); "overview" (or any unknown key) falls back to the stock catalog.
pub const TAB_KEYS: &[&str] = &[
    "overview",
    "performance",
    "technicals",
    "valuation",
    "dividends",
    "profitability",
    "financials",
    "income_statement",
    "balance_sheet",
    "cash_flow",
    "per_share",
    "forecasts",
];

pub const PERFORMANCE_COLUMN_CATALOG: &[ScreenerColumn] = &[
    ScreenerColumn {
        key: "star",
        label: "★",
        sortable: false,
    },
    ScreenerColumn {
        key: "symbol",
        label: "Ticker",
        sortable: true,
    },
    ScreenerColumn {
        key: "return_1d",
        label: "1D %",
        sortable: true,
    },
    ScreenerColumn {
        key: "return_1w",
        label: "1W %",
        sortable: true,
    },
    ScreenerColumn {
        key: "return_1m",
        label: "1M %",
        sortable: true,
    },
    ScreenerColumn {
        key: "return_3m",
        label: "3M %",
        sortable: true,
    },
    ScreenerColumn {
        key: "return_6m",
        label: "6M %",
        sortable: true,
    },
    ScreenerColumn {
        key: "return_ytd",
        label: "YTD %",
        sortable: true,
    },
    ScreenerColumn {
        key: "return_1y",
        label: "1Y %",
        sortable: true,
    },
    ScreenerColumn {
        key: "return_3y",
        label: "3Y %",
        sortable: true,
    },
    ScreenerColumn {
        key: "atr_pct",
        label: "ATR %",
        sortable: true,
    },
];

pub const TECHNICAL_COLUMN_CATALOG: &[ScreenerColumn] = &[
    ScreenerColumn {
        key: "star",
        label: "★",
        sortable: false,
    },
    ScreenerColumn {
        key: "symbol",
        label: "Ticker",
        sortable: true,
    },
    ScreenerColumn {
        key: "rsi_14",
        label: "RSI(14)",
        sortable: true,
    },
    ScreenerColumn {
        key: "macd_hist",
        label: "MACD",
        sortable: true,
    },
    ScreenerColumn {
        key: "supertrend_direction",
        label: "Supertrend",
        sortable: false,
    },
    ScreenerColumn {
        key: "sma_20",
        label: "SMA 20",
        sortable: true,
    },
    ScreenerColumn {
        key: "sma_50",
        label: "SMA 50",
        sortable: true,
    },
    ScreenerColumn {
        key: "sma_200",
        label: "SMA 200",
        sortable: true,
    },
    ScreenerColumn {
        key: "ema_9",
        label: "EMA 9",
        sortable: true,
    },
    ScreenerColumn {
        key: "ema_21",
        label: "EMA 21",
        sortable: true,
    },
    ScreenerColumn {
        key: "bb_bandwidth",
        label: "BBW %",
        sortable: true,
    },
];

pub const VALUATION_COLUMN_CATALOG: &[ScreenerColumn] = &[
    ScreenerColumn {
        key: "star",
        label: "★",
        sortable: false,
    },
    ScreenerColumn {
        key: "symbol",
        label: "Ticker",
        sortable: true,
    },
    ScreenerColumn {
        key: "pe_ratio",
        label: "P/E",
        sortable: true,
    },
    ScreenerColumn {
        key: "forward_pe",
        label: "Fwd P/E",
        sortable: true,
    },
    ScreenerColumn {
        key: "pb_ratio",
        label: "P/B",
        sortable: true,
    },
    ScreenerColumn {
        key: "ps_ratio",
        label: "P/S",
        sortable: true,
    },
    ScreenerColumn {
        key: "ev_ebitda",
        label: "EV/EBITDA",
        sortable: true,
    },
    ScreenerColumn {
        key: "peg_ratio",
        label: "PEG",
        sortable: true,
    },
];

pub const DIVIDEND_COLUMN_CATALOG: &[ScreenerColumn] = &[
    ScreenerColumn {
        key: "star",
        label: "★",
        sortable: false,
    },
    ScreenerColumn {
        key: "symbol",
        label: "Ticker",
        sortable: true,
    },
    ScreenerColumn {
        key: "dividend_yield",
        label: "Div Yield %",
        sortable: true,
    },
    ScreenerColumn {
        key: "payout_ratio",
        label: "Payout %",
        sortable: true,
    },
    ScreenerColumn {
        key: "dividend_per_share",
        label: "DPS ₹",
        sortable: true,
    },
];

pub const PROFITABILITY_COLUMN_CATALOG: &[ScreenerColumn] = &[
    ScreenerColumn {
        key: "star",
        label: "★",
        sortable: false,
    },
    ScreenerColumn {
        key: "symbol",
        label: "Ticker",
        sortable: true,
    },
    ScreenerColumn {
        key: "roe",
        label: "ROE %",
        sortable: true,
    },
    ScreenerColumn {
        key: "roce",
        label: "ROCE %",
        sortable: true,
    },
    ScreenerColumn {
        key: "operating_margin",
        label: "Op Margin %",
        sortable: true,
    },
    ScreenerColumn {
        key: "net_margin",
        label: "Net Margin %",
        sortable: true,
    },
];

pub const FINANCIAL_COLUMN_CATALOG: &[ScreenerColumn] = &[
    ScreenerColumn {
        key: "star",
        label: "★",
        sortable: false,
    },
    ScreenerColumn {
        key: "symbol",
        label: "Ticker",
        sortable: true,
    },
    ScreenerColumn {
        key: "revenue_cr",
        label: "Revenue ₹Cr",
        sortable: true,
    },
    ScreenerColumn {
        key: "revenue_growth_yoy",
        label: "Rev Growth %",
        sortable: true,
    },
    ScreenerColumn {
        key: "net_profit_cr",
        label: "PAT ₹Cr",
        sortable: true,
    },
    ScreenerColumn {
        key: "eps",
        label: "EPS ₹",
        sortable: true,
    },
];

pub const INCOME_STATEMENT_COLUMN_CATALOG: &[ScreenerColumn] = &[
    ScreenerColumn {
        key: "star",
        label: "★",
        sortable: false,
    },
    ScreenerColumn {
        key: "symbol",
        label: "Ticker",
        sortable: true,
    },
    ScreenerColumn {
        key: "revenue_cr",
        label: "Revenue ₹Cr",
        sortable: true,
    },
    ScreenerColumn {
        key: "net_profit_cr",
        label: "PAT ₹Cr",
        sortable: true,
    },
    ScreenerColumn {
        key: "eps",
        label: "EPS ₹",
        sortable: true,
    },
];

pub const BALANCE_SHEET_COLUMN_CATALOG: &[ScreenerColumn] = &[
    ScreenerColumn {
        key: "star",
        label: "★",
        sortable: false,
    },
    ScreenerColumn {
        key: "symbol",
        label: "Ticker",
        sortable: true,
    },
    ScreenerColumn {
        key: "debt_to_equity",
        label: "Debt/Equity",
        sortable: true,
    },
    ScreenerColumn {
        key: "current_ratio",
        label: "Current Ratio",
        sortable: true,
    },
    ScreenerColumn {
        key: "interest_coverage",
        label: "Int Coverage",
        sortable: true,
    },
    ScreenerColumn {
        key: "book_value_per_share",
        label: "Book Value ₹",
        sortable: true,
    },
];

pub const CASH_FLOW_COLUMN_CATALOG: &[ScreenerColumn] = &[
    ScreenerColumn {
        key: "star",
        label: "★",
        sortable: false,
    },
    ScreenerColumn {
        key: "symbol",
        label: "Ticker",
        sortable: true,
    },
    ScreenerColumn {
        key: "fcf_cr",
        label: "Free CF ₹Cr",
        sortable: true,
    },
];

pub const PER_SHARE_COLUMN_CATALOG: &[ScreenerColumn] = &[
    ScreenerColumn {
        key: "star",
        label: "★",
        sortable: false,
    },
    ScreenerColumn {
        key: "symbol",
        label: "Ticker",
        sortable: true,
    },
    ScreenerColumn {
        key: "eps",
        label: "EPS ₹",
        sortable: true,
    },
    ScreenerColumn {
        key: "book_value_per_share",
        label: "Book Value ₹",
        sortable: true,
    },
    ScreenerColumn {
        key: "dividend_per_share",
        label: "DPS ₹",
        sortable: true,
    },
];

pub const FORECAST_COLUMN_CATALOG: &[ScreenerColumn] = &[
    ScreenerColumn {
        key: "star",
        label: "★",
        sortable: false,
    },
    ScreenerColumn {
        key: "symbol",
        label: "Ticker",
        sortable: true,
    },
    ScreenerColumn {
        key: "forward_pe",
        label: "Fwd P/E",
        sortable: true,
    },
    ScreenerColumn {
        key: "peg_ratio",
        label: "PEG",
        sortable: true,
    },
];

/// Column slice for one view tab. "overview", "" and unknown keys return the
/// stock `COLUMN_CATALOG` itself (byte-for-byte legacy behavior).
pub fn columns_for_tab(tab: &str) -> &'static [ScreenerColumn] {
    match tab {
        "performance" => PERFORMANCE_COLUMN_CATALOG,
        "technicals" => TECHNICAL_COLUMN_CATALOG,
        "valuation" => VALUATION_COLUMN_CATALOG,
        "dividends" => DIVIDEND_COLUMN_CATALOG,
        "profitability" => PROFITABILITY_COLUMN_CATALOG,
        "financials" => FINANCIAL_COLUMN_CATALOG,
        "income_statement" => INCOME_STATEMENT_COLUMN_CATALOG,
        "balance_sheet" => BALANCE_SHEET_COLUMN_CATALOG,
        "cash_flow" => CASH_FLOW_COLUMN_CATALOG,
        "per_share" => PER_SHARE_COLUMN_CATALOG,
        "forecasts" => FORECAST_COLUMN_CATALOG,
        _ => COLUMN_CATALOG,
    }
}

/// Dedup-union of every catalog key in deterministic seed order: stock
/// `COLUMN_CATALOG` first, then each tab catalog, then the ETF/Bond/MF
/// catalogs via `catalog_for`. This seeds the page's column flags and
/// master display order.
pub fn all_master_column_keys() -> Vec<&'static str> {
    let mut keys: Vec<&'static str> = Vec::new();
    for catalog in [
        COLUMN_CATALOG,
        PERFORMANCE_COLUMN_CATALOG,
        TECHNICAL_COLUMN_CATALOG,
        VALUATION_COLUMN_CATALOG,
        DIVIDEND_COLUMN_CATALOG,
        PROFITABILITY_COLUMN_CATALOG,
        FINANCIAL_COLUMN_CATALOG,
        INCOME_STATEMENT_COLUMN_CATALOG,
        BALANCE_SHEET_COLUMN_CATALOG,
        CASH_FLOW_COLUMN_CATALOG,
        PER_SHARE_COLUMN_CATALOG,
        FORECAST_COLUMN_CATALOG,
        ETF_COLUMN_CATALOG,
        BOND_COLUMN_CATALOG,
        MF_COLUMN_CATALOG,
    ] {
        for col in catalog {
            if !keys.contains(&col.key) {
                keys.push(col.key);
            }
        }
    }
    keys
}

/// Screener filter state. A `Decimal` bound of `ZERO` means "unset".
/// Empty string or `"ALL"` means "no restriction" for the categorical fields.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FilterCriteria {
    pub search_query: String,
    pub exchange: String,
    pub sector: String,
    pub market_cap_category: String,
    /// Market index membership key ("" / "ALL" / unknown key = no restriction),
    /// resolved against `screener::indices::INDEX_CATALOG`.
    #[serde(default)]
    pub market_index: String,
    pub min_price: Decimal,
    pub max_price: Decimal,
    pub min_pe: Decimal,
    pub max_pe: Decimal,
    pub min_rvol: Decimal,
    pub min_rsi: Decimal,
    pub max_rsi: Decimal,
    pub fno_only: bool,
    pub watchlist_only: bool,
}

impl FilterCriteria {
    /// `""` (the `Default`) and `"ALL"` both mean "no restriction".
    fn is_unset(value: &str) -> bool {
        value.is_empty() || value == "ALL"
    }

    /// `market_index` counts as an active restriction only when it names a
    /// *known* catalog benchmark; unknown keys degrade to "no restriction".
    fn index_is_active(&self) -> bool {
        !Self::is_unset(&self.market_index) && benchmark_for(&self.market_index).is_some()
    }

    pub fn matches(&self, record: &ScreenerEquityRecord) -> bool {
        let query = self.search_query.trim().to_lowercase();
        if !query.is_empty() {
            let symbol = record.symbol.to_lowercase();
            let name = record.name.to_lowercase();
            if !symbol.contains(&query) && !name.contains(&query) {
                return false;
            }
        }
        if !Self::is_unset(&self.exchange) && record.exchange != self.exchange {
            return false;
        }
        if !Self::is_unset(&self.sector) && record.sector != self.sector {
            return false;
        }
        if !Self::is_unset(&self.market_cap_category)
            && record.market_cap_category != self.market_cap_category
        {
            return false;
        }
        if self.index_is_active() && !index_contains(&self.market_index, &record.symbol) {
            return false;
        }
        if self.min_price > Decimal::ZERO && record.price < self.min_price {
            return false;
        }
        if self.max_price > Decimal::ZERO && record.price > self.max_price {
            return false;
        }
        if self.min_pe > Decimal::ZERO && record.pe_ratio < self.min_pe {
            return false;
        }
        if self.max_pe > Decimal::ZERO && record.pe_ratio > self.max_pe {
            return false;
        }
        if self.min_rvol > Decimal::ZERO && record.rvol < self.min_rvol {
            return false;
        }
        if self.min_rsi > Decimal::ZERO && record.rsi_14 < self.min_rsi {
            return false;
        }
        if self.max_rsi > Decimal::ZERO && record.rsi_14 > self.max_rsi {
            return false;
        }
        if self.fno_only && !record.is_fno {
            return false;
        }
        if self.watchlist_only && !record.is_watchlist {
            return false;
        }
        true
    }

    pub fn active_count(&self) -> usize {
        let mut count = 0;
        if self.search_query.chars().next().is_some() {
            count += 1;
        }
        if !Self::is_unset(&self.exchange) {
            count += 1;
        }
        if !Self::is_unset(&self.sector) {
            count += 1;
        }
        if !Self::is_unset(&self.market_cap_category) {
            count += 1;
        }
        if self.index_is_active() {
            count += 1;
        }
        if self.min_price > Decimal::ZERO {
            count += 1;
        }
        if self.max_price > Decimal::ZERO {
            count += 1;
        }
        if self.min_pe > Decimal::ZERO {
            count += 1;
        }
        if self.max_pe > Decimal::ZERO {
            count += 1;
        }
        if self.min_rvol > Decimal::ZERO {
            count += 1;
        }
        if self.min_rsi > Decimal::ZERO {
            count += 1;
        }
        if self.max_rsi > Decimal::ZERO {
            count += 1;
        }
        if self.fno_only {
            count += 1;
        }
        if self.watchlist_only {
            count += 1;
        }
        count
    }
}

/// One row of the ETF screener universe: index tracking, premium/discount
/// against NAV, fund size, cost and flow metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenerEtfRecord {
    pub symbol: String,
    pub name: String,
    pub underlying_index: String,
    pub nav: Decimal,
    pub ltp: Decimal,
    pub premium_discount_pct: Decimal,
    pub aum_cr: Decimal,
    pub expense_ratio: Decimal,
    pub tracking_error: Decimal,
    pub return_1y: Decimal,
    pub volume: u64,
}

/// One row of the bond screener universe: coupon/yield economics, clean price
/// against face value, maturity profile and credit quality.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenerBondRecord {
    pub isin: String,
    pub issuer_name: String,
    pub bond_type: String,
    pub coupon_rate: Decimal,
    pub yield_to_maturity: Decimal,
    pub face_value: Decimal,
    pub clean_price: Decimal,
    pub maturity_date: String,
    pub modified_duration: Decimal,
    pub credit_rating: String,
}

/// One row of the mutual-fund screener universe: NAV, fund size, cost and
/// trailing CAGR snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenerMfRecord {
    pub scheme_code: String,
    pub scheme_name: String,
    pub category: String,
    pub nav: Decimal,
    pub aum_cr: Decimal,
    pub expense_ratio: Decimal,
    pub cagr_3y: Decimal,
    pub cagr_5y: Decimal,
    pub risk_grade: String,
}

/// Tagged union over every screener asset class; the table layer renders one
/// heterogeneous list through the accessors below.
// Locked record union. No `Box<…>`: the screener pipeline clones records per
// filter/sort pass and boxing the equity arm would add an allocation per row;
// the size skew is inherent to the 50+ metric equity contract (doc §4.2).
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenerRecord {
    Stock(ScreenerEquityRecord),
    Etf(ScreenerEtfRecord),
    Bond(ScreenerBondRecord),
    Mf(ScreenerMfRecord),
}

impl ScreenerAssetType {
    /// Maps the filter-bar segment values ("stocks"/"etf"/"bonds"/"mf"); unknown -> Stocks.
    pub fn from_bar(value: &str) -> Self {
        match value {
            "etf" => ScreenerAssetType::Etf,
            "bonds" => ScreenerAssetType::Bonds,
            "mf" => ScreenerAssetType::MutualFunds,
            _ => ScreenerAssetType::Stocks,
        }
    }
}

pub const ETF_COLUMN_CATALOG: &[ScreenerColumn] = &[
    ScreenerColumn {
        key: "symbol",
        label: "Ticker",
        sortable: true,
    },
    ScreenerColumn {
        key: "nav",
        label: "NAV",
        sortable: true,
    },
    ScreenerColumn {
        key: "ltp",
        label: "LTP",
        sortable: true,
    },
    ScreenerColumn {
        key: "premium_discount_pct",
        label: "Prem/Disc",
        sortable: true,
    },
    ScreenerColumn {
        key: "aum_cr",
        label: "AUM",
        sortable: true,
    },
    ScreenerColumn {
        key: "expense_ratio",
        label: "Exp %",
        sortable: true,
    },
    ScreenerColumn {
        key: "tracking_error",
        label: "Track Err",
        sortable: false,
    },
    ScreenerColumn {
        key: "return_1y",
        label: "1Y Ret",
        sortable: true,
    },
    ScreenerColumn {
        key: "volume",
        label: "Vol",
        sortable: true,
    },
];

pub const BOND_COLUMN_CATALOG: &[ScreenerColumn] = &[
    ScreenerColumn {
        key: "symbol",
        label: "ISIN",
        sortable: true,
    },
    ScreenerColumn {
        key: "bond_type",
        label: "Type",
        sortable: false,
    },
    ScreenerColumn {
        key: "coupon_rate",
        label: "Coupon %",
        sortable: true,
    },
    ScreenerColumn {
        key: "yield_to_maturity",
        label: "YTM %",
        sortable: true,
    },
    ScreenerColumn {
        key: "clean_price",
        label: "Price",
        sortable: true,
    },
    ScreenerColumn {
        key: "face_value",
        label: "FV",
        sortable: false,
    },
    ScreenerColumn {
        key: "maturity_date",
        label: "Maturity",
        sortable: true,
    },
    ScreenerColumn {
        key: "modified_duration",
        label: "Mod Dur",
        sortable: true,
    },
    ScreenerColumn {
        key: "credit_rating",
        label: "Rating",
        sortable: false,
    },
];

pub const MF_COLUMN_CATALOG: &[ScreenerColumn] = &[
    ScreenerColumn {
        key: "symbol",
        label: "Code",
        sortable: true,
    },
    ScreenerColumn {
        key: "category",
        label: "Category",
        sortable: false,
    },
    ScreenerColumn {
        key: "nav",
        label: "NAV",
        sortable: true,
    },
    ScreenerColumn {
        key: "aum_cr",
        label: "AUM",
        sortable: true,
    },
    ScreenerColumn {
        key: "expense_ratio",
        label: "Exp %",
        sortable: true,
    },
    ScreenerColumn {
        key: "cagr_3y",
        label: "3Y CAGR",
        sortable: true,
    },
    ScreenerColumn {
        key: "cagr_5y",
        label: "5Y CAGR",
        sortable: true,
    },
    ScreenerColumn {
        key: "risk_grade",
        label: "Risk",
        sortable: false,
    },
];

/// Column catalog for the active asset segment (identity columns included).
pub fn catalog_for(asset: ScreenerAssetType) -> &'static [ScreenerColumn] {
    match asset {
        ScreenerAssetType::Stocks => COLUMN_CATALOG,
        ScreenerAssetType::Etf => ETF_COLUMN_CATALOG,
        ScreenerAssetType::Bonds => BOND_COLUMN_CATALOG,
        ScreenerAssetType::MutualFunds => MF_COLUMN_CATALOG,
    }
}

impl ScreenerRecord {
    /// Identity string: ticker / ISIN / scheme code.
    pub fn symbol(&self) -> &str {
        match self {
            ScreenerRecord::Stock(record) => &record.symbol,
            ScreenerRecord::Etf(record) => &record.symbol,
            ScreenerRecord::Bond(record) => &record.isin,
            ScreenerRecord::Mf(record) => &record.scheme_code,
        }
    }

    /// Display name: company / issuer / scheme name.
    pub fn name(&self) -> &str {
        match self {
            ScreenerRecord::Stock(record) => &record.name,
            ScreenerRecord::Etf(record) => &record.name,
            ScreenerRecord::Bond(record) => &record.issuer_name,
            ScreenerRecord::Mf(record) => &record.scheme_name,
        }
    }

    /// Last traded / clean / NAV price depending on the asset class.
    pub fn price(&self) -> Decimal {
        match self {
            ScreenerRecord::Stock(record) => record.price,
            ScreenerRecord::Etf(record) => record.ltp,
            ScreenerRecord::Bond(record) => record.clean_price,
            ScreenerRecord::Mf(record) => record.nav,
        }
    }

    /// Daily change for quote-driven rows; premium/discount for ETFs; zero
    /// for bonds and mutual funds (no intraday change on this surface).
    pub fn change_pct(&self) -> Decimal {
        match self {
            ScreenerRecord::Stock(record) => record.change_pct,
            ScreenerRecord::Etf(record) => record.premium_discount_pct,
            ScreenerRecord::Bond(_) => Decimal::ZERO,
            ScreenerRecord::Mf(_) => Decimal::ZERO,
        }
    }

    /// Traded volume; bonds and mutual funds report zero on this surface.
    pub fn volume(&self) -> u64 {
        match self {
            ScreenerRecord::Stock(record) => record.volume,
            ScreenerRecord::Etf(record) => record.volume,
            ScreenerRecord::Bond(_) => 0,
            ScreenerRecord::Mf(_) => 0,
        }
    }

    /// Decimal sort value for the given column `key`; None if the key is not a
    /// numeric sort key of this record kind.
    pub fn sort_decimal(&self, key: &str) -> Option<Decimal> {
        let inner: Decimal = match (self, key) {
            (ScreenerRecord::Stock(r), "price") => r.price,
            (ScreenerRecord::Stock(r), "change_pct") => r.change_pct,
            (ScreenerRecord::Stock(r), "volume") => Decimal::from(r.volume),
            (ScreenerRecord::Stock(r), "market_cap_cr") => r.market_cap_cr,
            (ScreenerRecord::Stock(r), "pe_ratio") => r.pe_ratio,
            (ScreenerRecord::Stock(r), "return_1d") => r.return_1d,
            (ScreenerRecord::Stock(r), "return_1w") => r.return_1w,
            (ScreenerRecord::Stock(r), "return_1m") => r.return_1m,
            (ScreenerRecord::Stock(r), "return_3m") => r.return_3m,
            (ScreenerRecord::Stock(r), "return_6m") => r.return_6m,
            (ScreenerRecord::Stock(r), "return_ytd") => r.return_ytd,
            (ScreenerRecord::Stock(r), "return_1y") => r.return_1y,
            (ScreenerRecord::Stock(r), "return_3y") => r.return_3y,
            (ScreenerRecord::Stock(r), "atr_pct") => r.atr_pct,
            (ScreenerRecord::Stock(r), "rsi_14") => r.rsi_14,
            (ScreenerRecord::Stock(r), "macd_hist") => r.macd_hist,
            (ScreenerRecord::Stock(r), "supertrend_direction") => {
                Decimal::from(r.supertrend_direction)
            }
            (ScreenerRecord::Stock(r), "sma_20") => r.sma_20,
            (ScreenerRecord::Stock(r), "sma_50") => r.sma_50,
            (ScreenerRecord::Stock(r), "sma_200") => r.sma_200,
            (ScreenerRecord::Stock(r), "ema_9") => r.ema_9,
            (ScreenerRecord::Stock(r), "ema_21") => r.ema_21,
            (ScreenerRecord::Stock(r), "bb_bandwidth") => r.bb_bandwidth,
            (ScreenerRecord::Stock(r), "forward_pe") => r.forward_pe,
            (ScreenerRecord::Stock(r), "pb_ratio") => r.pb_ratio,
            (ScreenerRecord::Stock(r), "ps_ratio") => r.ps_ratio,
            (ScreenerRecord::Stock(r), "ev_ebitda") => r.ev_ebitda,
            (ScreenerRecord::Stock(r), "peg_ratio") => r.peg_ratio,
            (ScreenerRecord::Stock(r), "dividend_yield") => r.dividend_yield,
            (ScreenerRecord::Stock(r), "payout_ratio") => r.payout_ratio,
            (ScreenerRecord::Stock(r), "dividend_per_share") => r.dividend_per_share,
            (ScreenerRecord::Stock(r), "roe") => r.roe,
            (ScreenerRecord::Stock(r), "roce") => r.roce,
            (ScreenerRecord::Stock(r), "operating_margin") => r.operating_margin,
            (ScreenerRecord::Stock(r), "net_margin") => r.net_margin,
            (ScreenerRecord::Stock(r), "revenue_cr") => r.revenue_cr,
            (ScreenerRecord::Stock(r), "revenue_growth_yoy") => r.revenue_growth_yoy,
            (ScreenerRecord::Stock(r), "net_profit_cr") => r.net_profit_cr,
            (ScreenerRecord::Stock(r), "eps") => r.eps,
            (ScreenerRecord::Stock(r), "debt_to_equity") => r.debt_to_equity,
            (ScreenerRecord::Stock(r), "current_ratio") => r.current_ratio,
            (ScreenerRecord::Stock(r), "interest_coverage") => r.interest_coverage,
            (ScreenerRecord::Stock(r), "book_value_per_share") => r.book_value_per_share,
            (ScreenerRecord::Stock(r), "fcf_cr") => r.fcf_cr,
            (ScreenerRecord::Etf(r), "ltp") => r.ltp,
            (ScreenerRecord::Etf(r), "nav") => r.nav,
            (ScreenerRecord::Etf(r), "premium_discount_pct") => r.premium_discount_pct,
            (ScreenerRecord::Etf(r), "aum_cr") => r.aum_cr,
            (ScreenerRecord::Etf(r), "expense_ratio") => r.expense_ratio,
            (ScreenerRecord::Etf(r), "tracking_error") => r.tracking_error,
            (ScreenerRecord::Etf(r), "return_1y") => r.return_1y,
            (ScreenerRecord::Etf(r), "volume") => Decimal::from(r.volume),
            (ScreenerRecord::Bond(r), "coupon_rate") => r.coupon_rate,
            (ScreenerRecord::Bond(r), "yield_to_maturity") => r.yield_to_maturity,
            (ScreenerRecord::Bond(r), "clean_price") => r.clean_price,
            (ScreenerRecord::Bond(r), "face_value") => r.face_value,
            (ScreenerRecord::Bond(r), "modified_duration") => r.modified_duration,
            (ScreenerRecord::Mf(r), "nav") => r.nav,
            (ScreenerRecord::Mf(r), "aum_cr") => r.aum_cr,
            (ScreenerRecord::Mf(r), "expense_ratio") => r.expense_ratio,
            (ScreenerRecord::Mf(r), "cagr_3y") => r.cagr_3y,
            (ScreenerRecord::Mf(r), "cagr_5y") => r.cagr_5y,
            _ => return None,
        };
        Some(inner)
    }
}
