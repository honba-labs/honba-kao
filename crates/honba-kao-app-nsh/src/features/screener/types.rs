//! Screener types — the zero-float `Decimal` financial models global to
//! the screener feature (screener-design-doc.md §4.2), plus the pure
//! formatting helpers the grid cells and the detail drawer share. No
//! component, no I/O, no floats in a financial path: the only `f64` in
//! the room is the sparkline's canvas feed, which fixture builders
//! produce from `Decimal` closes at the module boundary.

use num_traits::ToPrimitive as _;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// The asset class the grid scans: the filter bar's segmented switcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScreenerAssetType {
    #[default]
    Stocks,
    Etf,
    Bonds,
    MutualFunds,
}

impl ScreenerAssetType {
    pub const ALL: &'static [ScreenerAssetType] = &[
        ScreenerAssetType::Stocks,
        ScreenerAssetType::Etf,
        ScreenerAssetType::Bonds,
        ScreenerAssetType::MutualFunds,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ScreenerAssetType::Stocks => "Stocks",
            ScreenerAssetType::Etf => "ETF",
            ScreenerAssetType::Bonds => "Bonds",
            ScreenerAssetType::MutualFunds => "MF",
        }
    }
}

/// A quick-screen preset: one pressed pill on the top shelf.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum QuickPreset {
    #[default]
    All,
    Gainers,
    Losers,
    MostCapitalized,
    Momentum,
    Value,
    Dividend,
    MostActive,
    High52,
    Oversold,
    Overbought,
    Watchlist,
}

impl QuickPreset {
    pub const ALL: &'static [QuickPreset] = &[
        QuickPreset::All,
        QuickPreset::Gainers,
        QuickPreset::Losers,
        QuickPreset::MostCapitalized,
        QuickPreset::Momentum,
        QuickPreset::Value,
        QuickPreset::Dividend,
        QuickPreset::MostActive,
        QuickPreset::High52,
        QuickPreset::Oversold,
        QuickPreset::Overbought,
        QuickPreset::Watchlist,
    ];

    pub fn label(self) -> &'static str {
        match self {
            QuickPreset::All => "All stocks",
            QuickPreset::Gainers => "Top gainers",
            QuickPreset::Losers => "Top losers",
            QuickPreset::MostCapitalized => "Most capitalized",
            QuickPreset::Momentum => "Bullish momentum",
            QuickPreset::Value => "Value (P/E < 25)",
            QuickPreset::Dividend => "High dividend (>1.5%)",
            QuickPreset::MostActive => "Volume leaders",
            QuickPreset::High52 => "52W high",
            QuickPreset::Oversold => "Oversold RSI (<35)",
            QuickPreset::Overbought => "Overbought RSI (>70)",
            QuickPreset::Watchlist => "Watchlist",
        }
    }

    /// Whether pressing this preset again should reset the shelf to
    /// [`QuickPreset::All`].
    pub fn is_resettable(self) -> bool {
        self != QuickPreset::All
    }
}

/// The category tab above the toolbar; each tab owns its column set
/// (design doc §5.5 — 12 tabs at full parity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScreenerViewTab {
    #[default]
    Overview,
    Performance,
    Technicals,
    Valuation,
    Profitability,
    Dividends,
    Financials,
    IncomeStatement,
    BalanceSheet,
    CashFlow,
    PerShare,
    Forecasts,
}

impl ScreenerViewTab {
    pub const ALL: &'static [ScreenerViewTab] = &[
        ScreenerViewTab::Overview,
        ScreenerViewTab::Performance,
        ScreenerViewTab::Technicals,
        ScreenerViewTab::Valuation,
        ScreenerViewTab::Profitability,
        ScreenerViewTab::Dividends,
        ScreenerViewTab::Financials,
        ScreenerViewTab::IncomeStatement,
        ScreenerViewTab::BalanceSheet,
        ScreenerViewTab::CashFlow,
        ScreenerViewTab::PerShare,
        ScreenerViewTab::Forecasts,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ScreenerViewTab::Overview => "Overview",
            ScreenerViewTab::Performance => "Performance",
            ScreenerViewTab::Technicals => "Technicals",
            ScreenerViewTab::Valuation => "Valuation",
            ScreenerViewTab::Profitability => "Profitability",
            ScreenerViewTab::Dividends => "Dividends",
            ScreenerViewTab::Financials => "Financials",
            ScreenerViewTab::IncomeStatement => "Income Statement",
            ScreenerViewTab::BalanceSheet => "Balance Sheet",
            ScreenerViewTab::CashFlow => "Cash Flow",
            ScreenerViewTab::PerShare => "Per Share",
            ScreenerViewTab::Forecasts => "Forecasts",
        }
    }
}

