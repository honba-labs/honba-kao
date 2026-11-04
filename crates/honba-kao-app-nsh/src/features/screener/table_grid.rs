//! The screener's high-density data grid (screener-design-doc.md §5.2):
//! reactive filter evaluation over the fixture universe, tri-state
//! column sorting, the sort key extraction, and the CSV export — plus a
//! public re-export so host-side integration tests reach the pure logic
//! through `honba_kao_app_nsh::features::screener::table_grid::*`.
//!
//! This file carries three layers:
//! 1. pure evaluation logic (this cite-block; no component, no floats),
//! 2. the event/props contract,
//! 3. the `ScreenerTableGrid` component (columns, range-bar cells,
//!    sparklines, rating badges, shortlist stars, row presses).
//!
//! Layer 1 also hosts the multi-asset record layer: [`ScreenerRecord`]
//! rows blend stocks, ETFs, bonds, and mutual funds into one board. Those
//! records get their own filter, sort-key, column-set, and CSV fns that
//! delegate stock rows to the stock fns above and share only the
//! class-agnostic bounds (price and change percent) across classes.

use std::sync::OnceLock;

use crate::features::screener::column_header_menu::{column_filters_active, column_menu};
use crate::features::screener::types::{
    FilterCriteria, QuickPreset, ScreenerAssetType, ScreenerBondRecord, ScreenerColumn,
    ScreenerEtfRecord, ScreenerInstrument, ScreenerMfRecord, ScreenerRecord, ScreenerViewTab,
    SortDirection, SortState, TechnicalRating,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ---------------------------------------------------------------------------
// 1. Pure evaluation logic (Decimal math only)
// ---------------------------------------------------------------------------

/// Applies the active preset, the search text, and the granular criteria
/// to the universe (design doc §6).
pub fn filter_instruments(
    universe: &[ScreenerInstrument],
    preset: QuickPreset,
    search: &str,
    criteria: &FilterCriteria,
    shortlisted: &[String],
) -> Vec<ScreenerInstrument> {
    universe
        .iter()
        .filter(|inst| {
            // 1. Search query: symbol, name, or sector.
            if !search.is_empty() {
                let q = search.to_lowercase();
                let matches = inst.symbol.to_lowercase().contains(&q)
                    || inst.name.to_lowercase().contains(&q)
                    || inst.sector.to_lowercase().contains(&q);
                if !matches {
                    return false;
                }
            }

            // 2. Quick presets.
            match preset {
                QuickPreset::All => {}
                QuickPreset::Gainers => {
                    if inst.change_percent <= Decimal::ZERO {
                        return false;
                    }
                }
                QuickPreset::Losers => {
                    if inst.change_percent >= Decimal::ZERO {
                        return false;
                    }
                }
                QuickPreset::MostCapitalized => {
                    if inst.market_cap_cr < dec!(200_000) {
                        return false;
                    }
                }
                QuickPreset::Momentum => {
                    if inst.technical_rating != TechnicalRating::Buy
                        && inst.technical_rating != TechnicalRating::StrongBuy
                    {
                        return false;
                    }
                }
                QuickPreset::Value => match inst.pe_ratio {
                    Some(pe) if pe > Decimal::ZERO && pe < dec!(25) => {}
                    _ => return false,
                },
                QuickPreset::Dividend => match inst.dividend_yield {
                    Some(y) if y >= dec!(1.5) => {}
                    _ => return false,
                },
                QuickPreset::MostActive => {
                    if inst.volume < 2_000_000 {
                        return false;
                    }
                }
                QuickPreset::High52 => {
                    if inst.price < inst.high_52 * dec!(0.95) {
                        return false;
                    }
                }
                QuickPreset::Oversold => {
                    if inst.rsi_14 >= dec!(35) {
                        return false;
                    }
                }
                QuickPreset::Overbought => {
                    if inst.rsi_14 <= dec!(70) {
                        return false;
                    }
                }
                QuickPreset::Watchlist => {
                    if !shortlisted.contains(&inst.symbol) {
                        return false;
                    }
                }
            }

            // 3. Granular filter criteria.
            if !criteria.sector.is_empty()
                && criteria.sector != "All"
                && inst.sector != criteria.sector
            {
                return false;
            }
            if !criteria.market_cap_tier.contains(inst.market_cap_cr) {
                return false;
            }
            if criteria.min_price.is_some_and(|min_p| inst.price < min_p) {
                return false;
            }
            if criteria.max_price.is_some_and(|max_p| inst.price > max_p) {
                return false;
            }
            if criteria
                .min_change_pct
                .is_some_and(|min_c| inst.change_percent < min_c)
            {
                return false;
            }
            if criteria
                .max_change_pct
                .is_some_and(|max_c| inst.change_percent > max_c)
            {
                return false;
            }
            if criteria.min_volume.is_some_and(|min_v| inst.volume < min_v) {
                return false;
            }
            if let Some(min_pe) = criteria.min_pe {
                match inst.pe_ratio {
                    Some(pe) if pe >= min_pe => {}
                    _ => return false,
                }
            }
            if let Some(max_pe) = criteria.max_pe {
                match inst.pe_ratio {
                    Some(pe) if pe <= max_pe => {}
                    _ => return false,
                }
            }
            if let Some(max_pb) = criteria.max_pb {
                match inst.pb_ratio {
                    Some(pb) if pb <= max_pb => {}
                    _ => return false,
                }
            }
            if let Some(min_roe) = criteria.min_roe {
                match inst.roe {
                    Some(roe) if roe >= min_roe => {}
                    _ => return false,
                }
            }
            if let Some(min_fcf) = criteria.min_fcf_yield {
                match inst.fcf_yield_pct {
                    Some(fcf) if fcf >= min_fcf => {}
                    _ => return false,
                }
            }
            if let Some(min_d) = criteria.min_dividend {
                match inst.dividend_yield {
                    Some(y) if y >= min_d => {}
                    _ => return false,
                }
            }
            if criteria.min_rsi.is_some_and(|min_r| inst.rsi_14 < min_r) {
                return false;
            }
            if criteria.max_rsi.is_some_and(|max_r| inst.rsi_14 > max_r) {
                return false;
            }

            true
        })
        .cloned()
        .collect()
}

/// A sort value: one key per column, one comparison rule per kind.
#[derive(Debug, Clone, PartialEq)]
pub enum SortValue {
    Number(Decimal),
    Text(String),
}

impl SortValue {
    /// `Ord`-like comparison; `Text` compares case-insensitively.
    pub fn cmp_value(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (self, other) {
            (SortValue::Number(a), SortValue::Number(b)) => {
                if a < b {
                    Ordering::Less
                } else if a > b {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            }
            (SortValue::Text(a), SortValue::Text(b)) => {
                let (a, b) = (a.to_uppercase(), b.to_uppercase());
                a.cmp(&b)
            }
            // Numbers sort before texts (a column never mixes kinds).
            (SortValue::Number(_), SortValue::Text(_)) => Ordering::Less,
            (SortValue::Text(_), SortValue::Number(_)) => Ordering::Greater,
        }
    }
}

/// The sort key of one instrument for a column id (`types::tab_columns`).
/// `Option` metrics sort as a huge sentinel so missing data sinks an
/// otherwise-descending board instead of straddling the comparison.
pub fn sort_value(inst: &ScreenerInstrument, column_id: &str) -> SortValue {
    let absent = dec!(999_999_999);
    let number = |d: Decimal| SortValue::Number(d);
    match column_id {
        "symbol" => SortValue::Text(inst.symbol.clone()),
        "rating" => number(Decimal::from(inst.technical_rating.rank())),
        "volume" => number(Decimal::from(inst.volume)),
        "avg_volume_30d" => number(Decimal::from(inst.avg_volume_30d)),
        "price" => number(inst.price),
        "change" => number(inst.change),
        "change_percent" => number(inst.change_percent),
        "perf_1w" => number(inst.perf_1w),
        "perf_1m" => number(inst.perf_1m),
        "perf_3m" => number(inst.perf_3m),
        "perf_1y" => number(inst.perf_1y),
        "perf_ytd" => number(inst.perf_ytd),
        "market_cap_cr" => number(inst.market_cap_cr),
        "pe_ratio" => number(inst.pe_ratio.unwrap_or(absent)),
        "pb_ratio" => number(inst.pb_ratio.unwrap_or(absent)),
        "dividend_yield" => number(inst.dividend_yield.unwrap_or(absent)),
        "roe" => number(inst.roe.unwrap_or(absent)),
        "roce" => number(inst.roce.unwrap_or(absent)),
        "net_margin" => number(inst.net_margin.unwrap_or(absent)),
        // Statement metrics (design doc §5.5): same `Option` sentinel.
        "payout_ratio" => number(inst.payout_ratio.unwrap_or(absent)),
        "dividend_per_share" => number(inst.dividend_per_share.unwrap_or(absent)),
        "dps_growth_5y" => number(inst.dps_growth_5y.unwrap_or(absent)),
        "revenue_cr" => number(inst.revenue_cr.unwrap_or(absent)),
        "revenue_growth_yoy" => number(inst.revenue_growth_yoy.unwrap_or(absent)),
        "ebitda_cr" => number(inst.ebitda_cr.unwrap_or(absent)),
        "pat_cr" => number(inst.pat_cr.unwrap_or(absent)),
        "eps" => number(inst.eps.unwrap_or(absent)),
        "total_debt_cr" => number(inst.total_debt_cr.unwrap_or(absent)),
        "debt_to_equity" => number(inst.debt_to_equity.unwrap_or(absent)),
        "current_ratio" => number(inst.current_ratio.unwrap_or(absent)),
        "book_value_per_share" => number(inst.book_value_per_share.unwrap_or(absent)),
        "operating_cf_cr" => number(inst.operating_cf_cr.unwrap_or(absent)),
        "free_cf_cr" => number(inst.free_cf_cr.unwrap_or(absent)),
        "fcf_yield_pct" => number(inst.fcf_yield_pct.unwrap_or(absent)),
        "cash_per_share" => number(inst.cash_per_share.unwrap_or(absent)),
        "sales_per_share" => number(inst.sales_per_share.unwrap_or(absent)),
        "rsi_14" => number(inst.rsi_14),
        "sma_20" => number(inst.sma_20),
        "sma_50" => number(inst.sma_50),
        "sma_200" => number(inst.sma_200),
        "high_52" => number(inst.high_52),
        "low_52" => number(inst.low_52),
        "range_52w" => {
            let spread = inst.high_52 - inst.low_52;
            if spread <= Decimal::ZERO {
                number(Decimal::ZERO)
            } else {
                number((inst.price - inst.low_52) / spread)
            }
        }
        // Unknown/visual-only ids fall back to the symbol order.
        _ => SortValue::Text(inst.symbol.clone()),
    }
}

/// Sorts by the active state; stable (a `None`/missing id keeps the
/// universe's order). `Desc` is the first press (the gainers' board).
pub fn sort_instruments(
    mut rows: Vec<ScreenerInstrument>,
    sort: Option<&SortState>,
) -> Vec<ScreenerInstrument> {
    if let Some(state) = sort {
        let direction = state.direction;
        rows.sort_by(|a, b| {
            let key_a = sort_value(a, &state.column_id);
            let key_b = sort_value(b, &state.column_id);
            match direction {
                SortDirection::Desc => key_b.cmp_value(&key_a),
                SortDirection::Asc => key_a.cmp_value(&key_b),
            }
        });
    }
    rows
}

/// The columns of one tab that start visible (all of them; the column
/// modal hides some).
pub fn default_visible_columns(tab: ScreenerViewTab) -> Vec<String> {
    crate::features::screener::types::tab_columns(tab)
        .iter()
        .map(|c| c.id.to_string())
        .collect()
}

/// The available columns of a tab sliced per the visible ids.
pub fn visible_tab_columns(
    tab: ScreenerViewTab,
    visible: &[String],
) -> Vec<&'static ScreenerColumn> {
    crate::features::screener::types::tab_columns(tab)
        .iter()
        .filter(|c| visible.iter().any(|id| id == c.id))
        .collect()
}

fn csv_cell(cell: &str) -> String {
    if cell.contains(',') || cell.contains('"') || cell.contains('\n') {
        format!("\"{}\"", cell.replace('"', "\"\""))
    } else {
        cell.to_string()
    }
}

/// One stock row's CSV cells (the port mapping's per-row values); shared
/// by [`export_csv`] and [`export_records_csv`]'s stock section so the
/// two documents can never drift apart.
fn stock_csv_cells(inst: &ScreenerInstrument) -> Vec<String> {
    let opt_text = |d: Option<Decimal>| d.map(|v| v.round_dp(2).to_string()).unwrap_or_default();
    vec![
        inst.symbol.clone(),
        inst.name.clone(),
        inst.sector.clone(),
        inst.exchange.clone(),
        inst.price.round_dp(2).to_string(),
        inst.change.round_dp(2).to_string(),
        inst.change_percent.round_dp(2).to_string(),
        inst.volume.to_string(),
        inst.market_cap_cr.round_dp(0).to_string(),
        opt_text(inst.pe_ratio),
        opt_text(inst.pb_ratio),
        opt_text(inst.roe),
        opt_text(inst.roce),
        opt_text(inst.dividend_yield),
        inst.rsi_14.round_dp(2).to_string(),
        inst.technical_rating.label().to_string(),
    ]
}

/// Appends one CSV line (each cell quoted via [`csv_cell`]) to the
/// document.
fn push_csv_line(out: &mut String, cells: &[String]) {
    let line: Vec<String> = cells.iter().map(|c| csv_cell(c)).collect();
    out.push_str(&line.join(","));
    out.push('\n');
}

/// The active rows as one CSV document (design doc §3's port mapping;
/// the page logs and clipboard-copies the string).
pub fn export_csv(rows: &[ScreenerInstrument]) -> String {
    use crate::features::screener::types::CSV_COLUMNS;
    let mut out = String::new();
    out.push_str(&CSV_COLUMNS.join(","));
    out.push('\n');
    for inst in rows {
        push_csv_line(&mut out, &stock_csv_cells(inst));
    }
    out
}

/// A match counter text: `"Matches: 24 / 32"`.
pub fn matches_text(matches: usize, total: usize) -> String {
    format!("Matches: {matches} / {total}")
}

/// The deterministic screener universe, surfaced through the feature's
/// public path (`pub(crate)` fixtures keep inputs internal; host-side
/// integration tests build rows through this).
pub fn screener_universe() -> Vec<ScreenerInstrument> {
    crate::features::fixtures::screener_universe()
}

// ---------------------------------------------------------------------------
// 1b. Multi-asset record layer (stocks + ETFs + bonds + mutual funds)
// ---------------------------------------------------------------------------

/// The deterministic multi-asset board: the stock universe first, then
/// the ETF, bond, and mutual-fund fixtures, each wrapped in its class
/// variant (host tests build rows through this, mirroring
/// [`screener_universe`]).
pub fn screener_records() -> Vec<ScreenerRecord> {
    let mut out: Vec<ScreenerRecord> = screener_universe()
        .into_iter()
        .map(ScreenerRecord::Stock)
        .collect();
    out.extend(
        crate::features::fixtures::screener_etfs()
            .into_iter()
            .map(ScreenerRecord::Etf),
    );
    out.extend(
        crate::features::fixtures::screener_bonds()
            .into_iter()
            .map(ScreenerRecord::Bond),
    );
    out.extend(
        crate::features::fixtures::screener_mfs()
            .into_iter()
            .map(ScreenerRecord::MutualFund),
    );
    out
}

/// Applies the asset-class slice, the search text, the quick preset, and
/// the granular criteria to the mixed record board (design doc §6).
///
/// Stock rows delegate wholesale to [`filter_instruments`] (search on
/// symbol/name/sector, every preset, every criterion). The other classes
/// keep the shared `Gainers`/`Losers`/`Watchlist`/`All` presets and only
/// the shared numeric bounds (`min_price`/`max_price`,
/// `min_change_pct`/`max_change_pct`); every other preset is a
/// stock-only heat screen (capitalization, momentum, value, 52-week
/// high, RSI bands, …) and drops non-stock rows outright.
pub fn filter_records(
    records: &[ScreenerRecord],
    asset: ScreenerAssetType,
    preset: QuickPreset,
    search: &str,
    criteria: &FilterCriteria,
    shortlisted: &[String],
) -> Vec<ScreenerRecord> {
    // 1. The asset-class slice.
    let subset: Vec<ScreenerRecord> = records
        .iter()
        .filter(|r| r.class() == asset)
        .cloned()
        .collect();

    // 2. Stock rows: one path for search + presets + criteria.
    if asset == ScreenerAssetType::Stocks {
        let stocks: Vec<ScreenerInstrument> = subset
            .iter()
            .filter_map(|r| match r {
                ScreenerRecord::Stock(s) => Some(s.clone()),
                _ => None,
            })
            .collect();
        return filter_instruments(&stocks, preset, search, criteria, shortlisted)
            .into_iter()
            .map(ScreenerRecord::Stock)
            .collect();
    }

    // 3. Non-stock rows: shared search/presets/bounds only.
    subset
        .into_iter()
        .filter(|r| {
            // Search: symbol + name plus the class's own facets (ETF
            // brand and focus, fund AMC and category); bonds match on
            // name only.
            if !search.is_empty() {
                let q = search.to_lowercase();
                let hit = r.symbol().to_lowercase().contains(&q)
                    || r.name().to_lowercase().contains(&q)
                    || match r {
                        ScreenerRecord::Etf(e) => {
                            e.brand.to_lowercase().contains(&q)
                                || e.focus.to_lowercase().contains(&q)
                        }
                        ScreenerRecord::MutualFund(m) => {
                            m.amc.to_lowercase().contains(&q)
                                || m.category.to_lowercase().contains(&q)
                        }
                        // Bonds: name only (already checked above);
                        // stock rows never reach this branch.
                        _ => false,
                    };
                if !hit {
                    return false;
                }
            }

            // Presets: the shared four apply to every class; every other
            // preset is a stock-only heat screen (see the doc comment).
            match preset {
                QuickPreset::All => {}
                QuickPreset::Gainers => {
                    if r.change_percent() <= Decimal::ZERO {
                        return false;
                    }
                }
                QuickPreset::Losers => {
                    if r.change_percent() >= Decimal::ZERO {
                        return false;
                    }
                }
                QuickPreset::Watchlist => {
                    if !shortlisted.iter().any(|s| s == r.symbol()) {
                        return false;
                    }
                }
                _ => return false,
            }

            // Shared numeric bounds only — the stock-only criteria
            // (sector, tier, volume, PE, dividend, RSI) do not apply.
            if criteria.min_price.is_some_and(|min_p| r.price() < min_p) {
                return false;
            }
            if criteria.max_price.is_some_and(|max_p| r.price() > max_p) {
                return false;
            }
            if criteria
                .min_change_pct
                .is_some_and(|min_c| r.change_percent() < min_c)
            {
                return false;
            }
            if criteria
                .max_change_pct
                .is_some_and(|max_c| r.change_percent() > max_c)
            {
                return false;
            }

            true
        })
        .collect()
}

/// The sort key of one record for a column id. Stock rows delegate to
/// [`sort_value`]; every other class reads its own metrics and falls
/// back to the huge sentinel for ids that belong to another class (the
/// same convention as [`sort_value`]).
pub fn sort_value_record(record: &ScreenerRecord, column_id: &str) -> SortValue {
    if let ScreenerRecord::Stock(inst) = record {
        return sort_value(inst, column_id);
    }
    let absent = dec!(999_999_999);
    let number = |d: Decimal| SortValue::Number(d);
    match column_id {
        // Shared ids for any class.
        "symbol" => SortValue::Text(record.symbol().to_string()),
        "price" | "nav" | "clean_price" => number(record.price()),
        "change_percent" => number(record.change_percent()),
        // ETF (and fund AUM/expense) metrics.
        "aum_cr" => match record {
            ScreenerRecord::Etf(r) => number(r.aum_cr),
            ScreenerRecord::MutualFund(r) => number(r.aum_cr),
            _ => number(absent),
        },
        "expense_ratio" => match record {
            ScreenerRecord::Etf(r) => number(r.expense_ratio),
            ScreenerRecord::MutualFund(r) => number(r.expense_ratio),
            _ => number(absent),
        },
        "tracking_error" => match record {
            ScreenerRecord::Etf(r) => number(r.tracking_error),
            _ => number(absent),
        },
        "perf_1y" => match record {
            ScreenerRecord::Etf(r) => number(r.perf_1y),
            _ => number(absent),
        },
        "perf_3y" => match record {
            ScreenerRecord::Etf(r) => number(r.perf_3y),
            _ => number(absent),
        },
        "volume" => match record {
            ScreenerRecord::Etf(r) => number(Decimal::from(r.volume)),
            _ => number(absent),
        },
        // Bond metrics.
        "coupon_rate" => match record {
            ScreenerRecord::Bond(r) => number(r.coupon_rate),
            _ => number(absent),
        },
        "ytw" => match record {
            ScreenerRecord::Bond(r) => number(r.ytw),
            _ => number(absent),
        },
        "duration_years" => match record {
            ScreenerRecord::Bond(r) => number(r.duration_years),
            _ => number(absent),
        },
        "maturity_date" => match record {
            ScreenerRecord::Bond(r) => SortValue::Text(r.maturity_date.clone()),
            _ => number(absent),
        },
        // Mutual-fund metrics.
        "sharpe_ratio" => match record {
            ScreenerRecord::MutualFund(r) => number(r.sharpe_ratio),
            _ => number(absent),
        },
        "cagr_1y" | "cagr_3y" | "cagr_5y" => match record {
            ScreenerRecord::MutualFund(r) => number(match column_id {
                "cagr_1y" => r.cagr_1y,
                "cagr_3y" => r.cagr_3y,
                _ => r.cagr_5y,
            }),
            _ => number(absent),
        },
        "rating_stars" => match record {
            ScreenerRecord::MutualFund(r) => number(Decimal::from(r.rating_stars)),
            _ => number(absent),
        },
        // Unknown ids on a non-stock record sink to the sentinel.
        _ => number(absent),
    }
}

/// Sorts records by the active state; stable (a `None`/missing id keeps
/// the board's order). Mirrors [`sort_instruments`] with
/// [`sort_value_record`] keys. `Desc` is the first press.
pub fn sort_records(
    mut rows: Vec<ScreenerRecord>,
    sort: Option<&SortState>,
) -> Vec<ScreenerRecord> {
    if let Some(state) = sort {
        let direction = state.direction;
        rows.sort_by(|a, b| {
            let key_a = sort_value_record(a, &state.column_id);
            let key_b = sort_value_record(b, &state.column_id);
            match direction {
                SortDirection::Desc => key_b.cmp_value(&key_a),
                SortDirection::Asc => key_a.cmp_value(&key_b),
            }
        });
    }
    rows
}

/// The column set of one asset class: stocks reuse the Overview tab's
/// columns; every other class carries its own fixed set (built once,
/// shared as `&'static`).
pub fn asset_columns(asset: ScreenerAssetType) -> &'static [ScreenerColumn] {
    match asset {
        ScreenerAssetType::Stocks => {
            crate::features::screener::types::tab_columns(ScreenerViewTab::Overview)
        }
        ScreenerAssetType::Etf => etf_columns(),
        ScreenerAssetType::Bonds => bond_columns(),
        ScreenerAssetType::MutualFunds => mf_columns(),
    }
}

