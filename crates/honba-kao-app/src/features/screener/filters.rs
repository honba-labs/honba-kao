use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use super::types::{FilterCriteria, ScreenerEquityRecord, ScreenerRecord};

/// Keeps (and clones) every record matching `criteria`, preserving input order.
pub fn filter_records(
    records: &[ScreenerEquityRecord],
    criteria: &FilterCriteria,
) -> Vec<ScreenerEquityRecord> {
    records
        .iter()
        .filter(|record| criteria.matches(record))
        .cloned()
        .collect()
}

fn directed(ordering: std::cmp::Ordering, ascending: bool) -> std::cmp::Ordering {
    if ascending {
        ordering
    } else {
        ordering.reverse()
    }
}

/// Stable sort by column `key`; unknown keys return an unsorted clone.
pub fn sort_records(
    records: &[ScreenerEquityRecord],
    key: &str,
    ascending: bool,
) -> Vec<ScreenerEquityRecord> {
    let mut sorted = records.to_vec();
    match key {
        "symbol" => sorted.sort_by(|a, b| {
            let ordering = a.symbol.to_lowercase().cmp(&b.symbol.to_lowercase());
            directed(ordering, ascending)
        }),
        "price" => sorted.sort_by(|a, b| directed(a.price.cmp(&b.price), ascending)),
        "change_pct" => sorted.sort_by(|a, b| directed(a.change_pct.cmp(&b.change_pct), ascending)),
        "volume" => sorted.sort_by(|a, b| directed(a.volume.cmp(&b.volume), ascending)),
        "market_cap_cr" => {
            sorted.sort_by(|a, b| directed(a.market_cap_cr.cmp(&b.market_cap_cr), ascending))
        }
        "pe_ratio" => sorted.sort_by(|a, b| directed(a.pe_ratio.cmp(&b.pe_ratio), ascending)),
        _ => {}
    }
    sorted
}

/// Inserts western thousands separators into a plain digit string.
fn group_thousands(digits: &str) -> String {
    let mut chars: Vec<char> = digits.chars().collect();
    let mut index = chars.len();
    while index > 3 {
        index -= 3;
        chars.insert(index, ',');
    }
    chars.into_iter().collect()
}

/// Rounds to 2 decimal places and renders exactly two fraction digits
/// (padding with `0` when the rounded value carries a shorter scale).
fn fixed2(value: Decimal) -> String {
    let rounded = value.round_dp(2);
    let text = rounded.to_string();
    let body = text.strip_prefix('-').unwrap_or(&text);
    let (int_part, frac) = match body.split_once('.') {
        Some((integer, fraction)) => (integer, fraction),
        None => (body, ""),
    };
    let mut fraction = frac.to_string();
    while fraction.len() < 2 {
        fraction.push('0');
    }
    if text.starts_with('-') {
        format!("-{int_part}.{fraction}")
    } else {
        format!("{int_part}.{fraction}")
    }
}

/// "₹2,842.50"
pub fn format_price(price: Decimal) -> String {
    let body = fixed2(price);
    let (sign, rest) = match body.strip_prefix('-') {
        Some(tail) => ("-", tail),
        None => ("", body.as_str()),
    };
    let (int_part, frac) = rest.split_once('.').unwrap_or((rest, "00"));
    format!("₹{sign}{}.{}", group_thousands(int_part), frac)
}

/// "+1.87%" / "-0.40%" / "0.00%"
pub fn format_change_pct(pct: Decimal) -> String {
    let rounded = pct.round_dp(2);
    let body = fixed2(rounded);
    if rounded > Decimal::ZERO {
        format!("+{body}%")
    } else {
        format!("{body}%")
    }
}

/// "6.5 Cr" / "42.0 L" / "80,050"
pub fn format_volume_compact(volume: u64) -> String {
    if volume >= 10_000_000 {
        let crore = volume / 10_000_000;
        let fraction = (volume % 10_000_000) / 1_000_000;
        format!("{crore}.{fraction} Cr")
    } else if volume >= 100_000 {
        let lakh = volume / 100_000;
        let fraction = (volume % 100_000) / 10_000;
        format!("{lakh}.{fraction} L")
    } else {
        group_thousands(&volume.to_string())
    }
}