/// The five-tier technical consensus the meter and badges share.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechnicalRating {
    StrongBuy,
    Buy,
    Neutral,
    Sell,
    StrongSell,
}

impl TechnicalRating {
    /// A canonical 5-tier ordinal: 4 = Strong Buy … 0 = Strong Sell.
    /// Used for sorting ("Rating" column) and the gauge's active index.
    pub fn rank(self) -> u8 {
        match self {
            TechnicalRating::StrongSell => 0,
            TechnicalRating::Sell => 1,
            TechnicalRating::Neutral => 2,
            TechnicalRating::Buy => 3,
            TechnicalRating::StrongBuy => 4,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            TechnicalRating::StrongBuy => "Strong Buy",
            TechnicalRating::Buy => "Buy",
            TechnicalRating::Neutral => "Neutral",
            TechnicalRating::Sell => "Sell",
            TechnicalRating::StrongSell => "Strong Sell",
        }
    }

    /// `(buy, sell)` indicator counts the consensus meter summarizes
    /// ("Bullish: {buy} Buy indicators, {sell} Sell indicators").
    pub fn counts(self) -> (u32, u32) {
        match self {
            TechnicalRating::StrongBuy => (17, 1),
            TechnicalRating::Buy => (14, 2),
            TechnicalRating::Neutral => (7, 7),
            TechnicalRating::Sell => (4, 14),
            TechnicalRating::StrongSell => (1, 17),
        }
    }

    /// Parses a fixture spelling ("StrongBuy", "Neutral", …).
    pub fn parse(text: &str) -> TechnicalRating {
        match text {
            "StrongBuy" => TechnicalRating::StrongBuy,
            "Buy" => TechnicalRating::Buy,
            "Sell" => TechnicalRating::Sell,
            "StrongSell" => TechnicalRating::StrongSell,
            _ => TechnicalRating::Neutral,
        }
    }
}

/// The market-cap tier the quick cap pill and the filter modal share.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MarketCapTier {
    #[default]
    All,
    Mega,
    Large,
    Mid,
    Small,
}

impl MarketCapTier {
    pub const ALL: &'static [MarketCapTier] = &[
        MarketCapTier::All,
        MarketCapTier::Mega,
        MarketCapTier::Large,
        MarketCapTier::Mid,
        MarketCapTier::Small,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MarketCapTier::All => "All",
            MarketCapTier::Mega => "Mega Cap",
            MarketCapTier::Large => "Large Cap",
            MarketCapTier::Mid => "Mid Cap",
            MarketCapTier::Small => "Small Cap",
        }
    }

    /// The tier's lower bound in ₹ crore (`None` = no lower bound).
    pub fn min_cap_cr(self) -> Option<Decimal> {
        match self {
            MarketCapTier::All => None,
            MarketCapTier::Mega => Some(dec!(200_000)),
            MarketCapTier::Large => Some(dec!(50_000)),
            MarketCapTier::Mid => Some(dec!(10_000)),
            MarketCapTier::Small => None,
        }
    }

    /// Whether a ₹-crore market cap falls in this tier's band
    /// (Mega ≥ 2,00,000; Large 50,000–2,00,000; Mid 10,000–50,000;
    /// Small < 10,000).
    pub fn contains(self, cap_cr: Decimal) -> bool {
        match self {
            MarketCapTier::All => true,
            MarketCapTier::Mega => cap_cr >= dec!(200_000),
            MarketCapTier::Large => cap_cr >= dec!(50_000) && cap_cr < dec!(200_000),
            MarketCapTier::Mid => cap_cr >= dec!(10_000) && cap_cr < dec!(50_000),
            MarketCapTier::Small => cap_cr < dec!(10_000),
        }
    }
}