/// The ETF column set (label text is the page contract).
fn etf_columns() -> &'static [ScreenerColumn] {
    static COLS: OnceLock<Vec<ScreenerColumn>> = OnceLock::new();
    COLS.get_or_init(|| {
        vec![
            ScreenerColumn::new("symbol", "Symbol"),
            ScreenerColumn::new("price", "Nav"),
            ScreenerColumn::new("change_percent", "Chg %"),
            ScreenerColumn::new("aum_cr", "AUM"),
            ScreenerColumn::new("expense_ratio", "Expense %"),
            ScreenerColumn::new("tracking_error", "Tracking %"),
            ScreenerColumn::new("perf_1y", "1Y %"),
            ScreenerColumn::new("perf_3y", "3Y %"),
            ScreenerColumn::new("volume", "Volume"),
        ]
    })
}

/// The bond column set (label text is the page contract).
fn bond_columns() -> &'static [ScreenerColumn] {
    static COLS: OnceLock<Vec<ScreenerColumn>> = OnceLock::new();
    COLS.get_or_init(|| {
        vec![
            ScreenerColumn::new("symbol", "Symbol"),
            ScreenerColumn::new("price", "Clean Price"),
            ScreenerColumn::new("change_percent", "Chg %"),
            ScreenerColumn::new("coupon_rate", "Coupon %"),
            ScreenerColumn::new("ytw", "YTW %"),
            ScreenerColumn::new("maturity_date", "Maturity"),
            ScreenerColumn::new("duration_years", "Duration"),
            ScreenerColumn::new("credit_rating", "Rating"),
        ]
    })
}