/// Strips one trailing zero from the fraction ("19.20" → "19.2").
fn trim_fraction_zero(body: &str) -> String {
    match body.split_once('.') {
        Some((int_part, frac)) if frac.ends_with('0') => {
            format!("{int_part}.{}", &frac[..frac.len() - 1])
        }
        _ => body.to_string(),
    }
}

/// "₹19.2L Cr" (≥ 1 lakh crore) / "₹63,000 Cr"
pub fn format_mcap_rs_cr(mcap_cr: Decimal) -> String {
    if mcap_cr >= dec!(100_000) {
        let lakh = mcap_cr / dec!(100_000);
        let body = trim_fraction_zero(&fixed2(lakh));
        let (int_part, frac) = match body.split_once('.') {
            Some((integer, fraction)) => (integer, Some(fraction)),
            None => (body.as_str(), None),
        };
        let grouped = group_thousands(int_part);
        match frac {
            Some(fraction) => format!("₹{grouped}.{fraction}L Cr"),
            None => format!("₹{grouped}L Cr"),
        }
    } else {
        let whole = mcap_cr.round_dp(0).to_string();
        format!("₹{} Cr", group_thousands(&whole))
    }
}

/// "₹725,000 Cr" — whole-crore figure with thousands separators; negatives
/// keep their leading minus sign.
pub fn format_cr_amount(amount_cr: Decimal) -> String {
    let whole = amount_cr.round_dp(0).to_string();
    let (sign, digits) = match whole.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", whole.as_str()),
    };
    format!("₹{sign}{} Cr", group_thousands(digits))
}

/// Supertrend direction glyph: "▲ Buy" for +1, "▼ Sell" for -1, "—" otherwise.
pub fn format_supertrend(direction: i8) -> String {
    match direction {
        1 => "▲ Buy".to_string(),
        -1 => "▼ Sell".to_string(),
        _ => "—".to_string(),
    }
}

/// Keeps Stock records filtered by the full `FilterCriteria` predicate and
/// non-stock records by search only (symbol + name, lowercase contains).
pub fn filter_universe(
    records: &[ScreenerRecord],
    criteria: &FilterCriteria,
) -> Vec<ScreenerRecord> {
    let query = criteria.search_query.trim().to_lowercase();
    let mut kept = Vec::new();
    for record in records {
        let include = match record {
            ScreenerRecord::Stock(stock) => criteria.matches(stock),
            other => {
                query.is_empty()
                    || other.symbol().to_lowercase().contains(&query)
                    || other.name().to_lowercase().contains(&query)
            }
        };
        if include {
            kept.push(record.clone());
        }
    }
    kept
}

/// Stable sort of the record union. `"symbol"` sorts identity strings and
/// `"maturity_date"` sorts Bond maturity strings lexically (non-bonds compare
/// equal); numeric keys resolve via `ScreenerRecord::sort_decimal` and only
/// sort when every record yields `Some`. Unknown keys return an unsorted clone.
pub fn sort_universe(
    records: &[ScreenerRecord],
    key: &str,
    ascending: bool,
) -> Vec<ScreenerRecord> {
    let mut sorted = records.to_vec();
    match key {
        "symbol" => sorted.sort_by(|a, b| {
            let ordering = a.symbol().to_lowercase().cmp(&b.symbol().to_lowercase());
            directed(ordering, ascending)
        }),
        "maturity_date" => sorted.sort_by(|a, b| {
            let ordering = match (a, b) {
                (ScreenerRecord::Bond(lhs), ScreenerRecord::Bond(rhs)) => {
                    lhs.maturity_date.cmp(&rhs.maturity_date)
                }
                _ => std::cmp::Ordering::Equal,
            };
            directed(ordering, ascending)
        }),
        _ => {
            let anchored: Option<Vec<Decimal>> =
                records.iter().map(|r| r.sort_decimal(key)).collect();
            if let Some(values) = anchored {
                let mut paired: Vec<(Decimal, ScreenerRecord)> =
                    values.into_iter().zip(records.iter().cloned()).collect();
                paired.sort_by(|a, b| directed(a.0.cmp(&b.0), ascending));
                sorted = paired.into_iter().map(|(_, record)| record).collect();
            }
        }
    }
    sorted
}