/// A complete financial screener instrument record (design doc §4.2,
/// extended with `day_low`/`day_high` per §5.3's live quote box).
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenerInstrument {
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub exchange: String,
    pub price: Decimal,
    pub change: Decimal,
    pub change_percent: Decimal,
    pub day_low: Decimal,
    pub day_high: Decimal,
    pub volume: i64,
    pub avg_volume_30d: i64,
    pub market_cap_cr: Decimal,
    pub pe_ratio: Option<Decimal>,
    pub pb_ratio: Option<Decimal>,
    pub roe: Option<Decimal>,
    pub roce: Option<Decimal>,
    pub net_margin: Option<Decimal>,
    pub dividend_yield: Option<Decimal>,
    pub rsi_14: Decimal,
    pub sma_20: Decimal,
    pub sma_50: Decimal,
    pub sma_200: Decimal,
    pub high_52: Decimal,
    pub low_52: Decimal,
    pub perf_1w: Decimal,
    pub perf_1m: Decimal,
    pub perf_3m: Decimal,
    pub perf_1y: Decimal,
    pub perf_ytd: Decimal,
    pub technical_rating: TechnicalRating,
    // ── Statement metrics (design doc §5.5), all `Option<Decimal>` so a
    // missing statement line renders an em dash instead of a zero.
    pub payout_ratio: Option<Decimal>,
    pub dividend_per_share: Option<Decimal>,
    pub dps_growth_5y: Option<Decimal>,
    pub revenue_cr: Option<Decimal>,
    pub revenue_growth_yoy: Option<Decimal>,
    pub ebitda_cr: Option<Decimal>,
    pub pat_cr: Option<Decimal>,
    pub eps: Option<Decimal>,
    pub total_debt_cr: Option<Decimal>,
    pub debt_to_equity: Option<Decimal>,
    pub current_ratio: Option<Decimal>,
    pub book_value_per_share: Option<Decimal>,
    pub operating_cf_cr: Option<Decimal>,
    pub free_cf_cr: Option<Decimal>,
    pub fcf_yield_pct: Option<Decimal>,
    pub cash_per_share: Option<Decimal>,
    pub sales_per_share: Option<Decimal>,
    /// Canvas-only feed (sparkline), converted from `Decimal` closes at
    /// the fixture boundary; never a financial path.
    pub sparkline_closes: Vec<f64>,
}

impl ScreenerInstrument {
    /// Whether the day's delta is a rise (green `▲`) or a fall.
    pub fn change_up(&self) -> bool {
        self.change > Decimal::ZERO
    }

    /// The instrument's position between its 52W low and high, in
    /// `[0.0, 1.0]` — the range bar cell's fill ratio (layout float,
    /// never a financial path).
    pub fn range_position(&self) -> f32 {
        let spread = self.high_52 - self.low_52;
        if spread <= Decimal::ZERO {
            return 0.0;
        }
        let ratio = (self.price - self.low_52) / spread;
        let clamped = ratio.max(Decimal::ZERO).min(Decimal::ONE);
        clamped.to_f64().unwrap_or(0.0) as f32
    }
}

/// The granular filter state (design doc §4.2): empty bounds pass;
/// `sector == "All"` or `""` means no sector filter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FilterCriteria {
    pub sector: String,
    pub market_cap_tier: MarketCapTier,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    pub min_change_pct: Option<Decimal>,
    pub max_change_pct: Option<Decimal>,
    pub min_volume: Option<i64>,
    pub min_pe: Option<Decimal>,
    pub max_pe: Option<Decimal>,
    pub max_pb: Option<Decimal>,
    pub min_dividend: Option<Decimal>,
    pub min_rsi: Option<Decimal>,
    pub max_rsi: Option<Decimal>,
    pub min_roe: Option<Decimal>,
    pub min_fcf_yield: Option<Decimal>,
}

impl FilterCriteria {
    /// How many bounds are active (the "Filters (N)" badge counter).
    pub fn active_count(&self) -> usize {
        let mut n = 0;
        if !self.sector.is_empty() && self.sector != "All" {
            n += 1;
        }
        if self.market_cap_tier != MarketCapTier::All {
            n += 1;
        }
        for bound in [
            self.min_price,
            self.max_price,
            self.min_change_pct,
            self.max_change_pct,
            self.min_pe,
            self.max_pe,
            self.max_pb,
            self.min_dividend,
            self.min_rsi,
            self.max_rsi,
            self.min_roe,
            self.min_fcf_yield,
        ] {
            if bound.is_some() {
                n += 1;
            }
        }
        if self.min_volume.is_some() {
            n += 1;
        }
        n
    }
}

