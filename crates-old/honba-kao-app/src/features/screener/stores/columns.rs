//! Screener static registries — asset classes (`ScreenerType` with its tabs
//! and filter pills) and the column registry (port of the labs app's
//! `core/columns.tsx`) with the per-tab column presets.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::filter_engine::SortDirection;

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
