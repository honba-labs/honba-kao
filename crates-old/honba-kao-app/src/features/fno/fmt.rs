//! Display formatting shared by the F&O analytics pages.
//!
//! Money stays in whole paise (`i64`) until the final string is built. Ratios
//! such as annualized basis are kept as integer centi-percent.

/// Group an integer with thousands separators, e.g. `24850` -> `24,850`.
pub(super) fn group_int(value: i64) -> String {
    let digits = value.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if value < 0 { format!("-{out}") } else { out }
}

/// Format paise as rupees with two decimals, e.g. `1425` -> `14.25`.
pub(super) fn fmt_paise(paise: i64) -> String {
    let sign = if paise < 0 { "-" } else { "" };
    let a = paise.unsigned_abs();
    format!("{sign}{}.{:02}", group_int((a / 100) as i64), a % 100)
}

/// Signed paise as rupees: `+4025` -> `+40.25`, `-310` -> `-3.10`.
pub(super) fn fmt_paise_signed(paise: i64) -> String {
    let sign = if paise > 0 { "+" } else { "" };
    format!("{sign}{}", fmt_paise(paise))
}

/// Signed integer count: `1200` -> `+1,200`, `-40` -> `-40`.
pub(super) fn fmt_signed(value: i64) -> String {
    let sign = if value > 0 { "+" } else { "" };
    format!("{sign}{}", group_int(value))
}

/// Centi-percent (hundredths of a percent) as `%`: `203` -> `2.03%`.
pub(super) fn fmt_centi_pct(centi: i64) -> String {
    let sign = if centi < 0 { "-" } else { "" };
    let a = centi.unsigned_abs();
    format!("{sign}{}.{:02}%", a / 100, a % 100)
}