/// One column's stable id + display label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenerColumn {
    pub id: &'static str,
    pub label: &'static str,
}

impl ScreenerColumn {
    pub const fn new(id: &'static str, label: &'static str) -> Self {
        ScreenerColumn { id, label }
    }
}

/// The column set of one category tab (design doc §5.2); the star
/// (shortlist) column is the grid's fixed leading one, outside this.
pub const fn tab_columns(tab: ScreenerViewTab) -> &'static [ScreenerColumn] {
    const OVERVIEW: [ScreenerColumn; 9] = [
        ScreenerColumn::new("symbol", "Symbol"),
        ScreenerColumn::new("price", "Price"),
        ScreenerColumn::new("change_percent", "Chg %"),
        ScreenerColumn::new("volume", "Volume"),
        ScreenerColumn::new("market_cap_cr", "Market Cap"),
        ScreenerColumn::new("pe_ratio", "P/E"),
        ScreenerColumn::new("range_52w", "52W Range"),
        ScreenerColumn::new("trend", "Trend"),
        ScreenerColumn::new("rating", "Rating"),
    ];
    const PERFORMANCE: [ScreenerColumn; 8] = [
        ScreenerColumn::new("symbol", "Symbol"),
        ScreenerColumn::new("price", "Price"),
        ScreenerColumn::new("change_percent", "1D %"),
        ScreenerColumn::new("perf_1w", "1W %"),
        ScreenerColumn::new("perf_1m", "1M %"),
        ScreenerColumn::new("perf_3m", "3M %"),
        ScreenerColumn::new("perf_1y", "1Y %"),
        ScreenerColumn::new("perf_ytd", "YTD %"),
    ];
    const TECHNICALS: [ScreenerColumn; 7] = [
        ScreenerColumn::new("symbol", "Symbol"),
        ScreenerColumn::new("price", "Price"),
        ScreenerColumn::new("rsi_14", "RSI(14)"),
        ScreenerColumn::new("sma_20", "SMA 20"),
        ScreenerColumn::new("sma_50", "SMA 50"),
        ScreenerColumn::new("sma_200", "SMA 200"),
        ScreenerColumn::new("rating", "Rating"),
    ];
    const VALUATION: [ScreenerColumn; 6] = [
        ScreenerColumn::new("symbol", "Symbol"),
        ScreenerColumn::new("price", "Price"),
        ScreenerColumn::new("market_cap_cr", "Market Cap"),
        ScreenerColumn::new("pe_ratio", "P/E"),
        ScreenerColumn::new("pb_ratio", "P/B"),
        ScreenerColumn::new("dividend_yield", "Div Yield %"),
    ];
    const PROFITABILITY: [ScreenerColumn; 5] = [
        ScreenerColumn::new("symbol", "Symbol"),
        ScreenerColumn::new("roe", "ROE %"),
        ScreenerColumn::new("roce", "ROCE %"),
        ScreenerColumn::new("net_margin", "Net Margin %"),
        ScreenerColumn::new("pe_ratio", "P/E"),
    ];
    const DIVIDENDS: [ScreenerColumn; 5] = [
        ScreenerColumn::new("symbol", "Symbol"),
        ScreenerColumn::new("dividend_yield", "Div Yield %"),
        ScreenerColumn::new("payout_ratio", "Payout %"),
        ScreenerColumn::new("dividend_per_share", "DPS ₹"),
        ScreenerColumn::new("dps_growth_5y", "DPS Growth 5Y %"),
    ];
    const FINANCIALS: [ScreenerColumn; 6] = [
        ScreenerColumn::new("symbol", "Symbol"),
        ScreenerColumn::new("revenue_cr", "Revenue ₹Cr"),
        ScreenerColumn::new("revenue_growth_yoy", "Rev Growth %"),
        ScreenerColumn::new("ebitda_cr", "EBITDA ₹Cr"),
        ScreenerColumn::new("pat_cr", "PAT ₹Cr"),
        ScreenerColumn::new("eps", "EPS ₹"),
    ];
    const INCOME_STATEMENT: [ScreenerColumn; 6] = [
        ScreenerColumn::new("symbol", "Symbol"),
        ScreenerColumn::new("revenue_cr", "Revenue ₹Cr"),
        ScreenerColumn::new("ebitda_cr", "EBITDA ₹Cr"),
        ScreenerColumn::new("pat_cr", "PAT ₹Cr"),
        ScreenerColumn::new("eps", "EPS ₹"),
        ScreenerColumn::new("revenue_growth_yoy", "Rev Growth %"),
    ];
    const BALANCE_SHEET: [ScreenerColumn; 5] = [
        ScreenerColumn::new("symbol", "Symbol"),
        ScreenerColumn::new("total_debt_cr", "Debt ₹Cr"),
        ScreenerColumn::new("debt_to_equity", "Debt/Equity"),
        ScreenerColumn::new("current_ratio", "Current Ratio"),
        ScreenerColumn::new("book_value_per_share", "Book Value ₹"),
    ];
    const CASH_FLOW: [ScreenerColumn; 4] = [
        ScreenerColumn::new("symbol", "Symbol"),
        ScreenerColumn::new("operating_cf_cr", "Op CF ₹Cr"),
        ScreenerColumn::new("free_cf_cr", "Free CF ₹Cr"),
        ScreenerColumn::new("fcf_yield_pct", "FCF Yield %"),
    ];
    const PER_SHARE: [ScreenerColumn; 5] = [
        ScreenerColumn::new("symbol", "Symbol"),
        ScreenerColumn::new("eps", "EPS ₹"),
        ScreenerColumn::new("sales_per_share", "Sales ₹"),
        ScreenerColumn::new("cash_per_share", "Cash ₹"),
        ScreenerColumn::new("book_value_per_share", "Book Value ₹"),
    ];
    // The doc specifies no dedicated forecast metrics; the forecast view
    // re-reads today's fundamentals (forward EPS, growth, quality, yield).
    const FORECASTS: [ScreenerColumn; 5] = [
        ScreenerColumn::new("symbol", "Symbol"),
        ScreenerColumn::new("price", "Price"),
        ScreenerColumn::new("eps", "Fwd EPS ₹"),
        ScreenerColumn::new("revenue_growth_yoy", "Rev Growth %"),
        ScreenerColumn::new("roe", "ROE %"),
    ];
    match tab {
        ScreenerViewTab::Overview => &OVERVIEW,
        ScreenerViewTab::Performance => &PERFORMANCE,
        ScreenerViewTab::Technicals => &TECHNICALS,
        ScreenerViewTab::Valuation => &VALUATION,
        ScreenerViewTab::Profitability => &PROFITABILITY,
        ScreenerViewTab::Dividends => &DIVIDENDS,
        ScreenerViewTab::Financials => &FINANCIALS,
        ScreenerViewTab::IncomeStatement => &INCOME_STATEMENT,
        ScreenerViewTab::BalanceSheet => &BALANCE_SHEET,
        ScreenerViewTab::CashFlow => &CASH_FLOW,
        ScreenerViewTab::PerShare => &PER_SHARE,
        ScreenerViewTab::Forecasts => &FORECASTS,
    }
}