/// The mutual-fund column set (label text is the page contract).
fn mf_columns() -> &'static [ScreenerColumn] {
    static COLS: OnceLock<Vec<ScreenerColumn>> = OnceLock::new();
    COLS.get_or_init(|| {
        vec![
            ScreenerColumn::new("symbol", "Symbol"),
            ScreenerColumn::new("price", "NAV"),
            ScreenerColumn::new("change_percent", "Chg %"),
            ScreenerColumn::new("aum_cr", "AUM"),
            ScreenerColumn::new("expense_ratio", "Expense %"),
            ScreenerColumn::new("sharpe_ratio", "Sharpe"),
            ScreenerColumn::new("cagr_1y", "1Y %"),
            ScreenerColumn::new("cagr_3y", "3Y %"),
            ScreenerColumn::new("cagr_5y", "5Y %"),
            ScreenerColumn::new("rating_stars", "Stars"),
        ]
    })
}

/// The columns that start visible for one asset class: the tab's own
/// defaults for stocks, the class's full column set otherwise (the
/// column modal hides some).
pub fn default_visible_columns_for(asset: ScreenerAssetType, tab: ScreenerViewTab) -> Vec<String> {
    match asset {
        ScreenerAssetType::Stocks => default_visible_columns(tab),
        other => asset_columns(other)
            .iter()
            .map(|c| c.id.to_string())
            .collect(),
    }
}

