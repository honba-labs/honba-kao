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

/// Screener filter state. A `Decimal` bound of `ZERO` means "unset".
/// Empty string or `"ALL"` means "no restriction" for the categorical fields.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FilterCriteria {
    pub search_query: String,
    pub exchange: String,
    pub sector: String,
    pub market_cap_category: String,
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