/// A sort's direction; the header's tri-state is
/// `Desc` → `Asc` → `None` (i.e. `Option<SortState>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortState {
    pub column_id: String,
    pub direction: SortDirection,
}

impl SortDirection {
    /// The next direction in the header's `Desc → Asc → None` toggler.
    pub fn next(self) -> Option<SortDirection> {
        match self {
            SortDirection::Desc => Some(SortDirection::Asc),
            SortDirection::Asc => None, // falls back to no sort
        }
    }

    /// The first press' direction for a column.
    pub const fn first() -> SortDirection {
        SortDirection::Desc
    }
}

/// The columns the CSV export includes, in order.
pub const CSV_COLUMNS: [&str; 16] = [
    "SYMBOL",
    "NAME",
    "SECTOR",
    "EXCHANGE",
    "PRICE",
    "CHANGE",
    "CHANGE %",
    "VOLUME",
    "MKT CAP (CR)",
    "P/E",
    "P/B",
    "ROE %",
    "ROCE %",
    "DIV YIELD %",
    "RSI(14)",
    "RATING",
];

// ---------------------------------------------------------------------------
// Pure formatting helpers (Decimal arithmetic only; integer string work).
// ---------------------------------------------------------------------------

fn thousands(int_digits: &str) -> String {
    let mut out = String::new();
    let len = int_digits.chars().count();
    for (i, digit) in int_digits.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// A fixed-precision ₹ value: `₹2,842.50` (negative: `-₹15.60`).
/// Decimals in, formatted string out — never a float.
pub fn inr(d: Decimal) -> String {
    let scaled = d.round_dp(2);
    let sign = if scaled < Decimal::ZERO { "-" } else { "" };
    let abs = scaled.abs();
    let int_decimal = abs.trunc();
    let int_digits = int_decimal.to_string();
    // The two paise digits: an integer shift so no exponent leaks in.
    let paise = (abs - int_decimal) * dec!(100);
    let paise_digits = format!("{:0>2}", paise.round_dp(0).to_string());
    format!("{sign}₹{}.{}", thousands(&int_digits), &paise_digits[..2])
}

/// The day's delta with a ₹ prefix and explicit sign: `+₹52.30`.
pub fn signed_inr(d: Decimal) -> String {
    if d.is_zero() {
        return "₹0.00".to_string();
    }
    if d > Decimal::ZERO {
        format!("+{}", inr(d))
    } else {
        inr(d) // '-₹…' already signed by `inr`
    }
}

/// A percentage text with an explicit sign: `+1.84%`, `-0.40%`, `0.00%`.
pub fn percent_text(d: Decimal) -> String {
    let scaled = d.round_dp(2);
    if scaled.is_zero() {
        "0.00%".to_string()
    } else if scaled > Decimal::ZERO {
        format!("+{}%", scaled)
    } else {
        format!("{}%", scaled)
    }
}

/// One-decimal normalized text: `42.5`, `-0.4`, `12`.
fn fmt_1dp(d: Decimal) -> String {
    let s = d.round_dp(1).to_string();
    if let Some(stripped) = s.strip_suffix(".0") {
        stripped.to_string()
    } else {
        s
    }
}

/// Indian compact volume notation for a share count (units): `42.5L`,
/// `1.2Cr`, `8,400`.
pub fn compact_volume(v: i64) -> String {
    let d = Decimal::from(v);
    if v.abs() >= 10_000_000 {
        format!("{}Cr", fmt_1dp(d / dec!(10_000_000)))
    } else if v.abs() >= 100_000 {
        format!("{}L", fmt_1dp(d / dec!(100_000)))
    } else {
        thousands(&d.round_dp(0).to_string())
    }
}

/// Indian compact market-cap notation for a ₹ crore figure:
/// `₹19.2L Cr`, `₹45,000 Cr`.
pub fn compact_cap_crore(cap_cr: Decimal) -> String {
    if cap_cr >= dec!(100_000) {
        format!("₹{}L Cr", fmt_1dp(cap_cr / dec!(100_000)))
    } else {
        format!("₹{} Cr", thousands(&cap_cr.round_dp(0).to_string()))
    }
}

// ---------------------------------------------------------------------------
// Multi-asset screener records (screener-design-doc.md §4) — zero floats.
// ---------------------------------------------------------------------------

/// One grid row across every asset class: the tagged union the multi-asset
/// screener renders, sorts, and filters uniformly.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // UI enum; the equity row is the fat variant.
pub enum ScreenerRecord {
    /// An equity row (the original [`ScreenerInstrument`]).
    Stock(ScreenerInstrument),
    /// An exchange-traded fund row.
    Etf(ScreenerEtfRecord),
    /// A bond row.
    Bond(ScreenerBondRecord),
    /// A mutual-fund row.
    MutualFund(ScreenerMfRecord),
}

impl ScreenerRecord {
    /// The row's trading symbol (the grid's leading identity cell).
    pub fn symbol(&self) -> &str {
        match self {
            ScreenerRecord::Stock(r) => &r.symbol,
            ScreenerRecord::Etf(r) => &r.symbol,
            ScreenerRecord::Bond(r) => &r.symbol,
            ScreenerRecord::MutualFund(r) => &r.symbol,
        }
    }

    /// The row's human-readable instrument name.
    pub fn name(&self) -> &str {
        match self {
            ScreenerRecord::Stock(r) => &r.name,
            ScreenerRecord::Etf(r) => &r.name,
            ScreenerRecord::Bond(r) => &r.name,
            ScreenerRecord::MutualFund(r) => &r.name,
        }
    }

    /// The headline price of the row: stock price / ETF NAV / bond clean
    /// price / mutual-fund NAV.
    pub fn price(&self) -> Decimal {
        match self {
            ScreenerRecord::Stock(r) => r.price,
            ScreenerRecord::Etf(r) => r.nav,
            ScreenerRecord::Bond(r) => r.clean_price,
            ScreenerRecord::MutualFund(r) => r.nav,
        }
    }

    /// The day's percentage move, shared by every asset class.
    pub fn change_percent(&self) -> Decimal {
        match self {
            ScreenerRecord::Stock(r) => r.change_percent,
            ScreenerRecord::Etf(r) => r.change_percent,
            ScreenerRecord::Bond(r) => r.change_percent,
            ScreenerRecord::MutualFund(r) => r.change_percent,
        }
    }

    /// The asset class this row belongs to (the tab/filter switcher).
    pub fn class(&self) -> ScreenerAssetType {
        match self {
            ScreenerRecord::Stock(_) => ScreenerAssetType::Stocks,
            ScreenerRecord::Etf(_) => ScreenerAssetType::Etf,
            ScreenerRecord::Bond(_) => ScreenerAssetType::Bonds,
            ScreenerRecord::MutualFund(_) => ScreenerAssetType::MutualFunds,
        }
    }
}

/// An ETF screener row: quoted on NAV with AUM, cost, tracking error, and
/// trailing performance — all `Decimal`, never a float.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenerEtfRecord {
    /// Exchange symbol (e.g. `NIFTYBEES`).
    pub symbol: String,
    /// Full fund name.
    pub name: String,
    /// Asset-manager brand (e.g. `Nippon`).
    pub brand: String,
    /// Investment focus (e.g. `Broad Market`).
    pub focus: String,
    /// Latest net asset value per unit.
    pub nav: Decimal,
    /// Absolute NAV change since the previous close.
    pub change: Decimal,
    /// Percentage NAV change since the previous close.
    pub change_percent: Decimal,
    /// Assets under management in ₹ crore.
    pub aum_cr: Decimal,
    /// Annual expense ratio in percent.
    pub expense_ratio: Decimal,
    /// Tracking error versus the underlying index, in percent.
    pub tracking_error: Decimal,
    /// Traded units in the session.
    pub volume: i64,
    /// Trailing one-year return in percent.
    pub perf_1y: Decimal,
    /// Trailing three-year return in percent.
    pub perf_3y: Decimal,
}