/// Exposes the internal `fixed2` rounding as a public helper ("6.10").
pub fn format_number2(value: Decimal) -> String {
    fixed2(value)
}

/// "7.26 %" — two decimals plus a space-percent suffix.
pub fn format_percent2(value: Decimal) -> String {
    format!("{} %", format_number2(value))
}

/// Renders one table cell for a union record; identity/star/range keys yield
/// `""` and fall through to the default arm; tab metrics render via formatters.
pub fn record_cell(record: &ScreenerRecord, key: &str) -> String {
    match (record, key) {
        (ScreenerRecord::Stock(r), "price") => format_price(r.price),
        (ScreenerRecord::Stock(r), "change_pct") => format_change_pct(r.change_pct),
        (ScreenerRecord::Stock(r), "volume") => format_volume_compact(r.volume),
        (ScreenerRecord::Stock(r), "rvol") => format!("{}x", r.rvol.round_dp(2)),
        (ScreenerRecord::Stock(r), "market_cap_cr") => format_mcap_rs_cr(r.market_cap_cr),
        (ScreenerRecord::Stock(r), "pe_ratio") => {
            if r.pe_ratio == Decimal::ZERO {
                "—".to_string()
            } else {
                r.pe_ratio.round_dp(1).to_string()
            }
        }
        (ScreenerRecord::Stock(r), "rsi_14") => format_percent2(r.rsi_14),
        (ScreenerRecord::Stock(r), "atr_pct") => format_percent2(r.atr_pct),
        (ScreenerRecord::Stock(r), "bb_bandwidth") => format_percent2(r.bb_bandwidth),
        (ScreenerRecord::Stock(r), "dividend_yield") => format_percent2(r.dividend_yield),
        (ScreenerRecord::Stock(r), "payout_ratio") => format_percent2(r.payout_ratio),
        (ScreenerRecord::Stock(r), "roe") => format_percent2(r.roe),
        (ScreenerRecord::Stock(r), "roce") => format_percent2(r.roce),
        (ScreenerRecord::Stock(r), "operating_margin") => format_percent2(r.operating_margin),
        (ScreenerRecord::Stock(r), "net_margin") => format_percent2(r.net_margin),
        (ScreenerRecord::Stock(r), "return_1d") => format_change_pct(r.return_1d),
        (ScreenerRecord::Stock(r), "return_1w") => format_change_pct(r.return_1w),
        (ScreenerRecord::Stock(r), "return_1m") => format_change_pct(r.return_1m),
        (ScreenerRecord::Stock(r), "return_3m") => format_change_pct(r.return_3m),
        (ScreenerRecord::Stock(r), "return_6m") => format_change_pct(r.return_6m),
        (ScreenerRecord::Stock(r), "return_ytd") => format_change_pct(r.return_ytd),
        (ScreenerRecord::Stock(r), "return_1y") => format_change_pct(r.return_1y),
        (ScreenerRecord::Stock(r), "return_3y") => format_change_pct(r.return_3y),
        (ScreenerRecord::Stock(r), "revenue_growth_yoy") => format_change_pct(r.revenue_growth_yoy),
        (ScreenerRecord::Stock(r), "macd_hist") => format_number2(r.macd_hist),
        (ScreenerRecord::Stock(r), "sma_20") => format_number2(r.sma_20),
        (ScreenerRecord::Stock(r), "sma_50") => format_number2(r.sma_50),
        (ScreenerRecord::Stock(r), "sma_200") => format_number2(r.sma_200),
        (ScreenerRecord::Stock(r), "ema_9") => format_number2(r.ema_9),
        (ScreenerRecord::Stock(r), "ema_21") => format_number2(r.ema_21),
        (ScreenerRecord::Stock(r), "forward_pe") => format_number2(r.forward_pe),
        (ScreenerRecord::Stock(r), "pb_ratio") => format_number2(r.pb_ratio),
        (ScreenerRecord::Stock(r), "ps_ratio") => format_number2(r.ps_ratio),
        (ScreenerRecord::Stock(r), "ev_ebitda") => format_number2(r.ev_ebitda),
        (ScreenerRecord::Stock(r), "peg_ratio") => format_number2(r.peg_ratio),
        (ScreenerRecord::Stock(r), "dividend_per_share") => format_number2(r.dividend_per_share),
        (ScreenerRecord::Stock(r), "debt_to_equity") => format_number2(r.debt_to_equity),
        (ScreenerRecord::Stock(r), "current_ratio") => format_number2(r.current_ratio),
        (ScreenerRecord::Stock(r), "interest_coverage") => format_number2(r.interest_coverage),
        (ScreenerRecord::Stock(r), "book_value_per_share") => {
            format_number2(r.book_value_per_share)
        }
        (ScreenerRecord::Stock(r), "eps") => format_number2(r.eps),
        (ScreenerRecord::Stock(r), "revenue_cr") => format_cr_amount(r.revenue_cr),
        (ScreenerRecord::Stock(r), "net_profit_cr") => format_cr_amount(r.net_profit_cr),
        (ScreenerRecord::Stock(r), "fcf_cr") => format_cr_amount(r.fcf_cr),
        (ScreenerRecord::Stock(r), "supertrend_direction") => {
            format_supertrend(r.supertrend_direction)
        }
        (ScreenerRecord::Etf(r), "ltp") => format_price(r.ltp),
        (ScreenerRecord::Etf(r), "nav") => format_price(r.nav),
        (ScreenerRecord::Etf(r), "premium_discount_pct") => {
            format_change_pct(r.premium_discount_pct)
        }
        (ScreenerRecord::Etf(r), "return_1y") => format_change_pct(r.return_1y),
        (ScreenerRecord::Etf(r), "aum_cr") => format_mcap_rs_cr(r.aum_cr),
        (ScreenerRecord::Etf(r), "expense_ratio") => format_percent2(r.expense_ratio),
        (ScreenerRecord::Etf(r), "tracking_error") => format_percent2(r.tracking_error),
        (ScreenerRecord::Etf(r), "volume") => format_volume_compact(r.volume),
        (ScreenerRecord::Bond(r), "coupon_rate") => format_percent2(r.coupon_rate),
        (ScreenerRecord::Bond(r), "yield_to_maturity") => format_percent2(r.yield_to_maturity),
        (ScreenerRecord::Bond(r), "clean_price") => format_price(r.clean_price),
        (ScreenerRecord::Bond(r), "face_value") => format_price(r.face_value),
        (ScreenerRecord::Bond(r), "modified_duration") => format_number2(r.modified_duration),
        (ScreenerRecord::Bond(r), "maturity_date") => r.maturity_date.clone(),
        (ScreenerRecord::Bond(r), "credit_rating") => r.credit_rating.clone(),
        (ScreenerRecord::Bond(r), "bond_type") => r.bond_type.clone(),
        (ScreenerRecord::Mf(r), "nav") => format_price(r.nav),
        (ScreenerRecord::Mf(r), "aum_cr") => format_mcap_rs_cr(r.aum_cr),
        (ScreenerRecord::Mf(r), "expense_ratio") => format_percent2(r.expense_ratio),
        (ScreenerRecord::Mf(r), "cagr_3y") => format_change_pct(r.cagr_3y),
        (ScreenerRecord::Mf(r), "cagr_5y") => format_change_pct(r.cagr_5y),
        (ScreenerRecord::Mf(r), "risk_grade") => r.risk_grade.clone(),
        (ScreenerRecord::Mf(r), "category") => r.category.clone(),
        _ => String::new(),
    }
}
