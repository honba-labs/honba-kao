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