/// The bond's credit-rating tier (the bond filter's rating pills).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BondCreditRating {
    /// Government of India sovereign paper.
    Sovereign,
    /// Top-most investment grade (AAA equivalent).
    Aaa,
    /// AA+ tier.
    AaPlus,
    /// AA tier.
    Aa,
    /// A+ tier.
    APlus,
}

impl BondCreditRating {
    /// Every rating, best first — the filter pill row.
    pub const ALL: &'static [BondCreditRating] = &[
        BondCreditRating::Sovereign,
        BondCreditRating::Aaa,
        BondCreditRating::AaPlus,
        BondCreditRating::Aa,
        BondCreditRating::APlus,
    ];

    /// The compact badge text: `SOV`, `AAA`, `AA+`, `AA`, `A+`.
    pub fn label(self) -> &'static str {
        match self {
            BondCreditRating::Sovereign => "SOV",
            BondCreditRating::Aaa => "AAA",
            BondCreditRating::AaPlus => "AA+",
            BondCreditRating::Aa => "AA",
            BondCreditRating::APlus => "A+",
        }
    }
}

/// The bond issuer category (the bond filter's issuer pills).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BondIssuerType {
    /// Government of India paper.
    Sovereign,
    /// Public-sector undertaking (NHAI, REC, PFC, NABARD, HUDCO…).
    CorporatePsu,
    /// Private corporate issuer.
    CorporatePrivate,
    /// Municipal issuer.
    Municipal,
}