/// The available columns of an asset class sliced per the visible ids.
fn visible_asset_columns(
    asset: ScreenerAssetType,
    visible: &[String],
) -> Vec<&'static ScreenerColumn> {
    asset_columns(asset)
        .iter()
        .filter(|c| visible.iter().any(|id| id == c.id))
        .collect()
}

/// The active records as one CSV document, sectioned per asset class
/// (stocks → ETFs → bonds → mutual funds): one header line per class
/// that has rows, then that class's rows (the page logs and
/// clipboard-copies the string).
pub fn export_records_csv(records: &[ScreenerRecord]) -> String {
    use crate::features::screener::types::CSV_COLUMNS;
    let mut out = String::new();

    // Stocks: the exact `export_csv` cells.
    let stocks: Vec<&ScreenerInstrument> = records
        .iter()
        .filter_map(|r| match r {
            ScreenerRecord::Stock(s) => Some(s),
            _ => None,
        })
        .collect();
    if !stocks.is_empty() {
        out.push_str(&CSV_COLUMNS.join(","));
        out.push('\n');
        for inst in stocks {
            push_csv_line(&mut out, &stock_csv_cells(inst));
        }
    }

    // ETFs.
    let etfs: Vec<&ScreenerEtfRecord> = records
        .iter()
        .filter_map(|r| match r {
            ScreenerRecord::Etf(e) => Some(e),
            _ => None,
        })
        .collect();
    if !etfs.is_empty() {
        out.push_str(
            "SYMBOL,NAME,BRAND,FOCUS,NAV,CHANGE,CHANGE %,AUM (CR),EXPENSE %,TRACKING %,VOLUME,1Y %,3Y %\n",
        );
        for r in etfs {
            push_csv_line(
                &mut out,
                &[
                    r.symbol.clone(),
                    r.name.clone(),
                    r.brand.clone(),
                    r.focus.clone(),
                    r.nav.round_dp(2).to_string(),
                    r.change.round_dp(2).to_string(),
                    r.change_percent.round_dp(2).to_string(),
                    r.aum_cr.round_dp(0).to_string(),
                    r.expense_ratio.round_dp(2).to_string(),
                    r.tracking_error.round_dp(2).to_string(),
                    r.volume.to_string(),
                    r.perf_1y.round_dp(2).to_string(),
                    r.perf_3y.round_dp(2).to_string(),
                ],
            );
        }
    }

    // Bonds.
    let bonds: Vec<&ScreenerBondRecord> = records
        .iter()
        .filter_map(|r| match r {
            ScreenerRecord::Bond(b) => Some(b),
            _ => None,
        })
        .collect();
    if !bonds.is_empty() {
        out.push_str(
            "SYMBOL,NAME,ISSUER,RATING,COUPON %,YTW %,CLEAN PRICE,FACE VALUE,MATURITY,DURATION (YRS),CHANGE %\n",
        );
        for r in bonds {
            push_csv_line(
                &mut out,
                &[
                    r.symbol.clone(),
                    r.name.clone(),
                    r.issuer_type.label().to_string(),
                    r.credit_rating.label().to_string(),
                    r.coupon_rate.round_dp(2).to_string(),
                    r.ytw.round_dp(2).to_string(),
                    r.clean_price.round_dp(2).to_string(),
                    r.face_value.round_dp(2).to_string(),
                    r.maturity_date.clone(),
                    r.duration_years.round_dp(2).to_string(),
                    r.change_percent.round_dp(2).to_string(),
                ],
            );
        }
    }

    // Mutual funds.
    let mfs: Vec<&ScreenerMfRecord> = records
        .iter()
        .filter_map(|r| match r {
            ScreenerRecord::MutualFund(m) => Some(m),
            _ => None,
        })
        .collect();
    if !mfs.is_empty() {
        out.push_str(
            "SYMBOL,NAME,AMC,CATEGORY,SCHEME,NAV,CHANGE,CHANGE %,AUM (CR),EXPENSE %,SHARPE,1Y %,3Y %,5Y %,STARS\n",
        );
        for r in mfs {
            push_csv_line(
                &mut out,
                &[
                    r.symbol.clone(),
                    r.name.clone(),
                    r.amc.clone(),
                    r.category.clone(),
                    r.scheme_type.label().to_string(),
                    r.nav.round_dp(2).to_string(),
                    r.change.round_dp(2).to_string(),
                    r.change_percent.round_dp(2).to_string(),
                    r.aum_cr.round_dp(0).to_string(),
                    r.expense_ratio.round_dp(2).to_string(),
                    r.sharpe_ratio.round_dp(2).to_string(),
                    r.cagr_1y.round_dp(2).to_string(),
                    r.cagr_3y.round_dp(2).to_string(),
                    r.cagr_5y.round_dp(2).to_string(),
                    r.rating_stars.to_string(),
                ],
            );
        }
    }

    out
}

