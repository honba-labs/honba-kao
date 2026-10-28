//! Currency / number formatting for Indian markets.

/// `₹18,42,650.00` (Indian lakh/crore grouping, 2 dp).
pub fn currency(value: f64) -> String {
    let sign = if value < 0.0 { "-" } else { "" };
    let magnitude = value.abs();
    let formatted = format!("{:.2}", magnitude);
    let (integer, fraction) = formatted.split_once('.').unwrap_or((&formatted, "00"));

    format!("{}₹{}.{}", sign, group_indian(integer), fraction)
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
    let grouped = group_indian(&q.abs().to_string());
    if q < 0 {
        format!("-{}", grouped)
    } else {
        grouped
    }
}

/// Group an unsigned integer string the Indian way: three digits, then pairs.
///
/// `1234567` → `12,34,567`; `123456` → `1,23,456`; `1234` → `1,234`.
fn group_indian(integer: &str) -> String {
    let digits: Vec<char> = integer.chars().collect();
    if digits.len() <= 3 {
        return integer.to_string();
    }

    let mut groups: Vec<String> = Vec::new();
    let mut end = digits.len();

    // Rightmost group always has three digits.
    groups.push(digits[end - 3..].iter().collect());
    end -= 3;

    while end > 0 {
        let start = end.saturating_sub(2);
        groups.push(digits[start..end].iter().collect());
        end = start;
    }

    groups.reverse();
    groups.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_the_indian_way() {
        assert_eq!(group_indian("1234567"), "12,34,567");
        assert_eq!(group_indian("123456"), "1,23,456");
        assert_eq!(group_indian("1234"), "1,234");
        assert_eq!(group_indian("999"), "999");
        assert_eq!(group_indian("10000000"), "1,00,00,000");
    }

    #[test]
    fn formats_rupees() {
        assert_eq!(currency(1_842_650.0), "₹18,42,650.00");
        assert_eq!(currency(-310.5), "-₹310.50");
        assert_eq!(quantity(-1234567), "-12,34,567");
    }
}