impl BondIssuerType {
    /// Every issuer type, best first — the filter pill row.
    pub const ALL: &'static [BondIssuerType] = &[
        BondIssuerType::Sovereign,
        BondIssuerType::CorporatePsu,
        BondIssuerType::CorporatePrivate,
        BondIssuerType::Municipal,
    ];

    /// The pill text: `Sovereign`, `PSU`, `Private`, `Municipal`.
    pub fn label(self) -> &'static str {
        match self {
            BondIssuerType::Sovereign => "Sovereign",
            BondIssuerType::CorporatePsu => "PSU",
            BondIssuerType::CorporatePrivate => "Private",
            BondIssuerType::Municipal => "Municipal",
        }
    }
}

/// The mutual fund's plan/option variant (the MF filter's pills).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MfSchemeType {
    /// Direct plan, growth option.
    DirectGrowth,
    /// Regular plan, growth option.
    RegularGrowth,
    /// Income distribution cum capital withdrawal option.
    Idcw,
}

impl MfSchemeType {
    /// Every scheme type, in canonical order — the filter pill row.
    pub const ALL: &'static [MfSchemeType] = &[
        MfSchemeType::DirectGrowth,
        MfSchemeType::RegularGrowth,
        MfSchemeType::Idcw,
    ];

    /// The pill text: `Direct Growth`, `Regular Growth`, `IDCW`.
    pub fn label(self) -> &'static str {
        match self {
            MfSchemeType::DirectGrowth => "Direct Growth",
            MfSchemeType::RegularGrowth => "Regular Growth",
            MfSchemeType::Idcw => "IDCW",
        }
    }
}

