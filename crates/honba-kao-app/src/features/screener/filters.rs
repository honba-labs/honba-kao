use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use super::types::{FilterCriteria, ScreenerEquityRecord};

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
