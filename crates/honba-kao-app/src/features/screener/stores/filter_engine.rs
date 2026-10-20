//! Screener filter engine — the pure, in-memory half of the labs app's
//! `use-screener-store.ts`: advanced-filter state, quick presets, the
//! search → preset → advanced predicate pipeline, sorting special cases and
//! the CSV body builder for the filtered rows.

use serde::{Deserialize, Serialize};

use crate::features::screener::data::indices::instrument_belongs_to_index;
use crate::features::screener::data::instruments::Instrument;

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

pub(super) const ALL: &str = "all";

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::screener::data::instruments;

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