/// A bond screener row: clean price, yield to maturity, and credit
/// metadata for one listed debt security — all `Decimal`, never a float.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenerBondRecord {
    /// Exchange bond symbol (e.g. `GS7182033`).
    pub symbol: String,
    /// Display name (e.g. `GOI 7.18% 2033`).
    pub name: String,
    /// Issuer category for the filter pills.
    pub issuer_type: BondIssuerType,
    /// Credit-rating tier for the filter pills.
    pub credit_rating: BondCreditRating,
    /// Annual coupon rate in percent.
    pub coupon_rate: Decimal,
    /// Yield to maturity in percent.
    pub ytw: Decimal,
    /// Clean (ex-interest) price per ₹100 face.
    pub clean_price: Decimal,
    /// Face value per unit (`dec!(1000)` on NSE).
    pub face_value: Decimal,
    /// Maturity date in `YYYY-MM-DD` form.
    pub maturity_date: String,
    /// Macaulay duration in years.
    pub duration_years: Decimal,
    /// The day's price move in percent.
    pub change_percent: Decimal,
}

/// A mutual-fund screener row: NAV, plan variant, cost, risk, and CAGR
/// history for one scheme — all `Decimal`, never a float.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenerMfRecord {
    /// Scheme shorthand symbol (e.g. `PPFCAP`).
    pub symbol: String,
    /// Full scheme name.
    pub name: String,
    /// Asset-management company.
    pub amc: String,
    /// SEBI category (e.g. `Flexi Cap`).
    pub category: String,
    /// Plan/option variant for the filter pills.
    pub scheme_type: MfSchemeType,
    /// Latest net asset value per unit.
    pub nav: Decimal,
    /// Absolute NAV change since the previous NAV.
    pub change: Decimal,
    /// Percentage NAV change since the previous NAV.
    pub change_percent: Decimal,
    /// Assets under management in ₹ crore.
    pub aum_cr: Decimal,
    /// Annual expense ratio in percent.
    pub expense_ratio: Decimal,
    /// Trailing Sharpe ratio.
    pub sharpe_ratio: Decimal,
    /// One-year CAGR in percent.
    pub cagr_1y: Decimal,
    /// Three-year CAGR in percent.
    pub cagr_3y: Decimal,
    /// Five-year CAGR in percent.
    pub cagr_5y: Decimal,
    /// Morningstar-style star rating (3..=5).
    pub rating_stars: u8,
}

/// A plain number in Indian compact crore notation without the ₹ prefix
/// (AUM of ETFs/MFs): `19.2L Cr` / `45,000 Cr`.
pub fn compact_cr(cr: Decimal) -> String {
    compact_cap_crore(cr).trim_start_matches('₹').to_string()
}