// ---------------------------------------------------------------------------
// 2. Event / props contract + component
// ---------------------------------------------------------------------------

use honba_kao_ui_core::primitive::badge::{BadgeVariant, HfBadge, HfBadgeProps};
use honba_kao_ui_core::primitive::sparkline::{HfSparklinePrimitive, HfSparklinePrimitiveProps};
use honba_kao_ui_core::theme::tokens::{HonbaStyle, MONO, colors, radius};
use nsh_core::prelude::*;

use crate::features::screener::types;

/// Maps a [`TechnicalRating`] to its badge variant.
fn rating_variant(rating: TechnicalRating) -> Option<BadgeVariant> {
    match rating {
        TechnicalRating::StrongBuy | TechnicalRating::Buy => Some(BadgeVariant::Success),
        TechnicalRating::Neutral => Some(BadgeVariant::Warning),
        TechnicalRating::Sell | TechnicalRating::StrongSell => Some(BadgeVariant::Danger),
    }
}

/// Fixed per-column cell widths (px).
fn cell_width(col_id: &str) -> f32 {
    match col_id {
        "symbol" => 240.0,
        "price" => 110.0,
        "change_percent" => 92.0,
        "perf_1w" | "perf_1m" | "perf_3m" | "perf_1y" | "perf_ytd" => 92.0,
        "volume" => 100.0,
        "market_cap_cr" => 124.0,
        "pe_ratio" | "pb_ratio" => 72.0,
        "dividend_yield" => 108.0,
        "roe" | "roce" | "net_margin" => 112.0,
        // Statement metrics (design doc §5.5).
        "payout_ratio" => 108.0,
        "dividend_per_share" => 110.0,
        "dps_growth_5y" => 128.0,
        "revenue_cr" | "ebitda_cr" | "operating_cf_cr" => 124.0,
        "revenue_growth_yoy" => 122.0,
        "pat_cr" | "free_cf_cr" | "total_debt_cr" => 112.0,
        "eps" => 96.0,
        "debt_to_equity" => 112.0,
        "current_ratio" => 116.0,
        "book_value_per_share" | "sales_per_share" => 132.0,
        "fcf_yield_pct" => 116.0,
        "cash_per_share" => 128.0,
        "rsi_14" => 92.0,
        "sma_20" | "sma_50" | "sma_200" => 104.0,
        "rating" => 116.0,
        "range_52w" => 132.0,
        "trend" => 112.0,
        // Multi-asset record columns.
        "nav" => 110.0,
        "aum_cr" => 124.0,
        "expense_ratio" => 108.0,
        "tracking_error" => 110.0,
        "coupon_rate" => 104.0,
        "ytw" => 88.0,
        "maturity_date" => 132.0,
        "duration_years" => 104.0,
        "credit_rating" => 96.0,
        "sharpe_ratio" => 96.0,
        "cagr_1y" | "cagr_3y" | "cagr_5y" => 92.0,
        "rating_stars" => 108.0,
        _ => 104.0,
    }
}

/// `Option<Decimal>` → "12.34" or "—".
fn opt_metric_text(value: Option<Decimal>) -> String {
    value.map_or_else(|| "—".to_string(), |v| v.round_dp(2).to_string())
}

/// `Option<Decimal>` → "12.34%" or "—".
fn opt_percent_text(value: Option<Decimal>) -> String {
    value.map_or_else(|| "—".to_string(), |v| format!("{}%", v.round_dp(2)))
}

/// The 52-week range cell: a slim track with an accent fill positioned at the
/// instrument's own position inside its 52-week range.
fn range_bar_cell(inst: &ScreenerInstrument) -> AnyElement {
    view()
        .w(px(120.0))
        .h(px(5.0))
        .rounded_full()
        .bg(colors::RAISED)
        .overflow_hidden()
        .child(
            view()
                .h_full()
                .w(px((120.0 * inst.range_position()).max(2.0)))
                .bg(colors::ACCENT),
        )
        .into_element()
}

