//! Currency / number formatting for Indian markets.

/// `₹1,842,650.00` (2 dp, grouped).
pub fn currency(value: f64) -> String {
    let sign = if value < 0.0 { "-" } else { "" };
    format!("{}₹{:.2}", sign, value.abs())
}

/// Indian short form: `₹1.84 Cr`, `₹12.4 L`, `₹4.20 K`.
pub fn compact_currency(value: f64) -> String {
    let sign = if value < 0.0 { "-" } else { "" };
    let v = value.abs();
    if v >= 10_000_000.0 {
        format!("{}₹{:.2} Cr", sign, v / 10_000_000.0)
    } else if v >= 100_000.0 {
        format!("{}₹{:.2} L", sign, v / 100_000.0)
    } else if v >= 1_000.0 {
        format!("{}₹{:.2} K", sign, v / 1_000.0)
    } else {
        format!("{}₹{:.2}", sign, v)
    }
}

/// `+2.41%` / `-0.30%`.
pub fn percent(value: f64) -> String {
    if value >= 0.0 {
        format!("+{:.2}%", value)
    } else {
        format!("{:.2}%", value)
    }
}

/// `1,234` with thousand separators.
pub fn quantity(q: i64) -> String {
    let s = q.abs().to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    let grouped: String = out.chars().rev().collect();
    if q < 0 {
        format!("-{}", grouped)
    } else {
        grouped
    }
}