/// The fixed-width frame every data cell sits in (no shrink, the
/// column's own width).
fn wrap_cell(content: AnyElement, col_id: &str) -> AnyElement {
    view()
        .flex_shrink_0()
        .flex_row()
        .items_center()
        .h_full()
        .w(px(cell_width(col_id)))
        .child(content)
        .into_element()
}

/// A ▲/▼ delta percent cell (the perf lanes and every class's change /
/// CAGR cells share this shape).
fn delta_cell(value: Decimal) -> AnyElement {
    let up = value > Decimal::ZERO;
    let tone = if up { colors::UP } else { colors::DOWN };
    view()
        .flex_row()
        .gap(px(2.0))
        .child(text(if up { "▲" } else { "▼" }).text_color(tone))
        .child(text(types::percent_text(value)).text_color(tone))
        .into_element()
}

/// One data cell in a row; dispatches on the column id.
fn cell(inst: &ScreenerInstrument, col_id: &str) -> AnyElement {
    let content: AnyElement = match col_id {
        "symbol" => view()
            .flex_col()
            .gap(px(2.0))
            .child(
                text(inst.symbol.clone())
                    .text_size(px(12.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_high(),
            )
            .child(
                view()
                    .rounded(px(6.0))
                    .bg(colors::INSET)
                    .px(px(6.0))
                    .py(px(1.0))
                    .child(text(inst.sector.clone()).text_size(px(9.0)).text_muted()),
            )
            .into_element(),
        "price" => text(types::inr(inst.price))
            .text_high()
            .text_size(px(12.0))
            .font_family(MONO)
            .into_element(),
        "change_percent" => {
            let up = inst.change_up();
            let tone = if up { colors::UP } else { colors::DOWN };
            view()
                .flex_row()
                .gap(px(2.0))
                .child(text(if up { "▲" } else { "▼" }).text_color(tone))
                .child(text(types::percent_text(inst.change_percent)).text_color(tone))
                .into_element()
        }
        "perf_1w" | "perf_1m" | "perf_3m" | "perf_1y" | "perf_ytd" => {
            let value = match col_id {
                "perf_1w" => inst.perf_1w,
                "perf_1m" => inst.perf_1m,
                "perf_3m" => inst.perf_3m,
                "perf_1y" => inst.perf_1y,
                _ => inst.perf_ytd,
            };
            let up = value > Decimal::ZERO;
            let tone = if up { colors::UP } else { colors::DOWN };
            view()
                .flex_row()
                .gap(px(2.0))
                .child(text(if up { "▲" } else { "▼" }).text_color(tone))
                .child(text(types::percent_text(value)).text_color(tone))
                .into_element()
        }
        "volume" => text(types::compact_volume(inst.volume))
            .text_muted()
            .text_size(px(11.0))
            .into_element(),
        "market_cap_cr" => text(types::compact_cap_crore(inst.market_cap_cr))
            .text_muted()
            .into_element(),
        "pe_ratio" | "pb_ratio" => {
            let value = if col_id == "pe_ratio" {
                inst.pe_ratio
            } else {
                inst.pb_ratio
            };
            text(opt_metric_text(value)).text_high().into_element()
        }
        "dividend_yield" => text(opt_metric_text(inst.dividend_yield))
            .text_muted()
            .into_element(),
        "roe" | "roce" | "net_margin" => {
            let value = match col_id {
                "roe" => inst.roe,
                "roce" => inst.roce,
                _ => inst.net_margin,
            };
            text(opt_percent_text(value)).text_muted().into_element()
        }
        // Statement metrics (design doc §5.5): percentages, ratios, and
        // ₹-crore aggregates render muted; em dash where the statement
        // line is absent.
        "payout_ratio" => text(opt_percent_text(inst.payout_ratio))
            .text_muted()
            .into_element(),
        "dividend_per_share" => text(opt_metric_text(inst.dividend_per_share))
            .text_muted()
            .font_family(MONO)
            .into_element(),
        "dps_growth_5y" => text(opt_percent_text(inst.dps_growth_5y))
            .text_muted()
            .into_element(),
        "revenue_cr" | "ebitda_cr" | "pat_cr" | "total_debt_cr" | "operating_cf_cr"
        | "free_cf_cr" => {
            let value = match col_id {
                "revenue_cr" => inst.revenue_cr,
                "ebitda_cr" => inst.ebitda_cr,
                "pat_cr" => inst.pat_cr,
                "total_debt_cr" => inst.total_debt_cr,
                "operating_cf_cr" => inst.operating_cf_cr,
                _ => inst.free_cf_cr,
            };
            text(value.map_or_else(|| "—".to_string(), types::compact_cr))
                .text_muted()
                .into_element()
        }
        "revenue_growth_yoy" => text(opt_percent_text(inst.revenue_growth_yoy))
            .text_muted()
            .into_element(),
        "fcf_yield_pct" => text(opt_percent_text(inst.fcf_yield_pct))
            .text_muted()
            .into_element(),
        "eps" => text(opt_metric_text(inst.eps)).text_high().into_element(),
        "book_value_per_share" | "cash_per_share" | "sales_per_share" => {
            let value = match col_id {
                "book_value_per_share" => inst.book_value_per_share,
                "cash_per_share" => inst.cash_per_share,
                _ => inst.sales_per_share,
            };
            text(opt_metric_text(value))
                .text_muted()
                .font_family(MONO)
                .into_element()
        }
        "debt_to_equity" | "current_ratio" => {
            let value = if col_id == "debt_to_equity" {
                inst.debt_to_equity
            } else {
                inst.current_ratio
            };
            text(opt_metric_text(value)).text_muted().into_element()
        }
        "rsi_14" => {
            let rsi = inst.rsi_14;
            let tone = if rsi > dec!(70) {
                colors::DANGER
            } else if rsi < dec!(35) {
                colors::UP
            } else {
                colors::MUTED
            };
            text(rsi.round_dp(1).to_string())
                .text_color(tone)
                .into_element()
        }
        "sma_20" | "sma_50" | "sma_200" => {
            let line = match col_id {
                "sma_20" => inst.sma_20,
                "sma_50" => inst.sma_50,
                _ => inst.sma_200,
            };
            let tone = if inst.price > line {
                colors::UP
            } else {
                colors::DOWN
            };
            text(line.round_dp(1).to_string())
                .text_color(tone)
                .into_element()
        }
        "range_52w" => range_bar_cell(inst),
        "trend" => view()
            .w(px(112.0))
            .h(px(22.0))
            .items_center()
            .child(HfSparklinePrimitive::element(HfSparklinePrimitiveProps {
                values: Prop::from(inst.sparkline_closes.clone()),
                height: Prop::from(Some(22.0_f32)),
                color: Prop::from(if inst.change_up() {
                    None::<SharedString>
                } else {
                    Some("text-rose-500".into())
                }),
            }))
            .into_element(),
        "rating" => HfBadge::element(HfBadgeProps {
            variant: Prop::from(rating_variant(inst.technical_rating)),
            outline: Prop::from(false),
            children: Children::default(),
        })
        .child(
            text(inst.technical_rating.label().to_string())
                .text_size(px(9.0))
                .font_weight(FontWeight::SEMIBOLD),
        )
        .into_element(),
        _ => text("—").text_high().into_element(),
    };

    wrap_cell(content, col_id)
}

/// The cells shared by every non-stock class: the symbol chip (brand /
/// issuer / AMC in place of the stock sector chip), the price cell
/// (label is the column's job — Nav, Clean Price, NAV), and the change
/// percent lane. `None` hands the id back to the class dispatcher.
fn shared_cell(record: &ScreenerRecord, col_id: &str) -> Option<AnyElement> {
    match col_id {
        "symbol" => {
            let chip = match record {
                ScreenerRecord::Etf(r) => r.brand.clone(),
                ScreenerRecord::Bond(r) => r.issuer_type.label().to_string(),
                ScreenerRecord::MutualFund(r) => r.amc.clone(),
                ScreenerRecord::Stock(_) => String::new(),
            };
            Some(
                view()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        text(record.symbol().to_string())
                            .text_size(px(12.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_high(),
                    )
                    .child(
                        view()
                            .rounded(px(6.0))
                            .bg(colors::INSET)
                            .px(px(6.0))
                            .py(px(1.0))
                            .child(text(chip).text_size(px(9.0)).text_muted()),
                    )
                    .into_element(),
            )
        }
        "price" => Some(
            text(types::inr(record.price()))
                .text_high()
                .text_size(px(12.0))
                .font_family(MONO)
                .into_element(),
        ),
        "change_percent" => Some(delta_cell(record.change_percent())),
        _ => None,
    }
}

/// The ETF metric cells (AUM, expense, tracking error, perf, volume).
fn etf_cell(r: &ScreenerEtfRecord, col_id: &str) -> AnyElement {
    match col_id {
        "aum_cr" => text(types::compact_cr(r.aum_cr))
            .text_muted()
            .into_element(),
        "expense_ratio" | "tracking_error" => {
            let value = if col_id == "expense_ratio" {
                r.expense_ratio
            } else {
                r.tracking_error
            };
            text(format!("{}%", value.round_dp(2)))
                .text_muted()
                .into_element()
        }
        "perf_1y" | "perf_3y" => delta_cell(if col_id == "perf_1y" {
            r.perf_1y
        } else {
            r.perf_3y
        }),
        "volume" => text(types::compact_volume(r.volume))
            .text_muted()
            .text_size(px(11.0))
            .into_element(),
        _ => text("—").text_high().into_element(),
    }
}

/// The bond metric cells (coupon, YTW, maturity, duration, credit
/// rating badge).
fn bond_cell(r: &ScreenerBondRecord, col_id: &str) -> AnyElement {
    match col_id {
        "coupon_rate" => text(format!("{}%", r.coupon_rate.round_dp(2)))
            .text_high()
            .font_family(MONO)
            .into_element(),
        "ytw" => text(format!("{}%", r.ytw.round_dp(2)))
            .text_muted()
            .font_family(MONO)
            .into_element(),
        "maturity_date" => text(r.maturity_date.clone())
            .text_muted()
            .text_size(px(11.0))
            .font_family(MONO)
            .into_element(),
        "duration_years" => text(format!("{}y", r.duration_years.round_dp(1)))
            .text_muted()
            .into_element(),
        "credit_rating" => {
            let label = r.credit_rating.label().to_string();
            let variant = match label.as_str() {
                "SOV" | "AAA" => BadgeVariant::Success,
                "AA+" | "AA" => BadgeVariant::Warning,
                _ => BadgeVariant::Danger,
            };
            HfBadge::element(HfBadgeProps {
                variant: Prop::from(Some(variant)),
                outline: Prop::from(false),
                children: Children::default(),
            })
            .child(
                text(label)
                    .text_size(px(9.0))
                    .font_weight(FontWeight::SEMIBOLD),
            )
            .into_element()
        }
        _ => text("—").text_high().into_element(),
    }
}

/// The mutual-fund metric cells (AUM, expense, Sharpe, CAGRs, stars).
fn mf_cell(r: &ScreenerMfRecord, col_id: &str) -> AnyElement {
    match col_id {
        "aum_cr" => text(types::compact_cr(r.aum_cr))
            .text_muted()
            .into_element(),
        "expense_ratio" => text(format!("{}%", r.expense_ratio.round_dp(2)))
            .text_muted()
            .into_element(),
        "sharpe_ratio" => text(r.sharpe_ratio.round_dp(2).to_string())
            .text_high()
            .font_family(MONO)
            .into_element(),
        "cagr_1y" | "cagr_3y" | "cagr_5y" => delta_cell(match col_id {
            "cagr_1y" => r.cagr_1y,
            "cagr_3y" => r.cagr_3y,
            _ => r.cagr_5y,
        }),
        "rating_stars" => {
            // ★ filled + ☆ hollow, capped at five glyphs.
            let n = usize::from(r.rating_stars.min(5));
            text(format!("{}{}", "★".repeat(n), "☆".repeat(5 - n)))
                .text_color(colors::ACCENT)
                .into_element()
        }
        _ => text("—").text_high().into_element(),
    }
}

/// One data cell of any asset-class record; dispatches on the record's
/// class first. Stock rows keep the exact stock cell vocabulary
/// (sparkline, rating badge, range bar, …) behind the same [`cell`]
/// dispatcher; the other classes share the symbol/price/change cells
/// and then read their own metrics.
fn cell_record(record: &ScreenerRecord, col_id: &str) -> AnyElement {
    if let ScreenerRecord::Stock(inst) = record {
        return cell(inst, col_id);
    }
    let content = match shared_cell(record, col_id) {
        Some(shared) => shared,
        None => match record {
            ScreenerRecord::Etf(r) => etf_cell(r, col_id),
            ScreenerRecord::Bond(r) => bond_cell(r, col_id),
            ScreenerRecord::MutualFund(r) => mf_cell(r, col_id),
            ScreenerRecord::Stock(_) => text("—").text_high().into_element(),
        },
    };
    wrap_cell(content, col_id)
}

/// One row: shortlist star, then the visible data cells. Whole-row press
/// activates the row (active symbol) and opens the detail drawer; the star
/// press toggles the row's shortlist membership independently. The column
/// set follows the asset class (stocks: the active tab; others: the
/// class's fixed set).
#[allow(clippy::too_many_arguments)]
fn grid_row_record(
    record: &ScreenerRecord,
    asset: ScreenerAssetType,
    tab: ScreenerViewTab,
    visible: &[String],
    active: bool,
    shortlisted: bool,
    active_signal: Signal<String>,
    drawer_signal: Signal<bool>,
    shortlist_signal: Signal<Vec<String>>,
) -> AnyElement {
    let columns: Vec<&'static ScreenerColumn> = if asset == ScreenerAssetType::Stocks {
        visible_tab_columns(tab, visible)
    } else {
        visible_asset_columns(asset, visible)
    };
    let row_symbol = record.symbol().to_string();
    let sym_to_toggle = row_symbol.clone();
    let row = if active {
        view()
            .flex_row()
            .items_center()
            .h(px(34.0))
            .rounded(px(radius::CONTROL))
            .flex_shrink_0()
            .bg(colors::RAISED)
            .border_1()
            .border_color(colors::ACCENT)
    } else {
        view()
            .flex_row()
            .items_center()
            .h(px(34.0))
            .rounded(px(radius::CONTROL))
            .flex_shrink_0()
            .bg(colors::SURFACE)
    };
    row.child(
        view()
            .flex_shrink_0()
            .w(px(44.0))
            .items_center()
            .justify_center()
            .child(
                text(if shortlisted { "★" } else { "☆" }).text_color(if shortlisted {
                    colors::ACCENT
                } else {
                    colors::MUTED
                }),
            )
            .on_press(move || {
                shortlist_signal.update(|list| {
                    let sym = sym_to_toggle.clone();
                    if let Some(at) = list.iter().position(|s| *s == sym) {
                        list.remove(at);
                    } else {
                        list.push(sym);
                    }
                });
            }),
    )
    .children(columns.iter().map(|col| cell_record(record, col.id)))
    .on_press(move || {
        active_signal.set(row_symbol.clone());
        drawer_signal.set(true);
    })
    .into_element()
}

/// A pressable header cell: muted label plus an optional blue "Filter" chip
/// when this column's filters are active, and an accent sort caret when this
/// column is the active sort key (display-only now). Pressing the cell
/// toggles this column's header menu (filters, sort, visibility) — the sort
/// cycle itself lives inside the menu.
fn header_cell(
    col: &ScreenerColumn,
    sort: Option<&SortState>,
    _sort_signal: Signal<Option<SortState>>,
    menu_signal: Signal<Option<String>>,
    criteria: &FilterCriteria,
) -> AnyElement {
    let col_id = col.id.to_string();
    let menu_id = col_id;
    let caret = sort.filter(|s| s.column_id == col.id).map(|s| {
        if s.direction == SortDirection::Desc {
            "▼"
        } else {
            "▲"
        }
    });
    view()
        .flex_shrink_0()
        .w(px(cell_width(col.id)))
        .flex_row()
        .gap(px(2.0))
        .items_center()
        .child(
            text(col.label)
                .text_size(px(10.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_muted(),
        )
        .children(if column_filters_active(col.id, criteria) {
            Some(
                view()
                    .px(px(4.0))
                    .py(px(1.0))
                    .rounded(px(radius::CONTROL))
                    .bg(colors::ACCENT)
                    .child(
                        text("Filter")
                            .text_size(px(8.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::ON_ACCENT),
                    )
                    .into_element(),
            )
        } else {
            None
        })
        .children(caret.map(|c| text(c).text_color(colors::ACCENT).into_element()))
        .on_press(move || {
            menu_signal.update(|current| {
                if current.as_deref() == Some(menu_id.as_str()) {
                    *current = None;
                } else {
                    *current = Some(menu_id.clone());
                }
            });
        })
        .into_element()
}

/// Screener data grid: fixed-width header row, scrollable row list, shortlist
/// stars, sort-and-press lanes. A plain builder fn (the `.agents/HANDOFF.md`
/// Js-safety boundary keeps Rust-domain payloads out of nsh's `Props`/JS
/// bridge, and the grid owns its state mutations directly via its
/// `Signal`s). The page mounts it inside a `dynamic` region that tracks the
/// synchronous `.get()` reads below; every press updates a signal, which
/// re-runs the region with fresh rows. PascalCase mirrors the repo's nsh
/// component builders (`Icon`, `Glyph`, …). The header's column set follows
/// the mounted asset class: the active tab's columns for stocks, the class's
/// fixed set (`asset_columns`) otherwise.
#[allow(non_snake_case)]
#[allow(clippy::too_many_arguments)]
pub fn ScreenerTableGrid(
    records: Signal<Vec<ScreenerRecord>>,
    asset: Signal<ScreenerAssetType>,
    tab: Signal<ScreenerViewTab>,
    sort_state: Signal<Option<SortState>>,
    active_symbol: Signal<String>,
    drawer_open: Signal<bool>,
    shortlisted: Signal<Vec<String>>,
    visible_columns: Signal<Vec<String>>,
    criteria: Signal<FilterCriteria>,
) -> View {
    // The presses mutate these: keep copies of the params before the
    // synchronous reads shadow them into plain values.
    let sort_signal = sort_state;
    let active_signal = active_symbol;
    let drawer_signal = drawer_open;
    let shortlist_signal = shortlisted;
    let criteria_signal = criteria;
    let visible_signal = visible_columns;
    let header_menu_open: Signal<Option<String>> = signal(None);

    let records = records.get();
    let all_criteria = criteria.get();
    let sector_options: Vec<String> = {
        let mut sectors: Vec<String> = records
            .iter()
            .filter_map(|record| match record {
                ScreenerRecord::Stock(inst) => Some(inst.sector.clone()),
                _ => None,
            })
            .collect();
        sectors.sort();
        sectors.dedup();
        sectors
    };
    let asset = asset.get();
    let tab = tab.get();
    let sort_state = sort_state.get();
    let active_symbol = active_symbol.get();
    let shortlisted = shortlisted.get();
    let visible_columns = visible_columns.get();
    let columns: Vec<&'static ScreenerColumn> = if asset == ScreenerAssetType::Stocks {
        visible_tab_columns(tab, &visible_columns)
    } else {
        visible_asset_columns(asset, &visible_columns)
    };

    let header = view()
        .flex_row()
        .items_center()
        .h(px(30.0))
        .px(px(2.0))
        .flex_shrink_0()
        .child(view().flex_shrink_0().w(px(44.0)))
        .children(columns.iter().map(|col| {
            header_cell(
                col,
                sort_state.as_ref(),
                sort_signal,
                header_menu_open,
                &all_criteria,
            )
        }));

    let body = scroll()
        .flex_1()
        .flex_col()
        .w_full()
        .gap(px(2.0))
        .children(records.iter().map(|record| {
            grid_row_record(
                record,
                asset,
                tab,
                &visible_columns,
                active_symbol == record.symbol(),
                shortlisted.iter().any(|s| s == record.symbol()),
                active_signal,
                drawer_signal,
                shortlist_signal,
            )
        }));

    view()
        .flex_col()
        .w_full()
        .h_full()
        .child(header)
        .child(dynamic(move || match header_menu_open.get() {
            Some(open_id) => match columns.iter().find(|col| col.id == open_id) {
                Some(col) => column_menu(
                    col,
                    sector_options.clone(),
                    criteria_signal,
                    visible_signal,
                    sort_signal,
                ),
                None => view(),
            },
            None => view(),
        }))
        .child(if records.is_empty() {
            view()
                .flex_1()
                .items_center()
                .justify_center()
                .child(text("No instruments match the active screen").text_muted())
                .into_element()
        } else {
            body.into_element()
        })
}
